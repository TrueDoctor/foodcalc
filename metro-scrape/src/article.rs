// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::[object Object];
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: [object Object] = serde_json::from_str(&json).unwrap();
// }

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Articles = Vec<Article>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub betty_article_id: ArticleId,
    pub country: String,
    pub locale: String,
    pub variants: HashMap<String, Variant>,
    pub variant_selector: HashMap<String, String>,
    pub brand_name: Option<String>,
    pub anonymous_visible: bool,
    pub anonymous_searchable: bool,
    pub food_non_food: bool,
    pub article_id: ArticleId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleId {
    pub betty_article_id: String,
    pub article_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variant {
    pub betty_variant_id: VariantId,
    pub description: String,
    pub group: Group,
    pub category_ids: Vec<Id>,
    pub categories: Vec<Category>,
    pub bundles: HashMap<String, Bundle>,
    pub availability: String,
    pub bundle_selector: HashMap<String, String>,
    pub image_url: String,
    pub image_url_s: String,
    pub image_url_l: String,
    pub successors: Vec<Option<serde_json::Value>>,
    pub anonymous_searchable: bool,
    pub anonymous_visible: bool,
    pub variant_id: VariantId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariantId {
    pub article_number: String,
    pub variant_number: String,
    pub betty_variant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    pub details: Details,
    pub bundle_id: BundleId,
    pub group: Group,
    pub categories: Vec<Category>,
    pub availability: String,
    pub stores: HashMap<String, Store>,
    pub display_id: String,
    pub customer_display_id: String,
    pub price_request_ids: Vec<Id>,
    pub ean_number: Vec<Option<serde_json::Value>>,
    pub description: String,
    pub variant_text: Option<String>,
    pub bundle_size: String,
    pub is_weight_article: String,
    pub brand_name: Option<String>,
    pub own_brand: Option<String>,
    pub empties_article_number: Option<serde_json::Value>,
    pub bundle_volume: String,
    pub bundle_depth: String,
    pub bundle_height: String,
    pub bundle_width: String,
    pub gross_weight: String,
    pub bundle_depth_measure_unit: Option<UnitOfMeasure>,
    pub bundle_height_measure_unit: Option<UnitOfMeasure>,
    pub bundle_width_measure_unit: Option<UnitOfMeasure>,
    pub warranty_months: String,
    pub season: Option<String>,
    pub image_url: Option<String>,
    pub image_url_s: Option<String>,
    pub image_url_l: Option<String>,
    pub long_description: String,
    pub selector: Selector,
    pub ref_bundle_ids: Vec<RefBundleId>,
    pub has_product_pass: bool,
    pub customs_tariff_number: Option<serde_json::Value>,
    pub logistic_class: Option<String>,
    pub logistic_info: LogisticInfo,
    pub service_options: Vec<Option<serde_json::Value>>,
    pub content_data: ContentData,
    pub weight_per_piece: Option<serde_json::Value>,
    pub brand_image: Option<String>,
    pub tracking_info: TrackingInfo,
    pub use_min_quantity_multiplier: Option<bool>,
    pub brand_info: BrandInfo,
    pub parent_compatible_search_terms: Option<serde_json::Value>,
    pub gtins: Vec<Option<serde_json::Value>>,
    pub customer_availability: String,
    pub betty_bundle_id: BundleId,
    pub fallback_image_url: String,
    pub show_customer_availability: bool,
    pub anonymous_visible: bool,
    pub anonymous_searchable: bool,
    pub referenced_subsystem_numbers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleId {
    pub betty_bundle_id: String,
    pub variant_number: String,
    pub article_number: String,
    pub betty_variant_id: String,
    pub bundle_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrandInfo {
    pub own_brand: bool,
    pub brand_image: String,
    pub image_info: ImageInfo,
    pub display_brand_name: Option<String>,
    pub display_sub_brand_name: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub width: Option<String>,
    pub height: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub id: String,
    pub levels: Vec<Level>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Level {
    pub id: String,

    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentData {
    pub net_piece_weight: Option<Net>,
    pub net_content_volume: Option<Net>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Net {
    pub value: i64,
    pub uom: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Details {
    pub header: Header,
    pub long_description: String,
    pub features: Vec<Feature>,
    pub main_features: Vec<Option<serde_json::Value>>,
    pub nutritional_table: Table,
    pub characteristics_table: Table,
    pub media: Media,
    pub energy_efficiency_info: EnergyEfficiencyInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub column_labels: Vec<String>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub row_label: String,
    pub cells: Vec<Cell>,
    pub sort_id: i64,
    pub meta_info: RowMetaInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    pub value: String,
    pub unit_of_measure: UnitOfMeasure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnergyEfficiencyInfo {
    pub directive: Option<serde_json::Value>,
    pub directive_label: Option<serde_json::Value>,
    pub efficiency_class: Option<serde_json::Value>,
    pub efficiency_class_label: Option<serde_json::Value>,
    pub label_image_url: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub label: String,
    pub sort_id: i64,
    pub unit: String,
    pub value: String,
    pub value_type: ValueType,
    pub feature_type: String,

    pub meta_info: FeatureMetaInfo,
    pub leafs: Vec<Feature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub misc_name_webshop: String,
    pub misc_summary: String,
    pub misc_main_features: Vec<Option<serde_json::Value>>,
    pub misc_search_keywords: Vec<String>,
    pub misc_picture_url: Option<String>,
    pub misc_pdf_url: String,
    pub misc_alternative_article: String,
    pub misc_article_details: String,
    pub misc_brand_name: String,
    pub misc_com_text2: String,
    pub misc_com_text_webshop: String,
    pub misc_crossselling_articles: String,
    pub misc_erp_variant_description: String,
    pub misc_metro_excl_recommendation: String,
    pub misc_guarantee_id: Option<String>,
    pub misc_guarantee_text: String,
    pub misc_name: String,
    pub misc_quantum_id: Option<String>,
    pub misc_quantum_text: String,
    pub misc_short_text2: String,
    pub misc_short_text_webshop: String,
    pub misc_seo_meta_desc: String,
    pub misc_seo_meta_title: String,
    pub misc_own_brand: String,
    pub misc_upselling: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub images: Vec<Image>,
    pub videos: Vec<Option<serde_json::Value>>,
    pub documents: Vec<Document>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub url: String,
    pub sequence_number: i64,
    pub title: String,
    pub asset_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub sequence_number: Option<i64>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub main_group_name: String,
    pub group_name: String,
    pub sub_group_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticInfo {
    pub options: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Id {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefBundleId {
    pub article_number: String,
    pub variant_number: String,
    pub bundle_number: String,
    pub subsystem_article_number: String,
    pub bundle_id: String,
    pub padded_variant_number_as_string: String,
    pub padded_bundle_number_as_string: String,
    pub mgb_bundle_number: String,
    pub variant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selector {
    pub packaging_type: PackagingType,
    pub content_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagingType {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Store {
    pub country: String,
    pub store_id: String,
    pub supplier: Supplier,
    pub selling_price_info: SellingPriceInfo,
    pub possible_delivery_modes: PossibleDeliveryModes,
    pub selected_delivery_mode: String,
    pub selected_fulfillment_type: String,
    pub selected_ft_evaluate_promotions: Option<Vec<Option<serde_json::Value>>>,
    pub store_level_flags: Vec<Option<serde_json::Value>>,
    pub anonymous_visible: bool,
    pub anonymous_searchable: bool,
    pub delisted: bool,
    pub customer_buyable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PossibleDeliveryModes {
    pub store: Option<PossibleDeliveryModesStore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PossibleDeliveryModesStore {
    pub possible_fulfillment_types: PossibleFulfillmentTypes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PossibleFulfillmentTypes {
    pub store: Option<PossibleFulfillmentTypesStore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PossibleFulfillmentTypesStore {
    pub selling_price_info: SellingPriceInfo,
    pub relative_leadtime: Option<serde_json::Value>,
    pub anonymous_flags: AnonymousFlags,
    pub fulfillment_store_id: String,
    pub availability: String,
    pub evaluate_promotions: Vec<Option<serde_json::Value>>,
    pub priority: i64,
    pub stock_must_be_checked: bool,
    pub service_option_i_ds: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousFlags {
    pub buyable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SellingPriceInfo {
    pub customer_individual_price: Option<serde_json::Value>,
    pub final_price: f64,
    pub shelf_price: f64,
    pub base_price: f64,
    pub list_net_price: f64,
    pub list_gross_price: f64,
    pub delivery_price: f64,
    pub kg_net: Option<serde_json::Value>,
    pub kg_gross: Option<serde_json::Value>,
    pub explanation: String,
    pub vat: f64,
    pub valid: bool,
    pub price_hidden_reason: Option<serde_json::Value>,
    pub net_price: f64,
    pub currency: Currency,
    pub vat_percent: f64,
    pub dnr_info: PackagingType,
    pub gross_price: f64,
    pub empties_info: EmptiesInfo,
    pub final_prices_info: HashMap<String, Option<f64>>,
    pub strike_through: Option<serde_json::Value>,
    pub summary_dnr_info: Option<serde_json::Value>,
    pub applied_adjustments: Vec<AppliedAdjustment>,
    pub applicable_promos: Vec<Option<serde_json::Value>>,
    pub promotion_labels: PackagingType,
    pub applied_promotion_info: AppliedPromotionInfo,
    pub tax_groups: Vec<Option<serde_json::Value>>,
    pub gross_strike_through: Option<serde_json::Value>,
    pub price_type: String,
    pub service_fees: PackagingType,
    pub average_per_unit: Option<serde_json::Value>,
    pub is_mrp_applied: Option<serde_json::Value>,
    pub mr_price: Option<serde_json::Value>,
    pub base_price_data: Option<serde_json::Value>,
    pub available_promotions: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedPromotionInfo {
    pub name: String,
    pub price: Option<serde_json::Value>,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmptiesInfo {
    pub empties_article_id: Option<serde_json::Value>,
    pub empties_vat: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Supplier {
    pub supplier_name: String,
    pub supplier_number: String,
    pub article_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackingInfo {
    pub tracking_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnitOfMeasure {
    #[serde(rename = "")]
    Empty,
    #[serde(rename = "E14")]
    E14,
    #[serde(rename = "KJO")]
    KJO,
    #[serde(rename = "%")]
    Percent,
    #[serde(rename = "°C")]
    DegreeCelsius,
    #[serde(rename = "m")]
    Meter,
    #[serde(rename = "cm")]
    Centimeter,
    #[serde(rename = "mm")]
    Millimeter,
    #[serde(rename = "GRM")]
    GRM,
    #[serde(rename = "g")]
    Gramm,
    #[serde(rename = "µg")]
    Microgramm,
    #[serde(rename = "mg")]
    #[serde(alias = "MGM")]
    Milligramm,
    #[serde(rename = "kg")]
    Kilogramm,
    #[serde(rename = "kJ")]
    #[serde(alias = "kj")]
    KJ,
    #[serde(rename = "kcal")]
    Kcal,
    #[serde(rename = "ml")]
    Milliliter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RowMetaInfo {
    #[serde(rename = "")]
    Empty,
    Header,
    Headline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureMetaInfo {
    Contains,
    Annotations,
    SubListStart,
    SubListEnd,
    #[serde(rename = "Additive_Class")]
    AdditiveClass,
    Allergen,
    #[serde(rename = "Free from")]
    FreeFrom,
    #[serde(rename = "E_Numbers")]
    ENumbers,
    #[serde(rename = "May_Contain")]
    MayContain,
    Header,
    #[serde(rename = "")]
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValueType {
    #[serde(rename = "TEXT")]
    Text,
    #[serde(rename = "NUMBER")]
    Number,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppliedAdjustment {
    Shelf,
    Promotion,
    Dnr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Currency {
    #[serde(rename = "EUR")]
    Eur,
}
