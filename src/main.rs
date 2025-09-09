mod http;
mod models;
mod services;

use crate::http::controllers::v1;
use crate::http::openapi::ApiDoc;
use crate::services::authorizer::Authorizer;
use crate::services::backends;
use crate::services::cedar_validation_service::CedarValidationService;
use crate::services::configuration::models::AppSettings;
use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::repositories::policy_repository::read_write::PolicyDataRepository;
use crate::services::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
use crate::services::schema_provider::KubernetesSchemaProvider;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use boxer_core::services::audit::log_audit_service::LogAuditService;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use boxer_core::services::backends::BackendConfiguration;
use boxer_core::services::observability::composed_logger::ComposedLogger;
use boxer_core::services::observability::open_telemetry;
use boxer_core::services::observability::open_telemetry::metrics::init_metrics;
use boxer_core::services::observability::open_telemetry::tracing::init_tracer;
use boxer_core::services::service_provider::ServiceProvider;
use env_filter::Builder;
use log::info;
use opentelemetry_instrumentation_actix_web::RequestTracing;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> Result<()> {
    let mut builder = Builder::new();

    let filter = if let Ok(ref filter) = std::env::var("RUST_LOG") {
        builder.parse(filter);
        builder.build()
    } else {
        Builder::default().parse("info").build()
    };

    let cm = AppSettings::new()?;

    let logger = ComposedLogger::new();
    let logger = {
        if cm.opentelemetry.log_settings.enabled {
            logger.with_logger(open_telemetry::logging::init_logger()?)
        } else {
            logger
        }
    };

    logger
        .with_logger(Box::new(env_logger::Builder::from_default_env().build()))
        .with_global_level(filter)
        .init()?;

    info!("Configuration manager started");

    if cm.opentelemetry.tracing_settings.enabled {
        info!("Tracing is enabled, starting tracer...");
        init_tracer()?;
    }

    if cm.opentelemetry.metrics_settings.enabled {
        info!("Metrics is enabled, starting metrics...");
        init_metrics()?;
    }
    let current_backend = backends::new()
        .configure(&cm.backend.kubernetes, cm.instance_name.clone())
        .await?;

    let schema_provider = Arc::new(KubernetesSchemaProvider::new(current_backend.get()));
    let action_repository = current_backend.get();
    let resource_repository = current_backend.get();
    let policy_repository = current_backend.get();
    let audit_service = Arc::new(LogAuditService::new());
    let cedar_validation_service = Arc::new(CedarValidationService::new(
        schema_provider,
        action_repository,
        resource_repository,
        policy_repository,
        audit_service.clone(),
    ));

    let action_repository: Arc<ActionDataRepository> = current_backend.get();
    let resource_repository: Arc<ResourceDiscoveryDocumentRepository> = current_backend.get();
    let policy_repository: Arc<PolicyDataRepository> = current_backend.get();

    let production_mode = !std::env::var("BOXER_ISSUER_DEBUG").is_ok();

    let schema_repository: Arc<SchemaRepository> = current_backend.get();

    info!("listening on {}:{}", &cm.listen_address.ip(), &cm.listen_address.port());
    let authorizer = Arc::new(Authorizer::new(cm.get_signatures()?, cm.token_settings));
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
            .service(v1::urls(production_mode, authorizer.clone(), audit_service.clone()))
            .service(SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", ApiDoc::openapi()))
    })
    .bind(cm.listen_address)?
    .run()
    .await
    .map_err(anyhow::Error::from)
}
