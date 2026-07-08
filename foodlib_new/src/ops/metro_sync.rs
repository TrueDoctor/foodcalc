//! Metro Cash & Carry data sync — fetch articles from the Metro API for all
//! ingredient_sources that have a Metro URL, then update prices, category, and
//! allergen properties in one pass.
//!
//! Replaces the deprecated `foodlib::ingredients::fetch_metro_prices`.

use std::str::FromStr;
use std::sync::Arc;

use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use sqlx::PgPool;

use crate::error::{Error, Result};
use crate::ops::allergens::AllergenOps;

use metro_scrape::article::{Article, Bundle};
use metro_scrape::request::fetch_articles_from_urls;

/// Metro is `store_id = 0` per legacy `foodlib::ingredients::METRO`.
pub const METRO_STORE_ID: i32 = 0;

/// Package weight of a bundle in kilograms.
///
/// Prefers the declared net piece weight (`content_data.net_piece_weight`) when
/// present and its unit is recognised, falling back to `gross_weight` (already
/// in kg) otherwise. An unrecognised net unit is treated as "no usable net" and
/// also falls back to gross, so an unexpected uom can never store a value off by
/// a factor of 1000.
fn bundle_weight_kg(bundle: &Bundle) -> Option<BigDecimal> {
    let net_kg = bundle.content_data.net_piece_weight.as_ref().and_then(|net| {
        let value = BigDecimal::from_i64(net.value)?;
        // Metro reports the unit as a full uppercase word (e.g. "GRAM"). Accept
        // the abbreviated forms too, defensively.
        match net.uom.to_ascii_uppercase().as_str() {
            "GRAM" | "G" | "GRM" => Some(value / 1000),
            "KILOGRAM" | "KG" | "KGM" => Some(value),
            _ => None,
        }
    });
    net_kg.or_else(|| BigDecimal::from_str(&bundle.gross_weight).ok())
}

#[derive(Debug, Clone, Default)]
pub struct SyncReport {
    pub urls_total: usize,
    pub articles_fetched: usize,
    pub prices_updated: usize,
    pub allergens_applied: usize,
    pub allergens_wiped: usize,
    pub failures: Vec<String>,
    /// Non-fatal anomalies worth a human's attention, e.g. an article that
    /// declares allergen data but yielded no classified properties.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SyncOptions {
    /// Before applying new classifications, delete all existing
    /// `ingredient_properties` rows for ingredients that have a Metro URL.
    /// This is the "fresh state" mode used during classifier development.
    pub wipe_existing: bool,
}

#[derive(Clone)]
pub struct MetroSyncOps {
    pool: Arc<PgPool>,
    allergens: AllergenOps,
}

#[derive(Debug, Clone)]
struct MetroSource {
    ingredient_source_id: i32,
    ingredient_id: i32,
    ingredient_name: String,
    url: String,
}

