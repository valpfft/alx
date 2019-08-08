extern crate reqwest;
extern crate select;
extern crate clap;
extern crate csv;

use std::{fmt, io};
use std::collections::HashMap;

use clap::{Arg, App};
use serde::Serialize;

#[macro_use] extern crate prettytable;
use prettytable::{Table, Row};

mod olx_client;
mod allegro_client;

fn main() {
    let matches = App::new("Olxer")
        .version("0.1.0")
        .author("Valiantsin Mikhaliuk <valiantsin.mikhaliuk@gmail.com>")
        .about("Hey Olx'er! Let's find something!")
        .arg(Arg::with_name("query")
             .required(true)
             .short("q")
             .long("query")
             .takes_value(true)
             .help("Search query"))
        .arg(Arg::with_name("min_price")
             .long("min-price")
             .takes_value(true)
             .help("Minimum price"))
        .arg(Arg::with_name("max_price")
             .long("max-price")
             .takes_value(true)
             .help("Maximum price"))
        .arg(Arg::with_name("export_csv")
             .long("export-csv")
             .takes_value(false)
             .help("Exports search result into csv"))
        .get_matches();

    let mut params = HashMap::new();
    params.insert("query", matches.value_of("query").unwrap());

    if matches.is_present("min_price") {
        params.insert("min_price", matches.value_of("min_price").unwrap());
    }

    if matches.is_present("max_price") {
        params.insert("max_price", matches.value_of("max_price").unwrap());
    }

    let mut offers = Vec::new();
    offers.append(&mut allegro_client::scrape(&params));
    offers.append(&mut olx_client::scrape(&params));

    offers.sort_unstable_by(|a, b| a.price.partial_cmp(&b.price).expect("Could not sort offers"));

    if matches.is_present("export_csv") {
        export_csv(&offers);
    } else {
        render_table(&offers);

        let lowest_price = offers.iter().min_by_key(|o| o.price as u32).expect("Could not find offer with a lowest price");

        println!("Total items: {}", offers.len());
        println!("Item with a lowest price: {}", lowest_price);
    }
}

fn export_csv(offers: &Vec<Offer>) {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    for offer in offers.iter() {
        wtr.serialize(offer).expect("Could not serialize offer into CSV");
    }

    wtr.flush().unwrap();
}

fn render_table(offers: &Vec<Offer>) {
    let mut table = Table::new();

    for offer in offers.iter() {
        table.add_row(offer.table_row());
    }

    table.printstd();
}

#[derive(Serialize)]
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
