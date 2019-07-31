extern crate reqwest;
extern crate select;

use std::fmt;

use select::document::Document;
use select::predicate::{Class, Name, Predicate, Attr};

#[macro_use] extern crate prettytable;
use prettytable::{Table, Row};

fn main() {
    let mut offers = scrape("https://www.olx.pl/oferty/q-playstation-4-pro/?search%5Bfilter_float_price%3Afrom%5D=600&");

    // offers.sort_by(|a, b| (a.price.round() as u32).cmp(&(b.price.round() as u32)));
    offers.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap() );

    let mut table = Table::new();

    for offer in offers.iter() {
        table.add_row(offer.table_row());
    }

    table.printstd();

    let lowest_price = offers.iter().min_by_key(|o| o.price.round() as u32).unwrap();

    println!("Total items: {}", offers.len());
    println!("Item with a lowest price: {}", lowest_price);
}

struct Offer {
    title: String,
    price: f32,
    url: String,
}

impl fmt::Display for Offer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "title: {}, price: {}\n url: {}", self.title, self.price, self.url)
    }
}

impl Offer {
    fn table_row(&self) -> Row {
        row![
            self.title,
            self.price,
            self.url
        ]
    }

    fn build_from_node(node: &select::node::Node) -> Offer {
        let title = node.find(Name("a").descendant(Name("strong"))).next().expect("Could not parse detailsLink text").text();
        let url = node.find(Name("a")).next().expect("Could not parse detailsLink link").attr("href").expect("Could not parse detailsLink href");
        let price = node.find(Class("price").descendant(Name("strong"))).next().expect("Could not parse price").text();

        Offer {
            title: title,
            price: parse_price(&price).unwrap(),
            url: url.to_string(),
        }
    }
}

fn scrape(url: &str) -> Vec<Offer> {
    let mut collection = Vec::new();

    let pages = get_all_pages(url);

    for page in pages {
        parse_page(page, &mut collection);
    }

    collection
}

fn parse_page(response: reqwest::Response, result: &mut Vec<Offer>) {
    let page = Document::from_read(response).expect("Could not parse page");

    for entry in page.find(Class("offer-wrapper")) {
        result.push(Offer::build_from_node(&entry));
    }
}

fn get_all_pages(base_url: &str) -> Vec<reqwest::Response> {
    let response = reqwest::get(base_url).expect("Could not get url");
    assert!(response.status().is_success());

    let first_page = Document::from_read(response).expect("Could not parse first page");
    let pager = first_page.find(Class("pager")).next().expect("Could not parse pager");

    let total_pages = pager.find(Attr("data-cy", "page-link-last").descendant(Name("span"))).next().expect("Could not find last page").text();

    let mut pages = Vec::new();

    for page_number in 1..(parse_price(&total_pages).unwrap().round() as u32) {
        let page = get_page(format!("{}/?page={}", base_url, page_number.to_string()));

        pages.push(page);
    }

    pages
}

fn get_page(url: String) -> reqwest::Response {
    let response = reqwest::get(&url).expect("Could not get url");
    assert!(response.status().is_success());

    response
}

fn parse_price(input: &str) -> Option<f32> {
    match input.trim_matches(char::is_alphabetic).replace(" ", "").replace(",", ".").parse::<f32>() {
        Ok(v) => Some(v),
        Err(_) => Some(9999999f32),
    }
}
