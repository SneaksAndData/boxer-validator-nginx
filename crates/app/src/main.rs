use anyhow::Result;
use boxer_core::services::backends::BackendConfiguration;
use boxer_core::services::observability::composed_logger::ComposedLogger;
use boxer_core::services::observability::open_telemetry;
use boxer_core::services::observability::open_telemetry::metrics::init_metrics;
use boxer_core::services::observability::open_telemetry::tracing::init_tracer;
use boxer_validator_nginx_http::services::backends;
use boxer_validator_nginx_http::services::configuration::models::AppSettings;
use env_filter::Builder;
use log::info;

const ROOT_METRICS_NAMESPACE: &str = "boxer-validator";

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
            logger.with_logger(open_telemetry::logging::init_logger(cm.deploy_environment.clone())?)
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

    let server = boxer_validator_nginx_http::start_api_server(current_backend, cm, ROOT_METRICS_NAMESPACE)?;

    server.await.map_err(anyhow::Error::from)
}
