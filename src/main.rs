mod http;
mod models;
mod services;


use crate::http::filters::jwt_filter::InternalTokenMiddlewareFactory;
use crate::http::urls::token_review;
use crate::services::cedar_validation_service::CedarValidationService;
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use log::info;
use crate::services::configuration::models::AppSettings;

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");

    env_logger::init();
    let addr = ("127.0.0.1", 8081);

    let cm = AppSettings::new()?;
    info!("Instance name {}", cm.instance_name);
    
    println!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Box::new(CedarValidationService::new())))
            // The last middleware in the chain should always be InternalTokenMiddleware
            // to ensure that the token is valid in the beginning of the request processing
            .wrap(InternalTokenMiddlewareFactory::new())
            .service(token_review)
    })
    .bind(addr)?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
