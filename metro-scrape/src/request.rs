use std::collections::HashMap;

use super::article::*;
use regex::Regex;
use reqwest::Method;
use serde::{Deserialize, Serialize};

fn extract_betty_identifier_from_url(url: &str) -> Option<ArticleIdentifier> {
    let re = Regex::new(r#"(BTY-[^/]+)/(\d+)"#).expect("regex compilation failed");
    let caps = re.captures(url)?;
    Some(ArticleIdentifier {
        betty_id: caps.get(1)?.as_str().to_string(),
        bundle_id: caps.get(2)?.as_str().to_string(),
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    result: HashMap<String, Article>,
    missing: serde_json::Value,
    context: serde_json::Value,
}

#[derive(Debug)]
pub struct ArticleIdentifier {
    betty_id: String,
    bundle_id: String,
}

async fn fetch_articles(articles: &[ArticleIdentifier]) -> Result<Vec<Article>, eyre::Error> {
    dbg!(articles);
    let article_parameters = articles.iter().fold(String::new(), |acc, article| {
        format!("{acc}&ids={}", article.betty_id)
    });

    let url = format!("https://produkte.metro.de/evaluate.article.v1/betty-articles?country=DE&locale=de-DE&storeIds=00062&details=true{article_parameters}");

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
pub async fn fetch_articles_from_urls(urls: &[&str]) -> Result<Vec<Article>, eyre::Error> {
    let articles = urls
        .iter()
        .filter_map(|url| extract_betty_identifier_from_url(url))
        .collect::<Vec<_>>();

    fetch_articles_batched(&articles).await
}
