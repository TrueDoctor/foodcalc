use std::time::Duration;

use inline_python::{python, Context};
use sqlx::postgres::types::PgMoney;
use sqlx::types::BigDecimal;

lazy_static::lazy_static! {
    static ref CONTEXT : inline_python::Context = {

     let c =    Context::new();
        c.run(python! {
            from selenium import webdriver
            from selenium.webdriver.common.by import By
            from selenium.webdriver.common.keys import Keys
            import time

            browser = webdriver.Chrome()
        });
    c
    };
}

pub(crate) fn fetch_metro_price_python(url: &str) -> Option<PgMoney> {
    use num::Num;
    log::info!("fetching price for {url}");
    CONTEXT.run(python! {
        browser.get('url)

    });

    fn find_price() -> String {
        CONTEXT.run(python! {
            try:
                elem = browser.find_element_by_css_selector(".mfcss_article-detail--price-breakdown")
                elem = elem.text
            except:
                elem = ""
        });

        CONTEXT.get::<String>("elem")
    }

    for _ in 0..40 {
        std::thread::sleep(Duration::from_millis(500));
        log::info!("waited for ones sec");
        match find_price().as_str() {
            "" => continue,
            price => {
                use regex::Regex;
                log::info!("got price {price}");
                let number_regex =
                    Regex::new(r"[0-9][0-9,]*").expect("failed to compile number regex");

                if let Some(number) = number_regex.find(price) {
                    log::info!("regex {}", number.as_str());
                    let number = number
                        .as_str()
                        .to_owned()
                        .replace('.', "")
                        .replace(',', ".");
                    log::info!("fetched price {number}");
                    return PgMoney::from_bigdecimal(
                        BigDecimal::from_str_radix(number.as_str(), 10).unwrap(),
                        2,
                    )
                    .ok();
                }
            }
        }
    }
    log::error!("failed to fetch price");
    None
}
