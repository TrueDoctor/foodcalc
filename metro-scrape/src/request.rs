use std::{collections::HashMap, hash::Hash};

use super::article::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

fn extract_betty_identifier_from_url(url: &str) -> Option<ArticleIdentifier> {
    let re = Regex::new(r"(BTY-[^/]+)/(\d+)").expect("regex compilation failed");
    let caps = re.captures(url)?;
    Some(ArticleIdentifier {
        betty_id: caps.get(1)?.as_str().to_string(),
        variant_id: caps.get(2)?.as_str().to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    result: HashMap<String, Article>,
    missing: Missing,
    context: serde_json::Value,
}

/// The API's `missing` block: ids it was asked for but could not resolve. Keyed
/// by `bettyVariantId` (e.g. `BTY-X358810032`), each entry carries the reasons
/// (e.g. `["NOT_FOUND"]` for a delisted article). We only need the keys/reasons.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Missing {
    #[serde(default)]
    variants: HashMap<String, MissingEntry>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct MissingEntry {
    #[serde(default)]
    reasons: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub struct ArticleIdentifier {
    betty_id: String,
    variant_id: String,
}

/// A variant the API could not resolve. `betty_id` is the plain betty id
/// (e.g. `BTY-X35881`), extracted from the `bettyVariantId` key; `reasons` is
/// Metro's explanation (typically `["NOT_FOUND"]` for a delisted article).
#[derive(Debug, Clone)]
pub struct MissingArticle {
    pub betty_id: String,
    pub reasons: Vec<String>,
}

/// Fetched articles plus the ids the API reported as missing (unresolvable).
struct FetchResult {
    articles: Vec<Article>,
    missing: Vec<MissingArticle>,
}

async fn fetch_articles(articles: &[ArticleIdentifier]) -> Result<FetchResult, eyre::Error> {
    let article_parameters = articles.iter().fold(String::new(), |acc, article| {
        format!("{acc}&ids={}{}", article.betty_id, article.variant_id)
    });

    let url = format!("https://produkte.metro.de/evaluate.article.v1/betty-variants?storeIds=00062&country=DE&locale=de-DE&details=true{article_parameters}");

    let client = reqwest::Client::new();
    let result = client
        .get(url)
        .header("calltreeid", "42")
        .send()
        .await?
        .text()
        .await?;

    let result: Response = serde_json::from_str(&result)?;
    let missing = result
        .missing
        .variants
        .into_iter()
        .map(|(betty_variant_id, entry)| MissingArticle {
            // The map key concatenates betty id + variant id (`BTY-X358810032`).
            // Recover the betty id by matching the known ids we asked for — the
            // betty id itself contains digits, so a naive digit-trim is wrong.
            betty_id: articles
                .iter()
                .map(|a| a.betty_id.as_str())
                .find(|id| betty_variant_id.starts_with(id))
                .unwrap_or(betty_variant_id.as_str())
                .to_string(),
            reasons: entry.reasons,
        })
        .collect();
    Ok(FetchResult {
        articles: result.result.values().cloned().collect(),
        missing,
    })
}

/// Fetch all chunks, isolating failures: a chunk whose HTTP request or parsing
/// fails is recorded in the returned failure list and skipped, while the other
/// chunks are still fetched. Returns `(successfully fetched articles, failure
/// messages)`.
async fn fetch_articles_batched(
    articles: &[ArticleIdentifier],
) -> (Vec<Article>, Vec<MissingArticle>, Vec<String>) {
    let mut result = Vec::new();
    let mut missing = Vec::new();
    let mut failures = Vec::new();
    for chunk in articles.chunks(40) {
        match fetch_articles(chunk).await {
            Ok(fetched) => {
                result.extend(fetched.articles);
                missing.extend(fetched.missing);
            }
            // The API takes the whole chunk in one request, so on failure we
            // can't tell which id was at fault — report the batch's betty ids.
            Err(e) => failures.push(format!(
                "failed to fetch batch [{}]: {e}",
                chunk
                    .iter()
                    .map(|a| a.betty_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }
    (result, missing, failures)
}

/// An id the Metro API could not resolve, mapped back to the ingredient that
/// asked for it. The caller (which knows ingredient names) formats the message.
#[derive(Debug, Clone)]
pub struct UnresolvedArticle {
    /// The ingredient whose source URL produced this id, if we could match it.
    pub ingredient_id: Option<i32>,
    /// The Metro betty id (e.g. `BTY-X35881`).
    pub betty_id: String,
    /// Why it didn't resolve, e.g. `["NOT_FOUND"]` for a delisted article.
    pub reasons: Vec<String>,
}

/// Fetched articles, ids the API reported as missing (delisted / `NOT_FOUND`),
/// and other failure messages (malformed URLs, failed fetch batches).
pub struct FetchArticlesResult {
    pub articles: Vec<(i32, Article)>,
    pub missing: Vec<UnresolvedArticle>,
    pub failures: Vec<String>,
}

/// Resolve Metro articles for `(id, url)` pairs. Malformed URLs and failed fetch
/// batches are recorded in `failures`; ids the API reports as missing are
/// returned in `missing` (so the caller can name them) rather than aborting the
/// whole call. Every URL that can be fetched still is.
pub async fn fetch_articles_from_urls<S: AsRef<str> + Hash>(
    urls: impl IntoIterator<Item = (i32, S)>,
) -> FetchArticlesResult {
    let mut failures = Vec::new();
    let mut article_identifiers = HashMap::new();
    for (id, url) in urls {
        let url = url.as_ref();
        if url.is_empty() {
            continue;
        }
        match extract_betty_identifier_from_url(url) {
            Some(ident) => {
                article_identifiers.insert(ident.betty_id.clone(), (ident, id));
            }
            None => failures.push(format!("invalid metro url for ingredient #{id}: {url}")),
        }
    }

    let identifiers = article_identifiers
        .values()
        .map(|x| x.0.clone())
        .collect::<Vec<_>>();

    let (articles, missing_raw, fetch_failures) = fetch_articles_batched(&identifiers).await;
    failures.extend(fetch_failures);

    // Map each unresolved betty id back to the ingredient that asked for it, so
    // a delisted product surfaces as an actionable report instead of vanishing.
    let missing = missing_raw
        .into_iter()
        .map(|m| UnresolvedArticle {
            ingredient_id: article_identifiers.get(&m.betty_id).map(|(_, id)| *id),
            betty_id: m.betty_id,
            reasons: m.reasons,
        })
        .collect();

    let mut articles = articles
        .iter()
        .filter_map(|article| {
            let betty_id = &article.betty_article_id.betty_article_id;
            article_identifiers
                .get(betty_id)
                .map(|ingredient_id| (ingredient_id.1, article.clone()))
        })
        .collect::<Vec<_>>();
    // sort articles by identifier
    articles.sort_by_key(|article| article.0);
    FetchArticlesResult {
        articles,
        missing,
        failures,
    }
}
