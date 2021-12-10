extern crate clap;
extern crate csv;
extern crate reqwest;
extern crate select;

use std::collections::HashMap;
use std::{fmt, io};

use clap::{App, Arg};
use serde::Serialize;

use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table};
mod allegro_lokalnie_client;
mod olx_client;

fn main() {
    let default_config_path: &str =
        &format!("{}/.config/alx_config.json", std::env::var("HOME").unwrap());

    let matches = App::new("alx")
        .version("0.3.0")
        .author("Valentin Michaluk <valentin.michaluk@gmail.com>")
        .about("Hey Alx'er! Let's find something!")
        .arg(
            Arg::with_name("query")
                .required_unless("setup")
                .short("q")
                .long("query")
                .takes_value(true)
                .help("Search query"),
        )
        .arg(
            Arg::with_name("min_price")
                .long("min-price")
                .takes_value(true)
                .help("Minimum price"),
        )
        .arg(
            Arg::with_name("max_price")
                .long("max-price")
                .takes_value(true)
                .help("Maximum price"),
        )
        .arg(
            Arg::with_name("export_csv")
                .long("export-csv")
                .takes_value(false)
                .help("Exports search result into csv"),
        )
        .arg(
            Arg::with_name("config_path")
                .long("config-path")
                .short("c")
                .takes_value(true)
                .help("Config path.")
                .default_value(default_config_path),
        )
        .arg(
            Arg::with_name("setup")
                .long("setup")
                .takes_value(false)
                .help("Perform initial setup?"),
        )
        .get_matches();

    // let config_path: &str = matches.value_of("config_path").unwrap();
    // if matches.is_present("setup") {
    // }
    let mut params = HashMap::new();
    params.insert("query", matches.value_of("query").unwrap());

    if matches.is_present("min_price") {
        params.insert("min_price", matches.value_of("min_price").unwrap());
    }

    if matches.is_present("max_price") {
        params.insert("max_price", matches.value_of("max_price").unwrap());
    }

    let mut offers = Vec::new();
    offers.append(&mut olx_client::scrape(&params));
    offers.append(&mut allegro_lokalnie_client::scrape(&params));
    offers.sort_unstable_by(|a, b| {
        a.price
            .partial_cmp(&b.price)
            .expect("Could not sort offers")
    });

    if matches.is_present("export_csv") {
        export_csv(&offers);
    } else {
        render_table(&offers);

        println!("Total items: {}", offers.len());

        match { offers.iter().min_by_key(|o| o.price as u32) } {
            Some(lp) => {
                println!("Item with a lowest price: {}", lp);
            }
            _ => (),
        }
    }
}

fn export_csv(offers: &Vec<Offer>) {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    for offer in offers.iter() {
        wtr.serialize(offer)
            .expect("Could not serialize offer into CSV");
    }

    wtr.flush().unwrap();
}

fn render_table(offers: &Vec<Offer>) {
    let mut table = Table::new();

    table
        .set_header(vec!["Title", "Price", "URL"])
        .set_content_arrangement(ContentArrangement::DynamicFullWidth);

    for offer in offers.iter() {
        table.add_row(vec![
            Cell::new(&offer.title),
            Cell::new(&offer.price.to_string()),
            Cell::new(&offer.url).add_attribute(Attribute::Italic),
        ]);
    }

    let url_column = table.get_column_mut(2).unwrap();
    url_column.set_padding((1, 1));
    url_column.set_cell_alignment(CellAlignment::Left);

    println!("{}", table);
}

#[derive(Serialize)]
pub struct Offer {
    title: String,
    price: f32,
    url: String,
}

impl fmt::Display for Offer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "title: {}, price: {}\n url: {}",
            self.title, self.price, self.url
        )
    }
}

fn parse_price(input: &str) -> Option<f32> {
    match {
        let t = input
            .trim_matches(char::is_alphabetic)
            .trim_matches(char::is_whitespace)
            .replace(",", ".");

        t.parse::<f32>()
    } {
        Ok(v) => Some(v),
        Err(_) => Some(9999999f32),
    }
}
