mod http;
mod models;
mod services;

use crate::http::controllers::{action_set, policy_set, schema};
use crate::http::controllers::{resource_set, token_review};
use crate::http::filters::jwt_filter::InternalTokenMiddlewareFactory;
use crate::http::openapi::ApiDoc;
use crate::services::backends;
use crate::services::cedar_validation_service::CedarValidationService;
use crate::services::configuration::models::AppSettings;
use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::repositories::policy_repository::read_write::PolicyDataRepository;
use crate::services::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
use crate::services::schema_provider::KubernetesSchemaProvider;
use actix_web::middleware::{Condition, Logger};
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use boxer_core::services::backends::BackendConfiguration;
use boxer_core::services::observability::composed_logger::ComposedLogger;
use boxer_core::services::observability::open_telemetry;
use boxer_core::services::service_provider::ServiceProvider;
use log::info;
use opentelemetry_instrumentation_actix_web::RequestTracing;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> Result<()> {
    ComposedLogger::new()
        .with_logger(open_telemetry::logging::init_logger()?)
        .with_logger(Box::new(env_logger::Builder::from_default_env().build()))
        .with_global_level(log::LevelFilter::Info)
        .init()?;

    let cm = AppSettings::new()?;
    info!("Configuration manager started");

    // open_telemetry::tracing::init_tracer()?;

    let current_backend = backends::new()
        .configure(&cm.backend.kubernetes, cm.instance_name.clone())
        .await?;

    let schema_provider = Arc::new(KubernetesSchemaProvider::new(current_backend.get()));
    let action_repository = current_backend.get();
    let resource_repository = current_backend.get();
    let policy_repository = current_backend.get();
    let cedar_validation_service = Arc::new(CedarValidationService::new(
        schema_provider,
        action_repository,
        resource_repository,
        policy_repository,
    ));

    let action_repository: Arc<ActionDataRepository> = current_backend.get();
    let resource_repository: Arc<ResourceDiscoveryDocumentRepository> = current_backend.get();
    let policy_repository: Arc<PolicyDataRepository> = current_backend.get();

    let production_mode = !std::env::var("BOXER_ISSUER_DEBUG").is_ok();

    let schema_repository: Arc<SchemaRepository> = current_backend.get();

    info!("listening on {}:{}", &cm.listen_address.ip(), &cm.listen_address.port());
    HttpServer::new(move || {
        App::new()
            .wrap(RequestTracing::new())
            .wrap(Logger::default())
            .app_data(web::Data::new(cedar_validation_service.clone()))
            .app_data(web::Data::new(schema_repository.clone()))
            .app_data(web::Data::new(action_repository.clone()))
            .app_data(web::Data::new(resource_repository.clone()))
            .app_data(web::Data::new(policy_repository.clone()))
            // The last middleware in the chain should always be InternalTokenMiddleware
            // to ensure that the token is valid in the beginning of the request processing
            .service(schema::crud())
            .service(action_set::crud())
            .service(resource_set::crud())
            .service(policy_set::crud())
            .service(
                web::scope("/token")
                    .wrap(Condition::new(production_mode, InternalTokenMiddlewareFactory::new()))
                    .service(token_review::token_review),
            )
            .service(SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
    .bind(cm.listen_address)?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
