use aws_mfa_profile::Sts;
use aws_sdk_sts::model::Credentials;
use aws_sdk_sts::output::GetSessionTokenOutput;
use aws_types::SdkConfig;
use clap::Parser;
use serde_json::Value;

use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::io::{self, BufReader};

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    profile: Option<String>,

    #[clap(short, long)]
    mfa_file: String,

    #[clap(short, long)]
    credentials_file: Option<String>,
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

/// return (serial, mfa_profile)
fn get_serial(profile: Option<String>, mfa_file: String) -> (String, String) {
    let profile = match profile {
        Some(profile) => profile,
        None => String::from("default"),
    };

    let f = OpenOptions::new()
        .read(true)
        .open(&mfa_file)
        .expect(&format!("file open failed: {}", mfa_file));
    let br = BufReader::new(f);
    let mfa_json: Value =
        serde_json::from_reader(br).expect(&format!("BufReader failed: {}", mfa_file));
    let mfa_json_array = mfa_json
        .as_array()
        .expect(&format!("file format failed: {}", mfa_file));
    let mut profile_json: Option<&Value> = None;
    for json in mfa_json_array.iter() {
        match json.get("profile") {
            Some(p) => {
                if p.as_str().unwrap() == profile {
                    profile_json = Some(&json);
                }
            }
            None => panic!("'profile' key not found: {}", mfa_file),
        }
    }
    match profile_json {
        Some(profile_json) => {
            let serial = profile_json
                .get("serial")
                .expect(&format!("'serial' key not found: {}", mfa_file));
            let serial = serial.as_str().unwrap();
            let mfa_prof = profile_json
                .get("mfa_profile")
                .expect(&format!("'mfa_profile' key not found: {}", mfa_file));
            let mfa_prof = mfa_prof.as_str().unwrap();
            return (String::from(serial), String::from(mfa_prof));
        }
        None => panic!("profile not found in mfa_file"),
    }
}

fn get_token() -> String {
    print!("[input] token code: ");
    io::stdout().flush().unwrap();

    let mut token = String::new();
    io::stdin()
        .read_line(&mut token)
        .expect("input token read failed");
    String::from(token.trim())
}

/// return (access_key_id, secret_access_key, session_token)
async fn get_session(sts: Sts, serial: String, token: String) -> (String, String, String) {
    let resp: GetSessionTokenOutput = sts.get_session_token(&serial, &token).await;
    let credentials: &Credentials = resp.credentials().expect("get_session_token failed");
    // println!("{:#?}", credentials);
    let key = String::from(
        credentials
            .access_key_id()
            .expect("get access_key_id failed"),
    );
    let skey = String::from(
        credentials
            .secret_access_key()
            .expect("get secret_access_key failed"),
    );
    let token = String::from(
        credentials
            .session_token()
            .expect("get session_token failed"),
    );
    (key, skey, token)
}

fn push_key(new_file_data: &mut Vec<String>, key: &str) {
    new_file_data.push(format!("aws_access_key_id = {}", key));
}

fn push_skey(new_file_data: &mut Vec<String>, skey: &str) {
    new_file_data.push(format!("aws_secret_access_key = {}", skey));
}

fn push_token(new_file_data: &mut Vec<String>, token: &str) {
    new_file_data.push(format!("aws_session_token = {}", token));
}

fn create_credentials_file_data(
    credentials_file: &str,
    mfa_prof: &str,
    key: &str,
    skey: &str,
    token: &str,
) -> Vec<String> {
    let f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(credentials_file)
        .expect(&format!("file open failed: {}", credentials_file));
    let br = BufReader::new(f);

    let mut in_target_profile = false;
    let mut writed_key = false;
    let mut writed_skey = false;
    let mut writed_token = false;

    let mut new_file_data: Vec<String> = vec![];
    for line in br.lines() {
        let line = String::from(line.unwrap());
        if line == format!("[{}]", mfa_prof) {
            in_target_profile = true;
            new_file_data.push(line);
            continue;
        }

        if in_target_profile {
            if line.to_lowercase().contains("aws_access_key_id") {
                push_key(&mut new_file_data, key);
                writed_key = true;
                continue;
            }
            if line.to_lowercase().contains("aws_secret_access_key") {
                push_skey(&mut new_file_data, skey);
                writed_skey = true;
                continue;
            }
            if line.to_lowercase().contains("aws_session_token") {
                push_token(&mut new_file_data, token);
                writed_token = true;
                continue;
            }
            if line.starts_with("[") {
                in_target_profile = false;
                if !writed_key {
                    push_key(&mut new_file_data, key);
                    writed_key = true;
                }
                if !writed_skey {
                    push_skey(&mut new_file_data, skey);
                    writed_skey = true;
                }
                if !writed_token {
                    push_token(&mut new_file_data, token);
                    writed_token = true;
                }
                new_file_data.push(line);
                continue;
            }
            new_file_data.push(line);
        } else {
            new_file_data.push(line);
        }
    }

    if in_target_profile {
        if !writed_key {
            push_key(&mut new_file_data, key);
        }
        if !writed_skey {
            push_skey(&mut new_file_data, skey);
        }
        if !writed_token {
            push_token(&mut new_file_data, token);
        }
    } else {
        if !(writed_key && writed_skey && writed_token) {
            panic!("create_credentials_file_data failed");
        }
    }
    new_file_data
}

fn backup_credentials_file(credentials_file: &str) {
    fs::rename(credentials_file, format!("{}.bkp", credentials_file))
        .expect(&format!("file rename failed: {}", credentials_file));
}

fn create_credentials_file(
    credentials_file: &str,
    mfa_prof: &str,
    key: &str,
    skey: &str,
    token: &str,
) {
    let new_credentials_file_data =
        create_credentials_file_data(credentials_file, mfa_prof, key, skey, token);
    backup_credentials_file(credentials_file);
    let mut new_credentials_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(credentials_file)
        .expect(&format!("file create failed: {}", credentials_file));
    writeln!(
        new_credentials_file,
        "{}",
        new_credentials_file_data.join("\n")
    )
    .expect(&format!("file write failed: {}", credentials_file));
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = get_config(&args.profile).await;
    let (serial, sts_prof) = get_serial(args.profile, args.mfa_file);
    // println!("serial  : {}", serial);
    // println!("mfa_prof: {}", mfa_prof);

    let sts = Sts::new(&config);
    let token = get_token();
    let (key, skey, token) = get_session(sts, serial, token).await;
    // println!("key  : {}", key);
    // println!("skey : {}", skey);
    // println!("token: {}", token);

    let credentials_file = match args.credentials_file {
        Some(credentials_file) => credentials_file,
        None => String::from("credentials"),
    };
    create_credentials_file(&credentials_file, &sts_prof, &key, &skey, &token);
}
