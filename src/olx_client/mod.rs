use crate::Offer;
use crate::parse_price;
use select::document::Document;
use select::predicate::{Class, Name, Predicate, Attr};

use std::collections::HashMap;

static BASE_URL: &str = "https://www.olx.pl/oferty";

impl Offer {
    fn build_from_node(node: &select::node::Node) -> Offer {
        let title = node.find(Name("a").descendant(Name("strong"))).next().expect("Could not parse detailsLink text").text();
        let url = node.find(Name("a")).next().expect("Could not parse detailsLink link").attr("href").expect("Could not parse detailsLink href");
        let price = node.find(Class("price").descendant(Name("strong"))).next().expect("Could not parse price").text();

        Offer {
            title: title,
            price: parse_price(&price).expect("Could not parse price"),
            url: url.to_string(),
        }
    }
}

pub fn scrape(params: &HashMap<&str, &str>) -> Vec<Offer> {
    let mut collection = Vec::new();

    let url = build_url(params);

    let pages = get_all_pages(&url);

    for page in pages {
        parse_page(page, &mut collection);
    }

    collection
}


fn build_url(params: &HashMap<&str, &str>) -> String {
    let mut url = format!("{}/q-{}", BASE_URL, format_query(params.get("query").unwrap()));

    if params.contains_key("min_price") {
        add_filter(
            &format!("search[filter_float_price:from]={}", params.get("min_price").unwrap()),
            &mut url
        );
    }

    if params.contains_key("max_price") {
        add_filter(
            &format!("search[filter_float_price:to]={}", params.get("max_price").unwrap()),
            &mut url
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


fn get_all_pages(base_url: &str) -> Vec<reqwest::Response> {
    let response = reqwest::get(base_url).expect("Could not get url");
    assert!(response.status().is_success());

    let first_page = Document::from_read(response).expect("Could not parse first page");

    let pager = first_page.find(Class("pager")).next();

    let total_pages = match pager {
        Some(pager) => {
            pager
                .find(Attr("data-cy", "page-link-last").descendant(Name("span")))
                .next()
                .expect("Could not find last page")
                .text()
                .parse::<u32>()
                .expect("Could not parse last page number")
        },
        None => 1
    };

    let mut pages = Vec::new();

    for page_number in 1..=total_pages {
        let page = get_page(format!("{}/?page={}", base_url, page_number.to_string()));

        pages.push(page);
    }

    pages
}

fn parse_page(response: reqwest::Response, result: &mut Vec<Offer>) {
    let page = Document::from_read(response).expect("Could not parse page");

    for entry in page.find(Class("offer-wrapper")) {
        result.push(Offer::build_from_node(&entry));
    }
}

fn get_page(url: String) -> reqwest::Response {
    let response = reqwest::get(&url).expect("Could not get url");
    assert!(response.status().is_success());

    response
}

fn format_query(query: &str) -> String {
    query.trim().replace(" ", "-")
}
