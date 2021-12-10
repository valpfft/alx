use crate::parse_price;
use crate::Offer;
use select::document::Document;
use select::predicate::And;
use select::predicate::Not;
use select::predicate::{Attr, Class, Name};

use std::collections::HashMap;
use std::num::ParseIntError;

static BASE_URL: &str = "https://allegrolokalnie.pl";
static OFFERS_PATH: &str = "oferty";

pub fn scrape(params: &HashMap<&str, &str>) -> Vec<Offer> {
    let mut collection = Vec::new();
    let url = build_url(params);
    let response = reqwest::get(&url).expect("Could not get url");
    assert!(response.status().is_success());

    let mut first_page = Document::from_read(response).expect("Could not parse first page");

    // scrape first page
    first_page = parse_page(first_page, &mut collection);

    if let Some(pager) = first_page.find(Class("pagination")).next() {
        let with_pagination = || -> Result<u32, ParseIntError> {
            pager
                .find(Name("label"))
                .next()
                .unwrap()
                .find(And(Name("span"), Not(Class("sr-only"))))
                .next()
                .unwrap()
                .text()
                .parse::<u32>()
        };

        // Parse rest of pages
        if let Ok(n) = with_pagination() {
            if n > 1 {
                for page_number in 2..=n {
                    let u = format!("{}/?page={}", &url, page_number);
                    let resp = reqwest::get(&u).expect("Could not get url");
                    assert!(resp.status().is_success());

                    let doc = Document::from_read(resp).unwrap();
                    parse_page(doc, &mut collection);
                }
            }
        }
    };

    collection
}

fn parse_page(doc: Document, result: &mut Vec<Offer>) -> Document {
    for entry in doc.find(Class("offer-card")) {
        result.push(Offer {
            title: entry
                .find(And(Name("h3"), Class("offer-card__title")))
                .next()
                .unwrap()
                .text(),
            price: parse_price(&entry.find(Attr("itemprop", "price")).next().unwrap().text())
                .unwrap(),
            url: format!(
                "{}{}",
                BASE_URL,
                entry
                    .attr("href")
                    .expect("Cannot parse offer href")
                    .to_string(),
            ),
        })
    }

    doc
}

fn build_url(params: &HashMap<&str, &str>) -> String {
    let mut url = format!(
        "{}/{}/q/{}",
        BASE_URL,
        OFFERS_PATH,
        params.get("query").unwrap()
    );

    if params.contains_key("min_price") {
        add_filter(
            &format!("price_from={}", params.get("min_price").unwrap()),
            &mut url,
        );
    }

    if params.contains_key("max_price") {
        add_filter(
            &format!("price_to={}", params.get("max_price").unwrap()),
            &mut url,
        );
    }

    url
}

fn add_filter(filter: &str, url: &mut String) {
    match url.find("?") {
        Some(_) => url.push_str(&format!("&{}", filter)),
        None => url.push_str(&format!("?{}", filter)),
    };
}
