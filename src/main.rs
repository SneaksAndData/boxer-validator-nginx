mod http;
mod models;
mod services;

use crate::http::filters::jwt_filter::InternalTokenMiddlewareFactory;
use crate::http::urls::token_review;
use actix_web::{web, App, HttpServer};
use std::io::Result;
use crate::services::validation_service::{CedarValidationService, ValidationService};

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");

    env_logger::init();
    let addr = ("127.0.0.1", 8081);

    let c = CedarValidationService::new();
    println!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || {
        App::new()
            // The last middleware in the chain should always be InternalTokenMiddleware
            // to ensure that the token is valid in the beginning of the request processing
            .wrap(InternalTokenMiddlewareFactory::new())
            .app_data(web::Data::new(CedarValidationService::new()))
            .service(token_review)
    })
    .bind(addr)?
    .run()
    .await
}
