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
use std::{fs, io::ErrorKind, process::exit};

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
    #[arg(short, long, default_value = "default.conf")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Run REST server")]
    Serve,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args = Args::parse();
    println!("Value for config: {}", args.config);

    let config: Config = match fs::read_to_string(&args.config) {
        Ok(config) => toml::from_str(&config).unwrap(),
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                println!("`{}` not exists.", args.config);
                exit(2);
            } else {
                panic!("Error: {}", e);
            }
        }
    };
    println!("Config: {:#?}", config);

    //if Commands::Serve = args.command {
    match args.command {
        Commands::Serve => server::run(config).await?,
        _ => {}
    }

    Ok(())
}