impl MetroSyncOps {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            allergens: AllergenOps::new(pool.clone()),
            pool,
        }
    }

    /// Run a full sync. If `ingredient_id` is `Some`, only sync that ingredient's sources;
    /// otherwise sync every Metro-URL'd source in the database.
    ///
    /// Each per-article failure is recorded in the report and does not abort the run.
    pub async fn sync(&self, ingredient_id: Option<i32>) -> Result<SyncReport> {
        self.sync_with(ingredient_id, SyncOptions::default()).await
    }

    /// Same as `sync` but with explicit options (e.g. `wipe_existing`).
    pub async fn sync_with(
        &self,
        ingredient_id: Option<i32>,
        opts: SyncOptions,
    ) -> Result<SyncReport> {
        let sources = self.load_metro_sources(ingredient_id).await?;
        let mut report = SyncReport {
            urls_total: sources.len(),
            ..Default::default()
        };

        if sources.is_empty() {
            return Ok(report);
        }

        if opts.wipe_existing {
            report.allergens_wiped = self.wipe_metro_ingredient_properties(&sources).await?;
        }

        // Ingredient names for human-readable report messages (ids alone aren't
        // recognizable in the UI). One name per ingredient id.
        let ingredient_names: std::collections::HashMap<i32, String> = sources
            .iter()
            .map(|s| (s.ingredient_id, s.ingredient_name.clone()))
            .collect();
        // Label an ingredient by name (falling back to id if somehow unknown).
        let label = |id: i32| match ingredient_names.get(&id) {
            Some(name) => format!("{name} (#{id})"),
            None => format!("#{id}"),
        };

        let url_pairs: Vec<(i32, String)> = sources
            .iter()
            .map(|s| (s.ingredient_id, s.url.clone()))
            .collect();
        // Per-URL/batch fetch failures are reported but do not abort the sync —
        // every article that can be fetched is still applied.
        let fetched = fetch_articles_from_urls(url_pairs).await;
        report.failures.extend(fetched.failures);
        report.articles_fetched = fetched.articles.len();

        // Ids Metro could not resolve (delisted / NOT_FOUND) — surface as failures
        // so a stale source URL is visible rather than silently leaving the
        // ingredient with no data.
        for m in &fetched.missing {
            let who = match m.ingredient_id {
                Some(id) => label(id),
                None => format!("Metro article {}", m.betty_id),
            };
            let reason = if m.reasons.is_empty() {
                "not available".to_string()
            } else {
                m.reasons.join(", ")
            };
            report.failures.push(format!(
                "{who}: Metro article {} not available ({reason})",
                m.betty_id
            ));
        }

        for (ingredient_id, article) in fetched.articles {
            // One ingredient_id can have multiple sources (rare); apply to whichever source
            // matches this article.
            let matching: Vec<&MetroSource> = sources
                .iter()
                .filter(|s| s.ingredient_id == ingredient_id)
                .collect();
            if matching.is_empty() {
                report
                    .failures
                    .push(format!("no source row for ingredient {}", label(ingredient_id)));
                continue;
            }

            // Allergens are keyed by ingredient, not source — apply once per ingredient.
            match self.apply_allergens(ingredient_id, &article).await {
                Ok(n) => {
                    report.allergens_applied += n;
                    // The article declared allergen data but we extracted nothing —
                    // likely an unrecognized label. Flag it rather than silently
                    // leaving the ingredient with no properties (which would, e.g.,
                    // make a wheat product read as gluten-free).
                    if n == 0 && AllergenOps::article_has_allergen_features(&article) {
                        report.warnings.push(format!(
                            "{}: article declares allergen data but no properties were classified",
                            label(ingredient_id)
                        ));
                    }
                }
                Err(e) => report
                    .failures
                    .push(format!("{} allergens: {e}", label(ingredient_id))),
            }

            for source in matching {
                match self.update_price_and_category(source, &article).await {
                    Ok(true) => report.prices_updated += 1,
                    Ok(false) => {} // no price available (unauth?), category still set
                    Err(e) => report.failures.push(format!(
                        "{} source #{}: {e}",
                        label(source.ingredient_id),
                        source.ingredient_source_id
                    )),
                }
            }
        }

        Ok(report)
    }

    /// Delete every `ingredient_properties` row for ingredients that are sourced
    /// from Metro. Returns the count of deleted rows. Manually-curated tags on
    /// non-Metro ingredients (e.g. items purchased only at Edna or IKEA) are
    /// preserved.
    async fn wipe_metro_ingredient_properties(
        &self,
        sources: &[MetroSource],
    ) -> Result<usize> {
        let ingredient_ids: Vec<i32> = sources.iter().map(|s| s.ingredient_id).collect();
        let rec = sqlx::query!(
            r#"DELETE FROM ingredient_properties WHERE ingredient_id = ANY($1)"#,
            &ingredient_ids,
        )
        .execute(&*self.pool)
        .await?;
        Ok(rec.rows_affected() as usize)
    }

    async fn load_metro_sources(
        &self,
        ingredient_id: Option<i32>,
    ) -> Result<Vec<MetroSource>> {
        let rows: Vec<MetroSource> = match ingredient_id {
            Some(id) => sqlx::query!(
                r#"
                SELECT s.ingredient_source_id, s.ingredient_id, i.name AS ingredient_name, s.url
                FROM ingredient_sources s
                JOIN ingredients i ON i.ingredient_id = s.ingredient_id
                WHERE s.store_id = $1 AND s.url IS NOT NULL AND s.url <> '' AND s.ingredient_id = $2
                "#,
                METRO_STORE_ID,
                id,
            )
            .fetch_all(&*self.pool)
            .await?
            .into_iter()
            .map(|r| MetroSource {
                ingredient_source_id: r.ingredient_source_id,
                ingredient_id: r.ingredient_id,
                ingredient_name: r.ingredient_name,
                url: r.url.unwrap_or_default(),
            })
            .collect(),
            None => sqlx::query!(
                r#"
                SELECT s.ingredient_source_id, s.ingredient_id, i.name AS ingredient_name, s.url
                FROM ingredient_sources s
                JOIN ingredients i ON i.ingredient_id = s.ingredient_id
                WHERE s.store_id = $1 AND s.url IS NOT NULL AND s.url <> ''
                "#,
                METRO_STORE_ID,
            )
            .fetch_all(&*self.pool)
            .await?
            .into_iter()
            .map(|r| MetroSource {
                ingredient_source_id: r.ingredient_source_id,
                ingredient_id: r.ingredient_id,
                ingredient_name: r.ingredient_name,
                url: r.url.unwrap_or_default(),
            })
            .collect(),
        };
        Ok(rows.into_iter().filter(|s| !s.url.is_empty()).collect())
    }

    async fn apply_allergens(&self, ingredient_id: i32, article: &Article) -> Result<usize> {
        let classified = AllergenOps::classify_article(article);
        if classified.is_empty() {
            return Ok(0);
        }
        let n = classified.len();
        self.allergens
            .apply_to_ingredient(ingredient_id, &classified)
            .await?;
        Ok(n)
    }

    /// Update price + package_size on the source row and upsert its metro_categories row.
    /// Returns `Ok(true)` if a price was found and applied; `Ok(false)` if no price was
    /// available but the category was still updated.
    async fn update_price_and_category(
        &self,
        source: &MetroSource,
        article: &Article,
    ) -> Result<bool> {
        let variant = article
            .variants
            .values()
            .next()
            .ok_or_else(|| Error::Misc(format!("no variant for ingredient #{}", source.ingredient_id)))?;
        let bundle = variant
            .bundles
            .values()
            .min_by_key(|b| {
                // Primary key: unit-aware package weight in grams (reuses
                // `bundle_weight_kg`). No usable weight sorts last (u64::MAX).
                let net = bundle_weight_kg(b)
                    .and_then(|kg| (kg * BigDecimal::from(1000)).to_u64())
                    .unwrap_or(u64::MAX);
                // Tie-break on gross weight (unitless kg): a multi-piece case and
                // a single piece share the same net *piece* weight (e.g. 9 × 275 g
                // vs 1 × 275 g), so without this the pick — and thus the price —
                // is arbitrary. Smaller gross is the single unit, whose price
                // matches its weight.
                let gross = (f64::from_str(&b.gross_weight).unwrap_or(f64::MAX) * 1000.) as u64;
                (net, gross)
            })
            .ok_or_else(|| Error::Misc(format!("no bundle for ingredient #{}", source.ingredient_id)))?;

        let category = bundle
            .categories
            .iter()
            .fold(String::new(), |acc, n| format!("{acc}/{}", n.name));
        // Article number, price and weight must all come from the SAME (selected)
        // bundle — multi-bundle articles (e.g. a single piece vs a 9-piece case)
        // have different prices/ids for the same net piece weight, so mixing
        // bundles yields a price/weight mismatch. Read the id off `bundle`, not
        // `article` (which would pick an arbitrary bundle's id).
        let article_number = (!bundle.customer_display_id.is_empty())
            .then_some(bundle.customer_display_id.as_str());
        sqlx::query!(
            r#"
            INSERT INTO metro_categories (ingredient_source_id, category, article_number)
            VALUES ($1, $2, $3)
            ON CONFLICT (ingredient_source_id) DO UPDATE SET
                category = EXCLUDED.category,
                article_number = EXCLUDED.article_number
            "#,
            source.ingredient_source_id,
            category,
            article_number,
        )
        .execute(&*self.pool)
        .await?;

        let price_opt = bundle
            .stores
            .values()
            .next()
            .and_then(|s| s.selling_price_info.as_ref())
            .map(|info| info.gross_price);
        let Some(price) = price_opt else {
            log::warn!(
                "ingredient #{} source #{}: no price (unauthenticated session?)",
                source.ingredient_id,
                source.ingredient_source_id
            );
            return Ok(false);
        };

        let weight = bundle_weight_kg(bundle).ok_or_else(|| {
            Error::Misc(format!("invalid gross_weight `{}`", bundle.gross_weight))
        })?;
        let price_bd = BigDecimal::from_f64(price)
            .ok_or_else(|| Error::Misc(format!("price {price} not representable as BigDecimal")))?;

        sqlx::query!(
            r#"
            UPDATE ingredient_sources
            SET price = $1, package_size = $2
            WHERE ingredient_source_id = $3
            "#,
            price_bd,
            weight,
            source.ingredient_source_id,
        )
        .execute(&*self.pool)
        .await?;
        Ok(true)
    }
}
