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

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    #[serde(rename = "bettyArticleId")]
    pub betty_article_id: ArticleId,
    pub country: String,
    pub locale: String,
    pub variants: HashMap<String, Variant>,
    #[serde(rename = "variantSelector")]
    pub variant_selector: HashMap<String, String>,
    #[serde(rename = "brandName")]
    pub brand_name: String,
    #[serde(rename = "anonymousVisible")]
    pub anonymous_visible: bool,
    #[serde(rename = "anonymousSearchable")]
    pub anonymous_searchable: bool,
    #[serde(rename = "foodNonFood")]
    pub food_non_food: bool,
    #[serde(rename = "articleId")]
    pub article_id: ArticleId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleId {
    #[serde(rename = "bettyArticleId")]
    pub betty_article_id: String,
    #[serde(rename = "articleNumber")]
    pub article_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Variant {
    #[serde(rename = "bettyVariantId")]
    pub betty_variant_id: VariantId,
    pub description: String,
    pub group: Group,
    #[serde(rename = "categoryIds")]
    pub category_ids: Vec<Id>,
    pub categories: Vec<Category>,
    pub bundles: HashMap<String, Bundle>,
    pub availability: String,
    #[serde(rename = "bundleSelector")]
    pub bundle_selector: HashMap<String, String>,
    #[serde(rename = "imageUrl")]
    pub image_url: String,
    #[serde(rename = "imageUrlS")]
    pub image_url_s: String,
    #[serde(rename = "imageUrlL")]
    pub image_url_l: String,
    pub successors: Vec<Option<serde_json::Value>>,
    #[serde(rename = "anonymousSearchable")]
    pub anonymous_searchable: bool,
    #[serde(rename = "anonymousVisible")]
    pub anonymous_visible: bool,
    #[serde(rename = "variantId")]
    pub variant_id: VariantId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VariantId {
    #[serde(rename = "articleNumber")]
    pub article_number: String,
    #[serde(rename = "variantNumber")]
    pub variant_number: String,
    #[serde(rename = "bettyVariantId")]
    pub betty_variant_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub details: Details,
    #[serde(rename = "bundleId")]
    pub bundle_id: BundleId,
    pub group: Group,
    pub categories: Vec<Category>,
    pub availability: String,
    pub stores: HashMap<String, Store>,
    #[serde(rename = "displayId")]
    pub display_id: String,
    #[serde(rename = "customerDisplayId")]
    pub customer_display_id: String,
    #[serde(rename = "priceRequestIds")]
    pub price_request_ids: Vec<Id>,
    #[serde(rename = "eanNumber")]
    pub ean_number: Vec<Option<serde_json::Value>>,
    pub description: String,
    #[serde(rename = "variantText")]
    pub variant_text: String,
    #[serde(rename = "bundleSize")]
    pub bundle_size: String,
    #[serde(rename = "isWeightArticle")]
    pub is_weight_article: String,
    #[serde(rename = "brandName")]
    pub brand_name: String,
    #[serde(rename = "ownBrand")]
    pub own_brand: Option<String>,
    #[serde(rename = "emptiesArticleNumber")]
    pub empties_article_number: Option<serde_json::Value>,
    #[serde(rename = "bundleVolume")]
    pub bundle_volume: String,
    #[serde(rename = "bundleDepth")]
    pub bundle_depth: String,
    #[serde(rename = "bundleHeight")]
    pub bundle_height: String,
    #[serde(rename = "bundleWidth")]
    pub bundle_width: String,
    #[serde(rename = "grossWeight")]
    pub gross_weight: String,
    #[serde(rename = "bundleDepthMeasureUnit")]
    pub bundle_depth_measure_unit: String,
    #[serde(rename = "bundleHeightMeasureUnit")]
    pub bundle_height_measure_unit: String,
    #[serde(rename = "bundleWidthMeasureUnit")]
    pub bundle_width_measure_unit: String,
    #[serde(rename = "warrantyMonths")]
    pub warranty_months: String,
    pub season: String,
    #[serde(rename = "imageUrl")]
    pub image_url: String,
    #[serde(rename = "imageUrlS")]
    pub image_url_s: String,
    #[serde(rename = "imageUrlL")]
    pub image_url_l: String,
    #[serde(rename = "longDescription")]
    pub long_description: String,
    pub selector: Selector,
    #[serde(rename = "refBundleIds")]
    pub ref_bundle_ids: Vec<RefBundleId>,
    #[serde(rename = "hasProductPass")]
    pub has_product_pass: bool,
    #[serde(rename = "customsTariffNumber")]
    pub customs_tariff_number: Option<serde_json::Value>,
    #[serde(rename = "logisticClass")]
    pub logistic_class: Option<String>,
    #[serde(rename = "logisticInfo")]
    pub logistic_info: LogisticInfo,
    #[serde(rename = "serviceOptions")]
    pub service_options: Vec<Option<serde_json::Value>>,
    #[serde(rename = "contentData")]
    pub content_data: ContentData,
    #[serde(rename = "weightPerPiece")]
    pub weight_per_piece: Option<serde_json::Value>,
    #[serde(rename = "brandImage")]
    pub brand_image: String,
    #[serde(rename = "trackingInfo")]
    pub tracking_info: TrackingInfo,
    #[serde(rename = "useMinQuantityMultiplier")]
    pub use_min_quantity_multiplier: Option<bool>,
    #[serde(rename = "brandInfo")]
    pub brand_info: BrandInfo,
    #[serde(rename = "parentCompatibleSearchTerms")]
    pub parent_compatible_search_terms: Option<serde_json::Value>,
    pub gtins: Vec<Option<serde_json::Value>>,
    #[serde(rename = "customerAvailability")]
    pub customer_availability: String,
    #[serde(rename = "bettyBundleId")]
    pub betty_bundle_id: BundleId,
    #[serde(rename = "fallbackImageUrl")]
    pub fallback_image_url: String,
    #[serde(rename = "showCustomerAvailability")]
    pub show_customer_availability: bool,
    #[serde(rename = "anonymousVisible")]
    pub anonymous_visible: bool,
    #[serde(rename = "anonymousSearchable")]
    pub anonymous_searchable: bool,
    #[serde(rename = "referencedSubsystemNumbers")]
    pub referenced_subsystem_numbers: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleId {
    #[serde(rename = "bettyBundleId")]
    pub betty_bundle_id: String,
    #[serde(rename = "variantNumber")]
    pub variant_number: String,
    #[serde(rename = "articleNumber")]
    pub article_number: String,
    #[serde(rename = "bettyVariantId")]
    pub betty_variant_id: String,
    #[serde(rename = "bundleNumber")]
    pub bundle_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrandInfo {
    #[serde(rename = "ownBrand")]
    pub own_brand: bool,
    #[serde(rename = "brandImage")]
    pub brand_image: String,
    #[serde(rename = "imageInfo")]
    pub image_info: ImageInfo,
    #[serde(rename = "displayBrandName")]
    pub display_brand_name: String,
    #[serde(rename = "displaySubBrandName")]
    pub display_sub_brand_name: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageInfo {
    pub width: Option<String>,
    pub height: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub id: String,
    pub levels: Vec<Level>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Level {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentData {
    #[serde(rename = "netPieceWeight")]
    pub net_piece_weight: Net,
    #[serde(rename = "netContentVolume")]
    pub net_content_volume: Option<Net>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Net {
    pub value: i64,
    pub uom: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Details {
    pub header: Header,
    #[serde(rename = "longDescription")]
    pub long_description: String,
    pub features: Vec<Feature>,
    #[serde(rename = "mainFeatures")]
    pub main_features: Vec<Option<serde_json::Value>>,
    #[serde(rename = "nutritionalTable")]
    pub nutritional_table: Table,
    #[serde(rename = "characteristicsTable")]
    pub characteristics_table: Table,
    pub media: Media,
    #[serde(rename = "energyEfficiencyInfo")]
    pub energy_efficiency_info: EnergyEfficiencyInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Table {
    #[serde(rename = "columnLabels")]
    pub column_labels: Vec<String>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Row {
    #[serde(rename = "rowLabel")]
    pub row_label: String,
    pub cells: Vec<Cell>,
    #[serde(rename = "sortId")]
    pub sort_id: i64,
    #[serde(rename = "metaInfo")]
    pub meta_info: RowMetaInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cell {
    pub value: String,
    #[serde(rename = "unitOfMeasure")]
    pub unit_of_measure: UnitOfMeasure,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnergyEfficiencyInfo {
    pub directive: Option<serde_json::Value>,
    #[serde(rename = "directiveLabel")]
    pub directive_label: Option<serde_json::Value>,
    #[serde(rename = "efficiencyClass")]
    pub efficiency_class: Option<serde_json::Value>,
    #[serde(rename = "efficiencyClassLabel")]
    pub efficiency_class_label: Option<serde_json::Value>,
    #[serde(rename = "labelImageUrl")]
    pub label_image_url: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feature {
    pub label: String,
    #[serde(rename = "sortId")]
    pub sort_id: i64,
    pub unit: String,
    pub value: String,
    #[serde(rename = "valueType")]
    pub value_type: ValueType,
    #[serde(rename = "featureType")]
    pub feature_type: String,
    #[serde(rename = "metaInfo")]
    pub meta_info: FeatureMetaInfo,
    pub leafs: Vec<Feature>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    #[serde(rename = "miscNameWebshop")]
    pub misc_name_webshop: String,
    #[serde(rename = "miscSummary")]
    pub misc_summary: String,
    #[serde(rename = "miscMainFeatures")]
    pub misc_main_features: Vec<Option<serde_json::Value>>,
    #[serde(rename = "miscSearchKeywords")]
    pub misc_search_keywords: Vec<String>,
    #[serde(rename = "miscPictureURL")]
    pub misc_picture_url: String,
    #[serde(rename = "miscPdfUrl")]
    pub misc_pdf_url: String,
    #[serde(rename = "miscAlternativeArticle")]
    pub misc_alternative_article: String,
    #[serde(rename = "miscArticleDetails")]
    pub misc_article_details: String,
    #[serde(rename = "miscBrandName")]
    pub misc_brand_name: String,
    #[serde(rename = "miscComText2")]
    pub misc_com_text2: String,
    #[serde(rename = "miscComTextWebshop")]
    pub misc_com_text_webshop: String,
    #[serde(rename = "miscCrosssellingArticles")]
    pub misc_crossselling_articles: String,
    #[serde(rename = "miscErpVariantDescription")]
    pub misc_erp_variant_description: String,
    #[serde(rename = "miscMetroExclRecommendation")]
    pub misc_metro_excl_recommendation: String,
    #[serde(rename = "miscGuaranteeID")]
    pub misc_guarantee_id: String,
    #[serde(rename = "miscGuaranteeText")]
    pub misc_guarantee_text: String,
    #[serde(rename = "miscName")]
    pub misc_name: String,
    #[serde(rename = "miscQuantumID")]
    pub misc_quantum_id: String,
    #[serde(rename = "miscQuantumText")]
    pub misc_quantum_text: String,
    #[serde(rename = "miscShortText2")]
    pub misc_short_text2: String,
    #[serde(rename = "miscShortTextWebshop")]
    pub misc_short_text_webshop: String,
    #[serde(rename = "miscSeoMetaDesc")]
    pub misc_seo_meta_desc: String,
    #[serde(rename = "miscSeoMetaTitle")]
    pub misc_seo_meta_title: String,
    #[serde(rename = "miscOwnBrand")]
    pub misc_own_brand: String,
    #[serde(rename = "miscUpselling")]
    pub misc_upselling: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Media {
    pub images: Vec<Image>,
    pub videos: Vec<Option<serde_json::Value>>,
    pub documents: Vec<Document>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub url: String,
    #[serde(rename = "sequenceNumber")]
    pub sequence_number: i64,
    pub title: String,
    #[serde(rename = "assetTypes")]
    pub asset_types: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "sequenceNumber")]
    pub sequence_number: i64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Group {
    #[serde(rename = "mainGroupName")]
    pub main_group_name: String,
    #[serde(rename = "groupName")]
    pub group_name: String,
    #[serde(rename = "subGroupName")]
    pub sub_group_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogisticInfo {
    pub options: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Id {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefBundleId {
    #[serde(rename = "articleNumber")]
    pub article_number: String,
    #[serde(rename = "variantNumber")]
    pub variant_number: String,
    #[serde(rename = "bundleNumber")]
    pub bundle_number: String,
    #[serde(rename = "subsystemArticleNumber")]
    pub subsystem_article_number: String,
    #[serde(rename = "bundleId")]
    pub bundle_id: String,
    #[serde(rename = "paddedVariantNumberAsString")]
    pub padded_variant_number_as_string: String,
    #[serde(rename = "paddedBundleNumberAsString")]
    pub padded_bundle_number_as_string: String,
    #[serde(rename = "mgbBundleNumber")]
    pub mgb_bundle_number: String,
    #[serde(rename = "variantId")]
    pub variant_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Selector {
    #[serde(rename = "packagingType")]
    pub packaging_type: PackagingType,
    #[serde(rename = "contentSize")]
    pub content_size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackagingType {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    pub country: String,
    #[serde(rename = "storeId")]
    pub store_id: String,
    pub supplier: Supplier,
    #[serde(rename = "sellingPriceInfo")]
    pub selling_price_info: SellingPriceInfo,
    #[serde(rename = "possibleDeliveryModes")]
    pub possible_delivery_modes: PossibleDeliveryModes,
    #[serde(rename = "selectedDeliveryMode")]
    pub selected_delivery_mode: String,
    #[serde(rename = "selectedFulfillmentType")]
    pub selected_fulfillment_type: String,
    #[serde(rename = "selectedFTEvaluatePromotions")]
    pub selected_ft_evaluate_promotions: Vec<Option<serde_json::Value>>,
    #[serde(rename = "storeLevelFlags")]
    pub store_level_flags: Vec<Option<serde_json::Value>>,
    #[serde(rename = "anonymousVisible")]
    pub anonymous_visible: bool,
    #[serde(rename = "anonymousSearchable")]
    pub anonymous_searchable: bool,
    pub delisted: bool,
    #[serde(rename = "customerBuyable")]
    pub customer_buyable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PossibleDeliveryModes {
    #[serde(rename = "STORE")]
    pub store: PossibleDeliveryModesStore,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PossibleDeliveryModesStore {
    #[serde(rename = "possibleFulfillmentTypes")]
    pub possible_fulfillment_types: PossibleFulfillmentTypes,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PossibleFulfillmentTypes {
    #[serde(rename = "STORE")]
    pub store: PossibleFulfillmentTypesStore,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PossibleFulfillmentTypesStore {
    #[serde(rename = "sellingPriceInfo")]
    pub selling_price_info: SellingPriceInfo,
    #[serde(rename = "relativeLeadtime")]
    pub relative_leadtime: Option<serde_json::Value>,
    #[serde(rename = "anonymousFlags")]
    pub anonymous_flags: AnonymousFlags,
    #[serde(rename = "fulfillmentStoreId")]
    pub fulfillment_store_id: String,
    pub availability: String,
    #[serde(rename = "evaluatePromotions")]
    pub evaluate_promotions: Vec<Option<serde_json::Value>>,
    pub priority: i64,
    #[serde(rename = "stockMustBeChecked")]
    pub stock_must_be_checked: bool,
    #[serde(rename = "serviceOptionIDs")]
    pub service_option_i_ds: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnonymousFlags {
    pub buyable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SellingPriceInfo {
    #[serde(rename = "customerIndividualPrice")]
    pub customer_individual_price: Option<serde_json::Value>,
    #[serde(rename = "finalPrice")]
    pub final_price: f64,
    #[serde(rename = "shelfPrice")]
    pub shelf_price: f64,
    #[serde(rename = "basePrice")]
    pub base_price: f64,
    #[serde(rename = "listNetPrice")]
    pub list_net_price: f64,
    #[serde(rename = "listGrossPrice")]
    pub list_gross_price: f64,
    #[serde(rename = "deliveryPrice")]
    pub delivery_price: f64,
    #[serde(rename = "kgNet")]
    pub kg_net: Option<serde_json::Value>,
    #[serde(rename = "kgGross")]
    pub kg_gross: Option<serde_json::Value>,
    pub explanation: String,
    pub vat: f64,
    pub valid: bool,
    #[serde(rename = "priceHiddenReason")]
    pub price_hidden_reason: Option<serde_json::Value>,
    #[serde(rename = "netPrice")]
    pub net_price: f64,
    pub currency: Currency,
    #[serde(rename = "vatPercent")]
    pub vat_percent: f64,
    #[serde(rename = "dnrInfo")]
    pub dnr_info: PackagingType,
    #[serde(rename = "grossPrice")]
    pub gross_price: f64,
    #[serde(rename = "emptiesInfo")]
    pub empties_info: EmptiesInfo,
    #[serde(rename = "finalPricesInfo")]
    pub final_prices_info: HashMap<String, Option<f64>>,
    #[serde(rename = "strikeThrough")]
    pub strike_through: Option<serde_json::Value>,
    #[serde(rename = "summaryDnrInfo")]
    pub summary_dnr_info: Option<serde_json::Value>,
    #[serde(rename = "appliedAdjustments")]
    pub applied_adjustments: Vec<AppliedAdjustment>,
    #[serde(rename = "applicablePromos")]
    pub applicable_promos: Vec<Option<serde_json::Value>>,
    #[serde(rename = "promotionLabels")]
    pub promotion_labels: PackagingType,
    #[serde(rename = "appliedPromotionInfo")]
    pub applied_promotion_info: AppliedPromotionInfo,
    #[serde(rename = "taxGroups")]
    pub tax_groups: Vec<Option<serde_json::Value>>,
    #[serde(rename = "grossStrikeThrough")]
    pub gross_strike_through: Option<serde_json::Value>,
    #[serde(rename = "priceType")]
    pub price_type: String,
    #[serde(rename = "serviceFees")]
    pub service_fees: PackagingType,
    #[serde(rename = "averagePerUnit")]
    pub average_per_unit: Option<serde_json::Value>,
    #[serde(rename = "isMRPApplied")]
    pub is_mrp_applied: Option<serde_json::Value>,
    #[serde(rename = "MRPrice")]
    pub mr_price: Option<serde_json::Value>,
    #[serde(rename = "basePriceData")]
    pub base_price_data: Option<serde_json::Value>,
    #[serde(rename = "availablePromotions")]
    pub available_promotions: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppliedPromotionInfo {
    pub name: String,
    pub price: Option<serde_json::Value>,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptiesInfo {
    #[serde(rename = "emptiesArticleId")]
    pub empties_article_id: Option<serde_json::Value>,
    #[serde(rename = "emptiesVat")]
    pub empties_vat: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Supplier {
    #[serde(rename = "supplierName")]
    pub supplier_name: String,
    #[serde(rename = "supplierNumber")]
    pub supplier_number: String,
    #[serde(rename = "articleNumber")]
    pub article_number: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackingInfo {
    #[serde(rename = "trackingId")]
    pub tracking_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UnitOfMeasure {
    #[serde(rename = "")]
    Empty,
    #[serde(rename = "g")]
    G,
    #[serde(rename = "kJ")]
    KJ,
    #[serde(rename = "kcal")]
    Kcal,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RowMetaInfo {
    #[serde(rename = "")]
    Empty,
    Header,
    Headline,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FeatureMetaInfo {
    Contains,
    #[serde(rename = "")]
    Empty,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ValueType {
    #[serde(rename = "TEXT")]
    Text,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AppliedAdjustment {
    #[serde(rename = "shelf")]
    Shelf,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Currency {
    #[serde(rename = "EUR")]
    Eur,
}
