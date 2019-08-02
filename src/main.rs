extern crate reqwest;
extern crate select;
extern crate clap;

use std::fmt;

use select::document::Document;
use select::predicate::{Class, Name, Predicate, Attr};

use clap::{Arg, App};

#[macro_use] extern crate prettytable;
use prettytable::{Table, Row};

fn main() {
    let matches = App::new("Olxer")
        .version("0.1.0")
        .author("Valiantsin Mikhaliuk <valiantsin.mikhaliuk@gmail.com>")
        .about("Hey Olx'er! Let's find something!")
        .arg(Arg::with_name("url")
             .short("u")
             .long("url")
             .takes_value(true)
             .help("Base url (first page)")
             .conflicts_with("query"))
        .arg(Arg::with_name("query")
             .short("q")
             .long("query")
             .takes_value(true)
             .help("Search query")
             .conflicts_with("url"))
        .arg(Arg::with_name("min_price")
             .long("min-price")
             .takes_value(true)
             .help("Minimum price"))
        .arg(Arg::with_name("max_price")
             .long("max-price")
             .takes_value(true)
             .help("Maximum price"))
        .get_matches();

    let mut url = match matches.value_of("url") {
        Some(url) => url.to_string(),
        None => {
            let query = matches.value_of("query").expect("Neither query is missing or url is not provided.");

            build_url(&query)
        },
    };

    url = match matches.value_of("min_price") {
        Some(min_price) => {
            add_filter(&format!("search[filter_float_price:from]={}", min_price), &mut url);

            url
        }, 
        None => url
    };

    url = match matches.value_of("max_price") {
        Some(max_price) => {
            add_filter(&format!("search[filter_float_price:to]={}", max_price), &mut url);

            url
        }, 
        None => url
    };

    println!("Scraping following url: {}", url);

    let mut offers = scrape(&url);

    offers.sort_unstable_by(|a, b| a.price.partial_cmp(&b.price).expect("Could not sort offers"));

    render_table(&offers);

    let lowest_price = offers.iter().min_by_key(|o| o.price as u32).expect("Could not find offer with a lowest price");

    println!("Total items: {}", offers.len());
    println!("Item with a lowest price: {}", lowest_price);
}

fn add_filter(filter: &str, url: &mut String) {
    match url.find("?") {
        Some(_) => url.push_str(&format!("&{}", filter)),
        None => url.push_str(&format!("?{}", filter)),
    };
}

fn render_table(offers: &Vec<Offer>) {
    let mut table = Table::new();

    for offer in offers.iter() {
        table.add_row(offer.table_row());
    }

    table.printstd();
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
            price: parse_price(&price).expect("Could not parse price"),
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

fn get_page(url: String) -> reqwest::Response {
    let response = reqwest::get(&url).expect("Could not get url");
    assert!(response.status().is_success());

    response
}

static BASE_URL: &str = "https://www.olx.pl/oferty";
fn build_url(query: &str) -> String {
    format!("{}/q-{}", BASE_URL, format_query(&query))
}

fn format_query(query: &str) -> String {
    query.trim().replace(" ", "-")
}

fn parse_price(input: &str) -> Option<f32> {
    match input.trim_matches(char::is_alphabetic).replace(" ", "").replace(",", ".").parse::<f32>() {
        Ok(v) => Some(v),
        Err(_) => Some(9999999f32),
    }
}
