use std::collections::BTreeMap;
use std::fs;

use metro_scrape::article::FeatureMetaInfo;
use metro_scrape::request::fetch_articles_from_urls;

fn walk_features<'a>(
    features: &'a [metro_scrape::article::Feature],
    out: &mut Vec<&'a metro_scrape::article::Feature>,
) {
    for f in features {
        out.push(f);
        walk_features(&f.leafs, out);
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/metro_urls.tsv".to_string());
    let contents = fs::read_to_string(&path)?;

    let urls: Vec<(i32, String)> = contents
        .lines()
        .filter_map(|line| {
            let (id, url) = line.split_once('\t')?;
            Some((id.parse().ok()?, url.to_string()))
        })
        .collect();

    eprintln!("Fetching {} articles...", urls.len());
    let (articles, failures) = fetch_articles_from_urls(urls).await;
    eprintln!("Got {} articles back.", articles.len());
    for f in &failures {
        eprintln!("fetch failure: {f}");
    }

    // (meta_info_variant_name, label) -> count
    let mut histogram: BTreeMap<(String, String), usize> = BTreeMap::new();
    // Track per-ingredient distinct labels for spot-checking
    let mut per_ingredient: BTreeMap<i32, Vec<(String, String, String)>> = BTreeMap::new();

    for (ingredient_id, article) in &articles {
        for variant in article.variants.values() {
            for bundle in variant.bundles.values() {
                let mut feats = Vec::new();
                walk_features(&bundle.details.features, &mut feats);
                for f in feats {
                    let kind = format!("{:?}", f.meta_info);
                    // Skip noise meta_info variants
                    if matches!(
                        f.meta_info,
                        FeatureMetaInfo::Empty
                            | FeatureMetaInfo::Header
                            | FeatureMetaInfo::SubListStart
                            | FeatureMetaInfo::SubListEnd
                            | FeatureMetaInfo::Annotations
                    ) {
                        continue;
                    }
                    *histogram
                        .entry((kind.clone(), f.label.clone()))
                        .or_default() += 1;
                    per_ingredient.entry(*ingredient_id).or_default().push((
                        kind,
                        f.label.clone(),
                        f.value.clone(),
                    ));
                }
            }
        }
    }

    println!("\n=== Histogram of (meta_info, label) across all ingredients ===\n");
    // Group by meta_info kind for readability
    let mut by_kind: BTreeMap<String, Vec<(String, usize)>> = BTreeMap::new();
    for ((kind, label), count) in &histogram {
        by_kind
            .entry(kind.clone())
            .or_default()
            .push((label.clone(), *count));
    }
    for (kind, mut entries) in by_kind {
        entries.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        println!("--- {} ({} distinct labels) ---", kind, entries.len());
        for (label, count) in entries {
            println!("  {:>4}  {}", count, label);
        }
        println!();
    }

    println!("\n=== Sample: first 5 ingredients with their tags ===\n");
    for (ing_id, tags) in per_ingredient.iter().take(5) {
        println!("ingredient_id={}:", ing_id);
        for (kind, label, value) in tags {
            println!("  [{}] {} = {}", kind, label, value);
        }
        println!();
    }

    Ok(())
}
