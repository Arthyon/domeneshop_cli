use serde::Deserialize;
use std::{fs, path::PathBuf};

use domeneshop_client::client::{DomeneshopClient, DomeneshopClientConfiguration};

use crate::{constants::CREDENTIALS_FILENAME, log_and_fail, Args};

#[derive(Deserialize)]
pub struct ApiCredentials {
    pub token: String,
    pub secret: String,
}

pub fn get_client(args: &Args, data_dir: &PathBuf) -> Option<DomeneshopClient> {
    let credentials = get_api_credentials(args, data_dir);
    match credentials {
        None => None,
        Some(credentials) => {
            let client = DomeneshopClient::new(
                credentials.token,
                credentials.secret,
                DomeneshopClientConfiguration::default(),
            );

            match client {
                Err(err) => {
                    _ = log_and_fail("Failed to create domeneshop client", err);
                    None
                }
                Ok(client) => Some(client),
            }
        }
    }
}

fn get_api_credentials(args: &Args, data_dir: &PathBuf) -> Option<ApiCredentials> {
    match (&args.secret, &args.token) {
        (Some(secret), Some(token)) => {
            println!("Using credentials from arguments");
            Some(ApiCredentials {
                secret: secret.clone(),
                token: token.clone(),
            })
        }
        _ => {
            let mut credentials = data_dir.clone();
            credentials.push(CREDENTIALS_FILENAME);
            let data = fs::read_to_string(credentials);
            match data {
                Err(err) => {
                    println!("Could not find credentials-file: {}", err);
                    None
                }
                Ok(credentials) => {
                    let json: Result<ApiCredentials, serde_json::Error> =
                        serde_json::from_str(&credentials);
                    match json {
                        Err(err) => {
                            println!("Error deserializing credentials: {}", err);
                            None
                        }
                        Ok(json) => Some(json),
                    }
                }
            }
        }
    }
}
