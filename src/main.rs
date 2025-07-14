use boxer_core::services::backends::SchemaRepositorySource;
mod http;
mod models;
mod services;

use crate::http::controllers::schema;
use crate::http::controllers::token_review;
use crate::http::filters::jwt_filter::InternalTokenMiddlewareFactory;
use crate::services::backends;
use crate::services::cedar_validation_service::CedarValidationService;
use crate::services::configuration::models::AppSettings;
use crate::services::schema_provider::KubernetesSchemaProvider;
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use boxer_core::services::backends::BackendConfiguration;
use log::info;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");

    env_logger::init();
    let addr = ("127.0.0.1", 8081);

    let cm = AppSettings::new()?;
    info!("Instance name {}", cm.instance_name);

    println!("listening on {}:{}", &addr.0, &addr.1);

    let current_backend = backends::new()
        .configure(&cm.backend.kubernetes, cm.instance_name.clone())
        .await?;
    let schema_provider = Arc::new(KubernetesSchemaProvider::new(current_backend.get_schemas_repository()));
    let cedar_validation_service = Arc::new(CedarValidationService::new(schema_provider));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cedar_validation_service.clone()))
            // The last middleware in the chain should always be InternalTokenMiddleware
            // to ensure that the token is valid in the beginning of the request processing
            .wrap(InternalTokenMiddlewareFactory::new())
            .service(schema::crud())
            .service(token_review::get)
    })
    .bind(addr)?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
