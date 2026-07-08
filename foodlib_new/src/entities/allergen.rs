use serde::{Deserialize, Serialize};

/// A canonical property name we classify into.
///
/// These map 1:1 onto rows in `food_properties.name`. We use a `&'static str` newtype-like
/// pattern (just a `pub const`) rather than an enum so the curated vocabulary can grow without
/// recompiling every call site.
pub mod canonical {
    // Broad dietary classes (from Metro category paths)
    pub const FLEISCH: &str = "fleisch";
    pub const SCHWEIN: &str = "schwein";
    pub const FISCH: &str = "fisch";
    pub const MILCHPRODUKT: &str = "milchprodukt";
    pub const KAESE: &str = "käse";
    pub const EI_PRODUKT: &str = "ei-produkt";

    // EU-14 allergens (from Contains/MayContain features)
    pub const GLUTEN: &str = "gluten";
    pub const WEIZEN: &str = "weizen";
    pub const MILCH: &str = "milch";
    pub const EIER: &str = "eier";
    pub const SOJA: &str = "soja";
    pub const ERDNUESSE: &str = "erdnüsse";
    pub const SCHALENFRUECHTE: &str = "schalenfrüchte";
    pub const SESAM: &str = "sesamsamen";
    pub const SELLERIE: &str = "sellerie";
    pub const SENF: &str = "senf";
    pub const LUPINE: &str = "lupine";
    pub const SULFITE: &str = "schwefeldioxid & sulfite";

    // Common non-EU-14 avoidances. NOT legally-required allergens — these are
    // frequently-requested dietary avoidances (onions, mushrooms, ...) that guests
    // list on their needs form. Classified from the free-text ingredient list and
    // stored under the `~hinweis-` prefix so they stay OUT of the regulatory
    // "Enthält" disclosure and render in a separate "Hinweise" section.
    pub const ZWIEBEL: &str = "zwiebel";
    pub const KNOBLAUCH: &str = "knoblauch";
    pub const PILZ: &str = "pilz";
    pub const APFEL: &str = "apfel";
    pub const KAROTTE: &str = "karotte";
    pub const KICHERERBSE: &str = "kichererbse";
    pub const ZITRUS: &str = "zitrus";
    pub const GUAR: &str = "guar";

    /// Curated common avoidances (non-EU-14). Extend this list as guest needs
    /// dictate; each entry needs a matching rule in `classify_avoidance_label`.
    pub const AVOIDANCES: &[&str] =
        &[ZWIEBEL, KNOBLAUCH, PILZ, APFEL, KAROTTE, KICHERERBSE, ZITRUS, GUAR];

    /// All canonical names users can attach via the UI, ordered for display.
    /// Broad dietary classes first, then EU-14 allergens, then common avoidances.
    pub const ALL: &[&str] = &[
        FLEISCH,
        SCHWEIN,
        FISCH,
        MILCHPRODUKT,
        KAESE,
        EI_PRODUKT,
        GLUTEN,
        WEIZEN,
        MILCH,
        EIER,
        SOJA,
        ERDNUESSE,
        SCHALENFRUECHTE,
        SESAM,
        SELLERIE,
        SENF,
        LUPINE,
        SULFITE,
        ZWIEBEL,
        KNOBLAUCH,
        PILZ,
        APFEL,
        KAROTTE,
        KICHERERBSE,
        ZITRUS,
        GUAR,
    ];
}

/// A property attached to an ingredient with the source kind that produced it.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassifiedProperty {
    pub canonical_name: String,
    pub source: PropertySource,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PropertySource {
    /// From `bundle.categories` taxonomy path.
    Category,
    /// From a `Contains` feature in the article details.
    Contains,
    /// From a `MayContain` feature — stored under the `~spuren-` prefix convention.
    MayContain,
    /// A common non-EU-14 avoidance (onion, mushroom, ...) mined from the free-text
    /// ingredient list. Stored under `~hinweis-` so it stays out of the regulatory
    /// allergen disclosure and renders in a separate informational section.
    Avoidance,
}

impl PropertySource {
    /// The stored property name applies this source's prefix convention.
    /// `Contains`/`Category` use the canonical name directly; `MayContain`
    /// prepends `~spuren-`; `Avoidance` prepends `~hinweis-`.
    pub fn stored_name(&self, canonical: &str) -> String {
        match self {
            Self::Category | Self::Contains => canonical.to_string(),
            Self::MayContain => format!("~spuren-{canonical}"),
            Self::Avoidance => format!("~hinweis-{canonical}"),
        }
    }
}

/// The derived dietary predicates computed over an aggregated property set.
///
/// Never stored — recomputed at render time from the union of stored properties.
///
/// Each label is tri-state via the paired `*_trace` field:
///   * flag `false`              → label does not apply
///   * flag `true`, trace `false`→ label applies cleanly (e.g. "Vegan")
///   * flag `true`, trace `true` → label applies only because a `~spuren-`
///     (may-contain) warning on that axis was ignored → render with an asterisk
///     (e.g. "Vegan*"), explained by a footnote.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct DietaryFlags {
    pub vegan: bool,
    pub vegetarian: bool,
    pub lactose_free: bool,
    pub gluten_free: bool,
    /// Whether the corresponding flag holds only because may-contain traces on
    /// that axis were ignored (→ show the label with an asterisk).
    pub vegan_trace: bool,
    pub vegetarian_trace: bool,
    pub lactose_free_trace: bool,
    pub gluten_free_trace: bool,
}
