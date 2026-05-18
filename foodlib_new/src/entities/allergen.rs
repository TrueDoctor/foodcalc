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
}

impl PropertySource {
    /// The stored property name applies this source's prefix convention.
    /// `Contains`/`Category` use the canonical name directly; `MayContain` prepends `~spuren-`.
    pub fn stored_name(&self, canonical: &str) -> String {
        match self {
            Self::Category | Self::Contains => canonical.to_string(),
            Self::MayContain => format!("~spuren-{canonical}"),
        }
    }
}

/// The derived dietary predicates computed over an aggregated property set.
///
/// Never stored — recomputed at render time from the union of stored properties.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct DietaryFlags {
    pub vegan: bool,
    pub vegetarian: bool,
    pub lactose_free: bool,
    pub gluten_free: bool,
}
