use aws_mfa_profile::Sts;
use aws_sdk_sts::model::Credentials;
use aws_sdk_sts::output::GetSessionTokenOutput;
use aws_types::SdkConfig;
use clap::Parser;
use serde_json::Value;

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, BufReader};

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    profile: Option<String>,

    #[clap(short, long)]
    sts_file: String,

    #[clap(short, long)]
    credentials_file: String,
}

async fn get_config(profile: &Option<String>) -> SdkConfig {
    match profile {
        Some(profile) => {
            let cp = aws_config::profile::ProfileFileCredentialsProvider::builder()
                .profile_name(profile)
                .build();
            aws_config::from_env().credentials_provider(cp).load().await
        }
        None => aws_config::load_from_env().await,
    }
}

fn get_serial(profile: Option<String>, sts_file: String) -> (String, String) {
    let profile = match profile {
        Some(profile) => profile,
        None => String::from("default"),
    };

    let f = OpenOptions::new().read(true).open(sts_file).unwrap();
    let br = BufReader::new(f);
    let sts_json: Value = serde_json::from_reader(br).unwrap();
    let sts_json_array = sts_json.as_array().unwrap();
    let mut profile_json: Option<&Value> = None;
    for json in sts_json_array.iter() {
        if json.get("profile").unwrap().as_str().unwrap() == profile {
            profile_json = Some(&json);
            break;
        }
    }
    match profile_json {
        Some(profile_json) => {
            let serial = profile_json.get("serial").unwrap().as_str().unwrap();
            let sts_prof = profile_json.get("sts_profile").unwrap().as_str().unwrap();
            return (String::from(serial), String::from(sts_prof));
        }
        None => panic!("profile not found in sts_file"),
    }
}

fn get_token() -> String {
    print!("[input] token: ");
    io::stdout().flush().unwrap();

    let mut token = String::new();
    io::stdin().read_line(&mut token).unwrap();
    String::from(token.trim())
}

async fn get_session(sts: Sts, serial: String, token: String) -> (String, String, String) {
    let resp: GetSessionTokenOutput = sts.get_session_token(&serial, &token).await;
    let credentials: &Credentials = resp.credentials().unwrap();
    // println!("{:#?}", credentials);
    let key = String::from(credentials.access_key_id().unwrap());
    let skey = String::from(credentials.secret_access_key().unwrap());
    let token = String::from(credentials.session_token().unwrap());
    (key, skey, token)
}

fn create_credentials_file(
    credentials_file: &str,
    sts_prof: &str,
    key: &str,
    skey: &str,
    token: &str,
) {
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(credentials_file)
        .unwrap();
    let br = BufReader::new(f);
    for line in br.lines() {
        if line.unwrap() == format!("[{}]", sts_prof) {}
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = get_config(&args.profile).await;
    let (serial, sts_prof) = get_serial(args.profile, args.sts_file);
    // println!("serial  : {}", serial);
    // println!("sts_prof: {}", sts_prof);

    let sts = Sts::new(&config);
    let token = get_token();
    let (key, skey, token) = get_session(sts, serial, token).await;
    // println!("key  : {}", key);
    // println!("skey : {}", skey);
    // println!("token: {}", token);

    create_credentials_file(&args.credentials_file, &sts_prof, &key, &skey, &token);
}
