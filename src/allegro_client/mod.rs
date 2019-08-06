use crate::Offer;
use crate::parse_price;

extern crate slug;

use slug::slugify;

use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};

use serde::{Serialize, Deserialize};

mod auth;

static BASE_URL: &str = "https://api.allegro.pl";

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    items: Items,
}

#[derive(Serialize, Deserialize, Debug)]
struct Items {
    regular: Vec<AllegroOffer>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AllegroOffer {
    id: String,
    name: String,
    selling_mode: SellingMode,
}

#[derive(Serialize, Deserialize, Debug)]
struct SellingMode {
    format: String,
    price: Price,
}

#[derive(Serialize, Deserialize, Debug)]
struct Price {
    amount: String,
    currency: String
}

pub fn scrape(params: &HashMap<&str, &str>) -> Vec<Offer> {
    let access_token = auth::authenticate();
    let mut search_result = search(access_token, params);

    parse_response(&mut search_result)
}

impl AllegroOffer {
    fn url(&self) -> String {
        let slug = slugify(&self.name);

        format!("https://allegro.pl/oferta/{}-{}", slug, self.id)
    }
}

impl Offer {
    fn build_from_allegro_offer(allegro_offer: &AllegroOffer) -> Offer {
        let title = allegro_offer.name.to_string();
        Offer {
            title: title,
            price: parse_price(&allegro_offer.selling_mode.price.amount).expect("Could not parse price"),
            url: allegro_offer.url()
        }
    }
}

fn parse_response(response: &mut reqwest::Response) -> Vec<Offer> {
    let mut collection = Vec::new();

    let allegro_response: Response = response.json().unwrap();

    for allegro_offer in allegro_response.items.regular.iter() {
        collection.push(Offer::build_from_allegro_offer(&allegro_offer));
    }

    collection
}

fn search(token: String, params: &HashMap<&str, &str>) -> reqwest::Response {
    let client = reqwest::Client::new();

    let mut query = HashMap::new();

    query.insert("phrase", params.get("query").unwrap());

    if params.contains_key("min_price") {
        query.insert("price.from", params.get("min_price").unwrap());
    }

    if params.contains_key("max_price") {
        query.insert("price.to", params.get("max_price").unwrap());
    }

    let url = format!("{}/offers/listing", BASE_URL);

    let response = client.get(&url)
                         .headers(construct_headers())
                         .query(&query)
                         .bearer_auth(token)
                         .send();

    response.unwrap()
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.allegro.public.v1+json"));

    headers
}
