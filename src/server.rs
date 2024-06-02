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
//

use actix::{Actor, ActorContext, StreamHandler};
use actix_web::{
    dev::ServiceRequest, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use actix_web_httpauth::{
    extractors::bearer::{self, BearerAuth},
    middleware::HttpAuthentication,
};
use parking_lot::Mutex;
use std::{net::TcpStream, sync::Arc};
use tokio::{net::TcpSocket, sync::mpsc};

use crate::config::Config;
use crate::llm::{LlmBackend, OpenAiBackend};
use crate::{apitype, endpoint};
use crate::{appctx::AppContext, streamer::Streamer};

async fn index_html() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../static/index.html"))
}

mod auth {
    use crate::config::Config;

    pub fn validate_token(token: &str, config: &Config) -> bool {
        config.api_keys.iter().any(|key| key.key == token)
    }
}

async fn bearer_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req
        .app_data::<web::Data<Config>>()
        .map(|data| data.as_ref())
        .unwrap();
    let token = credentials.token();
    trace!("In bearer_validator, got token: {}", token);

    if !token.is_empty() {
        //let config = &ctx.config;

        if auth::validate_token(token, config) {
            Ok(req)
        } else {
            Err((actix_web::error::ErrorUnauthorized("Unauthorized"), req))
        }
    } else {
        Err((actix_web::error::ErrorUnauthorized("Unauthorized"), req))
    }
}

pub async fn run(config: Config) -> std::io::Result<()> {
    // Start the server

    let (host, port) = {
        let mut parts = config.listen.split(':');
        match (parts.next(), parts.next()) {
            (Some(host), Some(port)) => (host, port.parse().expect("port integer")),
            _ => ("127.0.0.1", 8080),
        }
    };

    println!("Starting server at http://{}:{}", host, port);

    let config = config.clone();

    let (tx_closer, rx_closer) = mpsc::channel::<std::net::SocketAddr>(10);
    let tx_closer = Arc::new(apitype::ClientCloser(tx_closer));
    let rx_closer = Arc::new(Mutex::new(rx_closer));

    HttpServer::new(move || {
        //let bearer_config = bearer::Config::default().realm("Unauthorized");
        let mut app = App::new()
            .app_data(web::Data::from(Arc::new(config.clone())))
            .app_data(web::Data::from(tx_closer.clone()))
            .wrap(HttpAuthentication::bearer(bearer_validator))
            .service(endpoint::chat_completions)
            .service(endpoint::broadcast)
            .service(endpoint::models)
            .route("/", web::get().to(index_html));

        if config.llm_backend == "openai" {
            debug!("use OpenAI backend");
            app = app.app_data(web::Data::from(AppContext::<OpenAiBackend>::from_config(
                &config,
            )));

            //app = app.wrap(HttpAuthentication::bearer(bearer_validator::<OpenAiBackend>));
            // } else if config.llm_backend == "pplx" {
            //     debug!("use Perplexity backend");
            // app = app.app_data(web::Data::from(AppContext::<PplxBackend>::from_config(
            //     &config,
            // )));
            //app = app.wrap(HttpAuthentication::bearer(bearer_validator<PplxBackend>));
        }

        app
    })
    // close socket connection when got signal
    .on_connect(move |c, _| {
        trace!("on_connect: {:?}", c);
        let rx_closer = rx_closer.clone();

        //tokio::spawn(async move {
        // wait for signal
        // and close the connection when got signal
        // let mut rx_closer = rx_closer.lock();
        // match rx_closer.recv().await {
        //     Some(port) => {
        //         debug!("Close connection for port: {}", port);
        //
        //         // c.downcast_ref::<TcpStream>()
        //         //     .unwrap()
        //         //     .shutdown(std::net::Shutdown::Both)
        //         //     .unwrap();
        //     }
        //     None => {
        //         debug!("No signal received, keep connection open");
        //     }
        // }
        //});
    })
    .bind((host, port))?
    .run()
    .await
}
