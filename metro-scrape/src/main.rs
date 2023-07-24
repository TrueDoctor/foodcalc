use metro_scrape::request::fetch_articles_from_urls;

#[tokio::main]
async fn main() {
    let result = fetch_articles_from_urls([(0, "https://produkte.metro.de/shop/pv/BTY-X208528/0032/0021/aro-H-Milch-3-5-Fett-5-00-l-Karton")]).await;
    println!("{:#?}", result);
}
