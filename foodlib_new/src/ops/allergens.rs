//! Allergen classification from Metro article data.
//!
//! Pipeline: Metro article → classify_article() → Vec<ClassifiedProperty>
//!                                              → attach to ingredient via apply_to_ingredient()
//!
//! Stored properties are the union of:
//!  - broad dietary classes from `bundle.categories` (e.g. `fleisch`, `käse`)
//!  - EU-14 allergens from `Contains` features (e.g. `gluten`, `milch`)
//!  - EU-14 allergens from `MayContain` features, prefixed `~spuren-`
//!
//! Dietary flags (`vegan`, `vegetarian`, `lactose_free`, `gluten_free`) are NOT stored;
//! they are derived at render time by `dietary_flags()`. Catering-grade strictness: any
//! `~spuren-X` warning disqualifies the corresponding flag.

use std::collections::HashSet;
use std::sync::Arc;

use sqlx::PgPool;

use crate::entities::allergen::*;
use crate::entities::property::Property;
use crate::error::Result;

use metro_scrape::article::{Article, Feature, FeatureMetaInfo};

#[derive(Clone)]
pub struct AllergenOps {
    pool: Arc<PgPool>,
}

impl AllergenOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Classify a Metro article into the set of properties that should be attached to the
    /// ingredient it sources. Deduplicated.
    pub fn classify_article(article: &Article) -> Vec<ClassifiedProperty> {
        use canonical::*;
        let mut out: HashSet<(String, PropertySource)> = HashSet::new();

        for variant in article.variants.values() {
            for bundle in variant.bundles.values() {
                // First pass: collect category segments for vegan-signal detection.
                // Metro retail taxonomy puts vegan analogues in the section of the food
                // they replace ("vegan meatballs" in the meat aisle). If *any* category
                // for the article explicitly says "vegan"/"vegetarisch", we suppress the
                // meat-class tags that would otherwise come from the meat-section path.
                let mut all_segments: Vec<String> = Vec::new();
                for cat in &bundle.categories {
                    all_segments.extend(category_path_segments(cat));
                }
                let is_vegan_marked = all_segments
                    .iter()
                    .any(|s| s.contains("vegan") || s.contains("vegetarisch"));

                for cat in &bundle.categories {
                    let segments = category_path_segments(cat);
                    for name in classify_category_segments(&segments) {
                        // Suppress meat/animal class category-tags when the product is
                        // labeled vegan/vegetarian in any of its categories.
                        if is_vegan_marked && matches!(name, FLEISCH | SCHWEIN | FISCH) {
                            continue;
                        }
                        out.insert((name.to_string(), PropertySource::Category));
                    }
                }

                let mut flat = Vec::new();
                flatten_features(&bundle.details.features, &mut flat);
                for f in flat {
                    let label = f.label.to_lowercase();
                    let source = match f.meta_info {
                        FeatureMetaInfo::Contains => PropertySource::Contains,
                        FeatureMetaInfo::MayContain => PropertySource::MayContain,
                        _ => continue,
                    };
                    for name in classify_allergen_label(&label) {
                        out.insert((name.to_string(), source));
                    }
                }
            }
        }

        out.into_iter()
            .map(|(canonical_name, source)| ClassifiedProperty {
                canonical_name,
                source,
            })
            .collect()
    }

    /// Apply classified properties to an ingredient. Idempotent: creates missing
    /// `food_properties` rows (case-insensitive lookup), inserts join rows with
    /// `ON CONFLICT DO NOTHING`.
    pub async fn apply_to_ingredient(
        &self,
        ingredient_id: i32,
        properties: &[ClassifiedProperty],
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for cp in properties {
            let stored = cp.source.stored_name(&cp.canonical_name);

            // Case-insensitive find-or-create.
            if let Some(existing) = sqlx::query_scalar!(
                r#"SELECT property_id FROM food_properties WHERE lower(name) = lower($1) LIMIT 1"#,
                stored
            )
            .fetch_optional(&mut *tx)
            .await?
            {
                sqlx::query!(
                    r#"INSERT INTO ingredient_properties (ingredient_id, property_id)
                       VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
                    ingredient_id,
                    existing
                )
                .execute(&mut *tx)
                .await?;
            } else {
                let created = sqlx::query_scalar!(
                    r#"INSERT INTO food_properties (name) VALUES ($1) RETURNING property_id"#,
                    stored
                )
                .fetch_one(&mut *tx)
                .await?;
                sqlx::query!(
                    r#"INSERT INTO ingredient_properties (ingredient_id, property_id)
                       VALUES ($1, $2) ON CONFLICT DO NOTHING"#,
                    ingredient_id,
                    created
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

fn flatten_features<'a>(features: &'a [Feature], out: &mut Vec<&'a Feature>) {
    for f in features {
        out.push(f);
        flatten_features(&f.leafs, out);
    }
}

/// Lowercased list of path components for a Metro category: each level's `display_name`
/// (when present) plus the leaf `name`. Hierarchy is preserved — index 0 is the root.
fn category_path_segments(cat: &metro_scrape::article::Category) -> Vec<String> {
    let mut out: Vec<String> = cat
        .levels
        .iter()
        .filter_map(|l| l.display_name.as_deref())
        .map(str::to_lowercase)
        .collect();
    // Append leaf name (always present); skip if it duplicates the last level.
    let leaf = cat.name.to_lowercase();
    if out.last().map(|s| s.as_str()) != Some(leaf.as_str()) {
        out.push(leaf);
    }
    out
}

/// Map structured Metro category path segments to canonical broad-class property names.
///
/// **Scope**: only classes that are NOT in the EU-14 allergen disclosure list. Dairy/egg
/// products are excluded — they're more accurately picked up from `Contains` features
/// (manufacturer-declared) than from Metro's retail taxonomy (which overincludes plant-
/// based substitutes shelved with the dairy aisle, e.g. vegan cheese under `/Käse`).
///
/// Uses regexes with `\b` word boundaries matched against each segment for precision.
fn classify_category_segments(segments: &[String]) -> Vec<&'static str> {
    use canonical::*;
    use regex::Regex;
    use std::sync::OnceLock;

    static RULES: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
    let rules = RULES.get_or_init(|| {
        let r = |p: &str| Regex::new(p).expect("category regex compiles");
        vec![
            // Dietary-relevant classes that are NOT in the EU-14 list — Contains features
            // don't surface these, so categories are the only signal.
            (r(r"(?i)\b(fleisch|wurst|schinken|speck)\b"), FLEISCH),
            (r(r"(?i)\bschwein\b"), SCHWEIN),
            (r(r"(?i)\b(fisch|meeresfrüchte)\b"), FISCH),
        ]
    });

    let mut out: Vec<&'static str> = Vec::new();
    for seg in segments {
        for (re, canonical) in rules {
            if re.is_match(seg) && !out.contains(canonical) {
                out.push(*canonical);
            }
        }
    }

    out
}

/// Map an allergen feature label (already lowercased) to canonical property names.
/// Returns multiple if the label implies several allergens (e.g. "weizen" → both
/// `weizen` and `gluten`).
///
/// Uses regex with `\b` boundaries to avoid German compound false-positives like
/// `"frisch"` ⊂ `"fisch"` or `"buttermilch"` ⊂ `"butter"` matching too eagerly.
fn classify_allergen_label(label: &str) -> Vec<&'static str> {
    use canonical::*;
    use regex::Regex;
    use std::sync::OnceLock;

    // Patterns are anchored with `\b` on both ends. German compounds are tolerated
    // for stems we *do* want to match (e.g. `\bweizen` matches "weizenmehl" because
    // word characters continue past the stem — that's correct; the stem is the
    // start of a German compound). But `\bfisch\b` rejects "frisch" since `\b`
    // needs a word/non-word boundary before, and `r` (the preceding char) is a
    // word character — so no boundary there.
    static RULES: OnceLock<Vec<(Regex, &'static [&'static str])>> = OnceLock::new();
    let rules = RULES.get_or_init(|| {
        let r = |p: &str| Regex::new(p).expect("allergen regex compiles");
        vec![
            // Gluten-bearing cereals. Dinkel (spelt) is botanically distinct from
            // Weizen (wheat) — both contain gluten but they're separate EU-14
            // sub-entries. So weizen and dinkel emit *different* specific tags.
            (r(r"\b(weizen|hartweizen)"), &[WEIZEN, GLUTEN][..]),
            (r(r"\bdinkel"), &[GLUTEN][..]),
            (r(r"\b(gerste|roggen|hafer|kamut)\b"), &[GLUTEN][..]),
            (r(r"\bgluten"), &[GLUTEN][..]),

            // Dairy stems. Note: this only matches the genuine dairy items.
            // Plant-based pseudo-milks (kokosmilch, sojamilch, hafermilch,
            // mandelmilch, reismilch) MUST not trigger here — handled below by
            // explicit exclusion before the regex runs.
            (r(r"\b(milch|vollmilch|magermilch|kuhmilch|laktose|molke|sahne|butter|joghurt|käse|milcheiweiß|milcheiweiss|pasteurisierte kuhmilch)"), &[MILCH][..]),

            // Eggs — longer stems like "eier"/"eigelb" use left-anchored boundary so they
            // match within compounds ("eigelbpulver"). The bare 2-char "ei" needs both
            // boundaries to avoid matching "eis"/"eisen"/"eiche"/"freisetzen".
            (r(r"\b(eier|eigelb|eiweiss|eiweiß|vollei|hühnerei|huehnerei)"), &[EIER][..]),
            (r(r"\bei\b"), &[EIER][..]),

            (r(r"\bsoja"), &[SOJA][..]),
            // Peanut stem accepts both the singular vowel "u" (erdnuss/erdnüsse
            // share the prefix "erdn") and the umlaut form. Old pattern
            // `\berdnu[sß]` silently failed on the plural form because the umlaut
            // shifts the 5th character from "u" to "ü".
            (r(r"\berdn[uü]"), &[ERDNUESSE][..]),

            // Tree nuts
            (r(r"\b(schalenfrüchte|haselnuss|haselnüsse|haselnusskerne|cashew|cashewkern|mandel|walnuss|walnüsse|walnusskern|pistazi|paranu[sß]|pekannu[sß]|pekanu[sß]|macadamia|queenslandnu[sß]|nu[sß]kern)"), &[SCHALENFRUECHTE][..]),
            // "nüsse" as a free-standing word, but exclude "erdnüsse" (peanuts go to ERDNUESSE).
            (r(r"(?:^|\W)nüsse\b"), &[SCHALENFRUECHTE][..]),

            (r(r"\bsesam"), &[SESAM][..]),
            (r(r"\bsellerie"), &[SELLERIE][..]),
            (r(r"\bsenf"), &[SENF][..]),
            (r(r"\blupine"), &[LUPINE][..]),
            (r(r"\b(schwefeldioxid|sulfit|sulphit)"), &[SULFITE][..]),
            (r(r"\bfisch"), &[FISCH][..]),

            // Broad dietary classes that are NOT in the EU-14 list but show up in
            // Contains labels for meat products. Manufacturer-declared ingredients
            // like "Schweinefleisch" or "Schweineschmalz" reliably indicate pork —
            // category paths often only say `/Wurst & Schinken/` and miss this.
            // FLEISCH is paired with SCHWEIN because all pork implies meat.
            (r(r"\bschwein"), &[SCHWEIN, FLEISCH][..]),
        ]
    });

    // Strip plant-based pseudo-milks from the label before running the dairy rule.
    // `Kokosmilch`, `Sojamilch`, `Hafermilch`, `Mandelmilch`, `Reismilch`,
    // `Dinkelmilch`, `Pflanzenmilch` etc. are not the EU-14 allergen milk.
    // We replace them with a neutral marker so the milk regex doesn't match.
    static PLANT_MILK: OnceLock<Regex> = OnceLock::new();
    let plant_milk = PLANT_MILK.get_or_init(|| {
        Regex::new(
            r"\b(kokos(?:nuss)?|soja|hafer|mandel|reis|dinkel|nuss|pflanzen|pflanzlich(?:e)?)[- ]?milch\b",
        )
        .expect("plant-milk regex compiles")
    });
    let sanitized: std::borrow::Cow<'_, str> = plant_milk.replace_all(label, "<<plant>>");

    let mut out: Vec<&'static str> = Vec::new();
    for (re, canonicals) in rules {
        if re.is_match(&sanitized) {
            for c in *canonicals {
                if !out.contains(c) {
                    out.push(c);
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Derive dietary flags from a recipe's aggregated property set.
///
/// Strict mode (catering): `~spuren-X` (may-contain) properties disqualify the matching flag.
/// A "may contain milk" warning means the recipe is NOT lactose-free and NOT vegan.
pub fn dietary_flags(properties: &[Property]) -> DietaryFlags {
    use canonical::*;
    let names: HashSet<String> = properties.iter().map(|p| p.name.to_lowercase()).collect();

    let has = |n: &str| names.contains(n);
    // Strict: a property `~spuren-X` blocks the same axis as having `X` itself.
    let has_any_form = |canonical: &str| has(canonical) || has(&format!("~spuren-{canonical}"));

    let meat_props = [
        FLEISCH,
        SCHWEIN,
        FISCH,
        "gelatine",
        "lab",
        "krebstiere",
        "weichtiere",
    ];
    let dairy_props = [MILCH, MILCHPRODUKT, KAESE, "milcheiweiß"];
    let egg_props = [EIER, EI_PRODUKT];
    let other_animal_props: [&str; 2] = ["tierische produkte", "honig"];
    let gluten_props = [
        GLUTEN, WEIZEN, "gerste", "roggen", "hafer", "dinkel", "kamut",
    ];

    let vegetarian = !meat_props.iter().any(|p| has_any_form(p));
    let vegan = vegetarian
        && !dairy_props.iter().any(|p| has_any_form(p))
        && !egg_props.iter().any(|p| has_any_form(p))
        && !other_animal_props.iter().any(|p| has_any_form(p));
    let vegetarian = vegetarian && !vegan;
    let lactose_free = !dairy_props.iter().any(|p| has_any_form(p)) && !vegan;
    let gluten_free = !gluten_props.iter().any(|p| has_any_form(p));

    DietaryFlags {
        vegan,
        vegetarian,
        lactose_free,
        gluten_free,
    }
}

/// Hierarchy of allergen properties: each entry maps a parent canonical name to
/// the set of more-specific child names it subsumes. When both are present in a
/// recipe's aggregated property set, the parent is sufficient — we drop children.
///
/// Example: `milch` subsumes `milcheiweiß` and `milchprodukt`, so if a recipe is
/// already tagged `milch`, displaying `Milch, Milcheiweiß, Milchprodukt` is just
/// noise. After compaction it reads `Milch` only.
fn allergen_hierarchy() -> &'static [(&'static str, &'static [&'static str])] {
    &[
        (
            "milch",
            &["milcheiweiß", "milcheiweiss", "milchprodukt", "käse"],
        ),
        ("eier", &["ei-produkt"]),
        ("fleisch", &["schwein"]),
        // Gluten subsumes specific cereals. Even if only `weizen` is present and
        // not `gluten`, we still want to show both because the cereal name is
        // additional information. Only collapse when the *parent* gluten tag is
        // present alongside.
        (
            "gluten",
            &["weizen", "roggen", "gerste", "hafer", "dinkel", "kamut"],
        ),
    ]
}

/// Drop children from `names` when their parent is present. Input must already be
/// lowercased, deduplicated, and sorted. Preserves all unrelated entries.
pub fn compact_allergen_set(names: &[String]) -> Vec<String> {
    let present: std::collections::HashSet<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut suppressed = std::collections::HashSet::new();
    for (parent, children) in allergen_hierarchy() {
        if present.contains(parent) {
            for c in *children {
                suppressed.insert(*c);
            }
        }
    }
    names
        .iter()
        .filter(|n| !suppressed.contains(n.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(name: &str) -> Property {
        Property {
            id: 0,
            name: name.to_string(),
        }
    }

    fn segs(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_lowercase()).collect()
    }

    #[test]
    fn category_classification() {
        assert_eq!(
            classify_category_segments(&segs(&[
                "food",
                "fleisch & wurstwaren",
                "fleisch",
                "schwein"
            ])),
            vec!["fleisch", "schwein"]
        );
        assert_eq!(
            classify_category_segments(&segs(&[
                "food",
                "fleisch & wurstwaren",
                "wurst & schinken",
                "rohwurst"
            ])),
            vec!["fleisch"],
            "wurst-only path implies fleisch but not schwein"
        );
        // Dairy/egg categories no longer trigger — manufacturer Contains features are
        // the more precise signal for milk/eggs. Categories overinclude (vegan
        // substitutes shelved under /Käse, plant-based margarine under /Molkereiprodukte).
        assert!(
            classify_category_segments(&segs(&["food", "molkereiprodukte", "milch"])).is_empty(),
            "dairy categories no longer classified — rely on Contains features"
        );
        assert!(
            classify_category_segments(&segs(&["food", "käse"])).is_empty(),
            "käse no longer classified by category"
        );
        assert!(classify_category_segments(&segs(&["food", "gemüse", "salat"])).is_empty());
    }

    #[test]
    fn allergen_label_classification() {
        assert_eq!(
            classify_allergen_label("milch und daraus gewonnene erzeugnisse"),
            vec!["milch"]
        );
        assert_eq!(
            classify_allergen_label("weizenmehl"),
            vec!["gluten", "weizen"]
        );
        assert_eq!(
            classify_allergen_label("dinkelmehl"),
            vec!["gluten"],
            "spelt has gluten but is not wheat — keep them distinct"
        );
        assert_eq!(
            classify_allergen_label("dinkelvollkornmehl"),
            vec!["gluten"],
        );
        assert_eq!(
            classify_allergen_label("getreidekörner, die gluten enthalten"),
            vec!["gluten"]
        );
        assert_eq!(classify_allergen_label("eigelbpulver"), vec!["eier"]);
        assert!(
            classify_allergen_label("eis").is_empty(),
            "must not match 'eis' for 'eier'"
        );
        assert!(
            classify_allergen_label("frische zutaten").is_empty(),
            "frisch ≠ fisch"
        );
        assert!(
            classify_allergen_label("frischhefe").is_empty(),
            "frischhefe ≠ fisch"
        );
        assert_eq!(
            classify_allergen_label("fischfilet"),
            vec!["fisch"],
            "fisch stems still match"
        );
        assert!(
            classify_allergen_label("brokkoli").is_empty(),
            "vegetables don't match"
        );
        assert!(
            classify_allergen_label("salz").is_empty(),
            "salt does not match"
        );
        assert!(
            classify_allergen_label("kokosmilch").is_empty(),
            "coconut milk is not dairy"
        );
        assert!(
            classify_allergen_label("sojamilch").is_empty(),
            "soy milk is not dairy"
        );
        assert!(
            classify_allergen_label("hafermilch").is_empty(),
            "oat milk is not dairy"
        );
        assert!(
            classify_allergen_label("mandelmilch").is_empty(),
            "almond milk is not dairy"
        );
        assert_eq!(
            classify_allergen_label("vollmilch"),
            vec!["milch"],
            "actual milk still matches"
        );
        assert_eq!(
            classify_allergen_label("milch und daraus gewonnene erzeugnisse"),
            vec!["milch"]
        );
        assert_eq!(
            classify_allergen_label("sesamsamen und daraus gewonnene erzeugnisse"),
            vec!["sesamsamen"]
        );
        assert_eq!(
            classify_allergen_label("e220 - schwefeldioxid"),
            vec!["schwefeldioxid & sulfite"]
        );
        assert_eq!(
            classify_allergen_label("haselnusskerne"),
            vec!["schalenfrüchte"]
        );
        // Plural "erdnüsse" must match (regression: previous `\berdnu[sß]` only
        // matched the singular stem because it required `u`, not `ü`).
        assert_eq!(
            classify_allergen_label("erdnüsse und daraus gewonnene erzeugnisse"),
            vec!["erdnüsse"]
        );
        assert_eq!(classify_allergen_label("erdnuss"), vec!["erdnüsse"]);
        // Manufacturer-declared pork ingredient → schwein + fleisch. Category
        // paths for Wurst/Schinken often don't reveal the meat type, but the
        // Contains label does.
        let mut pork = classify_allergen_label("schweinefleisch");
        pork.sort();
        assert_eq!(pork, vec!["fleisch", "schwein"]);
        let mut pork2 = classify_allergen_label("schweineschmalz");
        pork2.sort();
        assert_eq!(pork2, vec!["fleisch", "schwein"]);
    }

    #[test]
    fn compact_allergens_drops_subsumed_children() {
        let names: Vec<String> = [
            "milch",
            "milcheiweiß",
            "milchprodukt",
            "käse",
            "senf",
            "weizen",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        let mut compact = compact_allergen_set(&names);
        compact.sort();
        // milch absorbs milcheiweiß, milchprodukt, käse. weizen stays because gluten parent absent.
        assert_eq!(compact, vec!["milch", "senf", "weizen"]);
    }

    #[test]
    fn compact_allergens_keeps_children_without_parent() {
        let names: Vec<String> = ["weizen", "roggen"].iter().map(|s| s.to_string()).collect();
        assert_eq!(compact_allergen_set(&names), vec!["weizen", "roggen"]);
    }

    #[test]
    fn dietary_flags_basic() {
        let veg = dietary_flags(&[p("gemüse")]);
        assert!(veg.vegan && veg.gluten_free);

        let with_meat = dietary_flags(&[p("fleisch")]);
        assert!(!with_meat.vegetarian);
        assert!(!with_meat.vegan);

        let with_milk = dietary_flags(&[p("milch")]);
        assert!(with_milk.vegetarian);
        assert!(!with_milk.vegan);
        assert!(!with_milk.lactose_free);

        let with_wheat = dietary_flags(&[p("weizen"), p("gluten")]);
        assert!(!with_wheat.gluten_free);
    }

    #[test]
    fn dietary_flags_traces_block_strictly() {
        // Catering rule: "may contain milk" → not lactose-free, not vegan
        let with_trace = dietary_flags(&[p("~spuren-milch")]);
        assert!(!with_trace.lactose_free);
        assert!(!with_trace.vegan);
        assert!(
            with_trace.vegetarian,
            "traces of milk don't make it non-vegetarian"
        );

        let with_gluten_trace = dietary_flags(&[p("~spuren-gluten")]);
        assert!(!with_gluten_trace.gluten_free);
    }
}
