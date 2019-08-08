extern crate chrono;
extern crate time;

use chrono::prelude::*;
use time::Duration;

use std::io::{ErrorKind, Error};
use std::env;
use std::fs::File;
use std::io::Read;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use std::thread;

static CLIENT_ID: &str = "c53d0ffa80504646a89af61d55767aaa";
static CLIENT_SECRET: &str = "v3nBqCptmSSdFtkqsFtWng8Q2K3qlJRfwucFnWIXBt64gBKO10Q2TsEyAUtCZjX3";

pub fn authenticate() -> String {
    let config = read_config();

    let config = match config {
        Ok(_) => config.unwrap().token.access_token,
        Err(e) => {
            println!("Could'nt retrieve config file: {}", e);

            let client = reqwest::Client::new();

            let device_token = request_device_token(&client);

            println!("Pls follow: {}", device_token.verification_uri_complete);

            let token = request_token(client, &device_token.device_code, device_token.interval);

            let config = write_config(token);

            config.token.access_token
        }
    };

    config
}

#[derive(Deserialize, Serialize)]
struct Config {
    token: Token,
    expire_at: DateTime<Utc>,
}

impl Config {
    fn is_valid(&self) -> bool {
        Utc::now() < self.expire_at
    }
}

#[derive(Deserialize, Serialize)]
struct Token {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

#[derive(Deserialize)]
struct DeviceToken {
    device_code: String,
    interval: u64,
    verification_uri_complete: String,
}

fn read_config() -> Result<Config, Error> {
    let home_path = env::var("HOME").unwrap();
    let config_path = format!("{}/.config/olxer_config.json", home_path);

    let file = File::open(config_path);

    let mut file = match file {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    let mut content = String::new();

    let content = match file.read_to_string(&mut content) {
        Ok(_) => Ok(content),
        Err(e) => Err(e),
    };

    let config: Config = serde_json::from_str(&content.unwrap()).unwrap();

    if config.is_valid() {
        Ok(config)
    } else {
        Err(Error::new(ErrorKind::Other, "Config is outdated"))
    }
}

fn write_config(token: Token) -> Config {
    let home_path = env::var("HOME").unwrap();
    let config_path = format!("{}/.config/olxer_config.json", home_path);
    println!("{}", config_path);

    let expires_in = Duration::seconds(token.expires_in - 100);

    let config = Config { token: token, expire_at: Utc::now() + expires_in };

    ::serde_json::to_writer(&File::create(config_path).unwrap(), &config)
        .expect("Could not write config file");

    config
}

fn request_token(client: reqwest::Client, device_code: &str, interval: u64) -> Token {
    let interval_duration = std::time::Duration::from_secs(interval);

    println!("Pulling in {}", interval);

    thread::sleep(interval_duration);

    let url = format!("https://allegro.pl/auth/oauth/token?grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code&device_code={}", device_code);

    let mut response =  client.post(&url)
                              .basic_auth(CLIENT_ID, Some(CLIENT_SECRET))
                              .send()
                              .expect("Couldn't get request token");

    if let Ok(token) = response.json::<Token>() {
        token
    } else {
        request_token(client, device_code, interval)
    }
}

fn request_device_token(client: &reqwest::Client) -> DeviceToken {
    let params = [("client_id", CLIENT_ID)];
    let token: DeviceToken = client.post("https://allegro.pl/auth/oauth/device")
                                   .headers(construct_headers())
                                   .basic_auth(CLIENT_ID, Some(CLIENT_SECRET))
                                   .form(&params)
                                   .send()
                                   .expect("Could not get device token")
                                   .json()
                                   .expect("Couldn't parse json from device token response");

    token
}


fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("x-www-form-urlencoded"));

    headers
}
