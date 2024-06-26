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

async fn fetch_articles_batched(
    articles: &[ArticleIdentifier],
) -> Result<Vec<Article>, eyre::Error> {
    let mut result = Vec::new();
    for chunk in articles.chunks(40) {
        let articles = fetch_articles(chunk).await?;
        result.extend(articles);
    }
    Ok(result)
}
pub async fn fetch_articles_from_urls<S: AsRef<str> + Hash>(
    urls: impl IntoIterator<Item = (i32, S)>,
) -> Result<Vec<(i32, Article)>, eyre::Error> {
    let article_identifiers = urls
        .into_iter()
        .filter(|(_, url)| !url.as_ref().is_empty())
        .map(|(id, url)| {
            let ident = extract_betty_identifier_from_url(url.as_ref())
                .unwrap_or_else(|| panic!("invalid url {}", url.as_ref()));
            (ident.betty_id.clone(), (ident, id))
        })
        .collect::<HashMap<_, _>>();

    let identifiers = article_identifiers
        .values()
        .map(|x| x.0.clone())
        .collect::<Vec<_>>();

    let articles = fetch_articles_batched(&identifiers).await?;

    let mut result = articles
        .iter()
        .map(|article| {
            let betty_id = &article.betty_article_id.betty_article_id;
            let ingredient_id = article_identifiers.get(betty_id).unwrap();
            (ingredient_id.1, article.clone())
        })
        .collect::<Vec<_>>();
    // sort articles by identifier
    result.sort_by_key(|article| article.0);
    Ok(result)
}
