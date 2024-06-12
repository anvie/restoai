// Copyright (C) 2024 Neuversity
// All Rights Reserved.
//
// NOTICE: All information contained herein is, and remains
// the property of Neuversity.
// The intellectual and technical concepts contained
// herein are proprietary to Neuversity
// and are protected by trade secret or copyright law.
// Dissemination of this information or reproduction of this material
// is strictly forbidden unless prior written permission is obtained
// from Neuversity.

#![allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use clap::Command;
use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::{fs, io::ErrorKind, path::Path, process::exit};

mod apitype;
mod appctx;
mod config;
mod endpoint;
mod llm;
mod server;
mod streamer;

use config::Config;

#[derive(Parser, Debug)]
#[command(name = "rust-rest")]
#[command(about = "Basic Rest API server in Rust")]
#[command(author, version, long_about=None)]
struct Args {
    // #[arg(short, long, default_value = "default.conf")]
    // config: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Run REST server")]
    Serve {
        #[arg(short, long, default_value = "default.conf")]
        config: String,

        #[arg(short, long, help = "Listen address, default 127.0.0.1")]
        listen: Option<String>,

        #[arg(short, long, help = "Listen port, default: 8080")]
        port: Option<u16>,
    },

    #[command(about = "Add API key")]
    AddApiKey {
        #[arg(short, long, default_value = "default.conf")]
        config: String,

        #[arg(short, long, help = "Name of the API key")]
        name: String,
    },
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args = Args::parse();

    match args.command {
        Commands::Serve {
            config,
            listen,
            port,
        } => {
            println!("Value for config: {}", config);

            let config: Config = match fs::read_to_string(&config) {
                Ok(config) => toml::from_str(&config).unwrap(),
                Err(e) => {
                    if e.kind() == ErrorKind::NotFound {
                        println!("`{}` not exists.", config);
                        exit(2);
                    } else {
                        panic!("Error: {}", e);
                    }
                }
            };
            println!("Config: {:#?}", config);
            server::run(config, listen.as_deref(), port).await?
        }
        Commands::AddApiKey { config, name } => {
            println!("Add API key");

            if let Ok(config_str) = fs::read_to_string(&config) {
                let mut conf: Config = toml::from_str(&config_str).expect("Cannot parse config");
                let key = generate_key();
                conf.api_keys.push(config::ApiKey {
                    key: key.clone(),
                    name,
                    description: None,
                    permissions: vec!["read".to_string()],
                });
                let config_str = toml::to_string(&conf).expect("Cannot serialize config");
                fs::write(&config, config_str).expect("Cannot write config");
                println!("API key added: {}", key);
                //println!("Config: {:#?}", config);
            }
        }
    }

    Ok(())
}

fn generate_key() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    let code: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .collect::<Vec<u8>>()
        .into_iter()
        .map(char::from)
        .collect();
    format!("nsk-{}", code)
}
