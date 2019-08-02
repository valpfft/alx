extern crate reqwest;
extern crate select;
extern crate clap;

use std::fmt;

use clap::{Arg, App};

#[macro_use] extern crate prettytable;
use prettytable::{Table, Row};

mod olx_client;

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

            olx_client::build_url(&query)
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

    let mut offers = olx_client::scrape(&url);

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

pub struct Offer {
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
}

fn parse_price(input: &str) -> Option<f32> {
    match input.trim_matches(char::is_alphabetic).replace(" ", "").replace(",", ".").parse::<f32>() {
        Ok(v) => Some(v),
        Err(_) => Some(9999999f32),
    }
}
