use boxer_core::services::backends::SchemaRepositorySource;
mod http;
mod models;
mod services;

use crate::http::controllers::{action_set, schema};
use crate::http::controllers::{resource_set, token_review};
use crate::http::filters::jwt_filter::InternalTokenMiddlewareFactory;
use crate::http::openapi::ApiDoc;
use crate::services::backends;
use crate::services::base::actions_repository_source::ActionRepositorySource;
use crate::services::base::policy_repository_source::PolicyRepositorySource;
use crate::services::base::resource_repository_source::ResourceRepositorySource;
use crate::services::cedar_validation_service::CedarValidationService;
use crate::services::configuration::models::AppSettings;
use crate::services::schema_provider::KubernetesSchemaProvider;
use actix_web::middleware::Condition;
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use boxer_core::services::backends::BackendConfiguration;
use log::info;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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

    let schema_provider = Arc::new(KubernetesSchemaProvider::new(
        current_backend.get_schemas_repository(),
        cm.backend.kubernetes.schema_repository.name,
    ));
    let action_repository = current_backend.get_readonly_repository();
    let resource_repository = current_backend.get_resource_read_only_repository();
    let policy_repository = current_backend.get_policy_readonly_repository();
    let cedar_validation_service = Arc::new(CedarValidationService::new(
        schema_provider,
        action_repository,
        resource_repository,
        policy_repository,
    ));

    let action_repository = current_backend.get_action_data_repository();

    let debug_mode = !std::env::var("BOXER_ISSUER_DEBUG").is_ok();

    let schema_repository = current_backend.get_schemas_repository();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cedar_validation_service.clone()))
            .app_data(web::Data::new(schema_repository.clone()))
            .app_data(web::Data::new(action_repository.clone()))
            // The last middleware in the chain should always be InternalTokenMiddleware
            // to ensure that the token is valid in the beginning of the request processing
            .wrap(Condition::new(debug_mode, InternalTokenMiddlewareFactory::new()))
            .service(schema::crud())
            .service(action_set::crud())
            .service(token_review::get)
            .service(SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
    .bind(addr)?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
