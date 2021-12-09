extern crate chrono;
extern crate time;

use chrono::{DateTime, Duration, Utc};

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Error;
use std::io::Read;

use std::thread;

pub fn init_config(path: &str, cid: &str, sec: &str) {
    let config_file = &std::fs::File::create(path).unwrap();
    let config: Config = Config {
        expire_at: Utc::now(), //Needs refreshing right now
        cid: cid.to_string(),
        sec: sec.to_string(),
        token: Token {
            access_token: String::new(),
            refresh_token: String::new(),
            expires_in: -100,
        },
    };

    return ::serde_json::to_writer(config_file, &config).unwrap();
}

pub fn authenticate(config_path: &str) -> String {
    let config_file = &mut File::open(config_path).expect(
        r#"Cannot load config file. Specify it with -config or create new using `-setup` flag"#,
    );
    let mut config: Config = read_config(config_file).unwrap();

    if config.is_valid() {
        return config.token.access_token;
    } else {
        let ref cid = &config.cid;
        let ref sec = &config.sec;
        let client = reqwest::Client::new();
        let device_token = request_device_token(&client, cid, sec);

        println!(
            "Pls register your app at: {}",
            device_token.verification_uri_complete
        );

        let token = request_token(
            client,
            &device_token.device_code,
            device_token.interval,
            cid,
            sec,
        );

        let token = write_token(&mut config, config_path, token);

        token.access_token.to_string()
    }
}

#[derive(Deserialize, Serialize)]
struct Config {
    token: Token,
    expire_at: DateTime<Utc>,
    cid: String,
    sec: String,
}

impl Config {
    fn is_valid(&self) -> bool {
        Utc::now() <= self.expire_at
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

fn read_config(file: &mut File) -> Result<Config, Error> {
    let mut content = String::new();

    let config: Config = serde_json::from_str(
        &(match file.read_to_string(&mut content) {
            Err(e) => Err(e),
            Ok(_) => Ok(content),
        })
        .unwrap(),
    )
    .unwrap();

    return Ok(config);
}

fn write_token<'a>(config: &'a mut Config, path: &str, token: Token) -> &'a Token {
    let expires_in = Duration::seconds(token.expires_in - 100);
    config.token = token;
    config.expire_at = Utc::now() + expires_in;

    ::serde_json::to_writer(&File::create(path).unwrap(), &config)
        .expect("Could not write config file");

    &config.token
}

fn request_token(
    client: reqwest::Client,
    device_code: &str,
    interval: u64,
    cid: &str,
    sec: &str,
) -> Token {
    let interval_duration = std::time::Duration::from_secs(interval);

    println!("Pulling in {}", interval);

    thread::sleep(interval_duration);

    let url = format!("https://allegro.pl/auth/oauth/token?grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Adevice_code&device_code={}", device_code);

    let mut response = client
        .post(&url)
        .basic_auth(cid, Some(sec))
        .send()
        .expect("Couldn't get request token");

    if let Ok(token) = response.json::<Token>() {
        token
    } else {
        request_token(client, device_code, interval, cid, sec)
    }
}

fn request_device_token(client: &reqwest::Client, cid: &str, client_secret: &str) -> DeviceToken {
    let params = [("client_id", cid)];
    let token: DeviceToken = client
        .post("https://allegro.pl/auth/oauth/device")
        .headers(construct_headers())
        .basic_auth(cid, Some(client_secret))
        .form(&params)
        .send()
        .expect("Could not get device token")
        .json()
        .expect("Couldn't parse json from device token response");

    token
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("x-www-form-urlencoded"),
    );

    headers
}
