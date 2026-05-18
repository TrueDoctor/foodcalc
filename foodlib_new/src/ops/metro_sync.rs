//! Metro Cash & Carry data sync — fetch articles from the Metro API for all
//! ingredient_sources that have a Metro URL, then update prices, category, and
//! allergen properties in one pass.
//!
//! Replaces the deprecated `foodlib::ingredients::fetch_metro_prices`.

use std::str::FromStr;
use std::sync::Arc;

use bigdecimal::{BigDecimal, FromPrimitive};
use sqlx::PgPool;

use crate::error::{Error, Result};
use crate::ops::allergens::AllergenOps;

use metro_scrape::article::Article;
use metro_scrape::request::fetch_articles_from_urls;

/// Metro is `store_id = 0` per legacy `foodlib::ingredients::METRO`.
pub const METRO_STORE_ID: i32 = 0;

#[derive(Debug, Clone, Default)]
pub struct SyncReport {
    pub urls_total: usize,
    pub articles_fetched: usize,
    pub prices_updated: usize,
    pub allergens_applied: usize,
    pub allergens_wiped: usize,
    pub failures: Vec<String>,
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

        let url_pairs: Vec<(i32, String)> = sources
            .iter()
            .map(|s| (s.ingredient_id, s.url.clone()))
            .collect();
        let articles = fetch_articles_from_urls(url_pairs)
            .await
            .map_err(|e| Error::Misc(format!("Metro fetch failed: {e}")))?;
        report.articles_fetched = articles.len();

        for (ingredient_id, article) in articles {
            // One ingredient_id can have multiple sources (rare); apply to whichever source
            // matches this article.
            let matching: Vec<&MetroSource> = sources
                .iter()
                .filter(|s| s.ingredient_id == ingredient_id)
                .collect();
            if matching.is_empty() {
                report
                    .failures
                    .push(format!("no source row for ingredient #{ingredient_id}"));
                continue;
            }

            // Allergens are keyed by ingredient, not source — apply once per ingredient.
            match self.apply_allergens(ingredient_id, &article).await {
                Ok(n) => report.allergens_applied += n,
                Err(e) => report
                    .failures
                    .push(format!("ingredient #{ingredient_id} allergens: {e}")),
            }

            for source in matching {
                match self.update_price_and_category(source, &article).await {
                    Ok(true) => report.prices_updated += 1,
                    Ok(false) => {} // no price available (unauth?), category still set
                    Err(e) => report.failures.push(format!(
                        "ingredient #{} source #{}: {e}",
                        source.ingredient_id, source.ingredient_source_id
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
                SELECT ingredient_source_id, ingredient_id, url
                FROM ingredient_sources
                WHERE store_id = $1 AND url IS NOT NULL AND url <> '' AND ingredient_id = $2
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
                url: r.url.unwrap_or_default(),
            })
            .collect(),
            None => sqlx::query!(
                r#"
                SELECT ingredient_source_id, ingredient_id, url
                FROM ingredient_sources
                WHERE store_id = $1 AND url IS NOT NULL AND url <> ''
                "#,
                METRO_STORE_ID,
            )
            .fetch_all(&*self.pool)
            .await?
            .into_iter()
            .map(|r| MetroSource {
                ingredient_source_id: r.ingredient_source_id,
                ingredient_id: r.ingredient_id,
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
            .min_by_key(|b| (f64::from_str(&b.gross_weight).unwrap_or_default() * 1000.) as u64)
            .ok_or_else(|| Error::Misc(format!("no bundle for ingredient #{}", source.ingredient_id)))?;

        let category = bundle
            .categories
            .iter()
            .fold(String::new(), |acc, n| format!("{acc}/{}", n.name));
        sqlx::query!(
            r#"
            INSERT INTO metro_categories (ingredient_source_id, category)
            VALUES ($1, $2)
            ON CONFLICT (ingredient_source_id) DO UPDATE SET category = EXCLUDED.category
            "#,
            source.ingredient_source_id,
            category,
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

        let weight = BigDecimal::from_str(&bundle.gross_weight)
            .map_err(|e| Error::Misc(format!("invalid gross_weight `{}`: {e}", bundle.gross_weight)))?;
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
