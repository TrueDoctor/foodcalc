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
    missing: serde_json::Value,
    context: serde_json::Value,
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub struct ArticleIdentifier {
    betty_id: String,
    variant_id: String,
}

async fn fetch_articles(articles: &[ArticleIdentifier]) -> Result<Vec<Article>, eyre::Error> {
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
    Ok(result.result.values().cloned().collect())
}

/// Fetch all chunks, isolating failures: a chunk whose HTTP request or parsing
/// fails is recorded in the returned failure list and skipped, while the other
/// chunks are still fetched. Returns `(successfully fetched articles, failure
/// messages)`.
async fn fetch_articles_batched(
    articles: &[ArticleIdentifier],
) -> (Vec<Article>, Vec<String>) {
    let mut result = Vec::new();
    let mut failures = Vec::new();
    for chunk in articles.chunks(40) {
        match fetch_articles(chunk).await {
            Ok(articles) => result.extend(articles),
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
    (result, failures)
}

/// Resolve Metro articles for `(id, url)` pairs. Malformed URLs and failed
/// fetch batches are reported in the returned failure list rather than aborting
/// the whole call; every URL that can be fetched still is. Returns `(resolved
/// (id, article) pairs, failure messages)`.
pub async fn fetch_articles_from_urls<S: AsRef<str> + Hash>(
    urls: impl IntoIterator<Item = (i32, S)>,
) -> (Vec<(i32, Article)>, Vec<String>) {
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

    let (articles, fetch_failures) = fetch_articles_batched(&identifiers).await;
    failures.extend(fetch_failures);

    let mut result = articles
        .iter()
        .filter_map(|article| {
            let betty_id = &article.betty_article_id.betty_article_id;
            article_identifiers
                .get(betty_id)
                .map(|ingredient_id| (ingredient_id.1, article.clone()))
        })
        .collect::<Vec<_>>();
    // sort articles by identifier
    result.sort_by_key(|article| article.0);
    (result, failures)
}
