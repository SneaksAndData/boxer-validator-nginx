pub mod http;
pub mod models;
pub mod services;

use crate::http::controllers::v1;
use crate::services::authorizer::Authorizer;
use crate::services::configuration::models::AppSettings;
use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::repositories::policy_repository::read_write::PolicyDataRepository;
use crate::services::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
use crate::services::schema_provider::KubernetesSchemaProvider;
use actix_web::dev::Server;
use actix_web::middleware::{Logger, from_fn};
use actix_web::{App, HttpServer, web};
use boxer_core::contracts::internal_token::v1::boxer_claims::BoxerClaims;
use boxer_core::http::middleware::logging::custom_error_logging;
use boxer_core::services::audit::log_audit_service::LogAuditService;
use boxer_core::services::backends::kubernetes::kubernetes_repository::schema_repository::SchemaRepository;
use boxer_core::services::observability::open_telemetry::metrics::provider::MetricsProvider;
use boxer_core::services::service_provider::ServiceProvider;
use boxer_core::services::validation_service::cedar_validation_service::CedarValidationService;
use boxer_core::services::validation_service::schema_provider::SchemaProvider;
use http::openapi::ApiDoc;
use log::info;
use opentelemetry_instrumentation_actix_web::RequestTracing;
use services::backends::kubernetes::KubernetesBackend;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn start_api_server(
    current_backend: Arc<KubernetesBackend>,
    app_settings: AppSettings,
    root_metrics_namespace: &'static str,
) -> Result<Server, anyhow::Error> {
    let schema_provider: Arc<dyn SchemaProvider<BoxerClaims>> =
        Arc::new(KubernetesSchemaProvider::new(current_backend.get()));
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
        MetricsProvider::new(root_metrics_namespace, app_settings.instance_name.clone()),
    ));

    let action_repository: Arc<ActionDataRepository> = current_backend.get();
    let resource_repository: Arc<ResourceDiscoveryDocumentRepository> = current_backend.get();
    let policy_repository: Arc<PolicyDataRepository> = current_backend.get();

    let production_mode = !std::env::var("BOXER_ISSUER_DEBUG").is_ok();

    let schema_repository: Arc<SchemaRepository> = current_backend.get();

    info!(
        "listening on {}:{}",
        &app_settings.listen_address.ip(),
        &app_settings.listen_address.port()
    );
    let authorizer = Arc::new(Authorizer::new(
        app_settings.get_signatures()?,
        app_settings.token_settings,
    ));
    let http_server_builder = HttpServer::new(move || {
        App::new()
            .wrap(RequestTracing::new())
            .wrap(Logger::default())
            .wrap(from_fn(custom_error_logging))
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
    .bind(app_settings.listen_address)?;

    Ok(http_server_builder.run())
}
