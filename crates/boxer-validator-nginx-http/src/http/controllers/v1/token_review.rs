use crate::http::filters::jwt_filter::InternalTokenMiddlewareFactory;
use crate::services::authorizer::Authorizer;
use actix_web::http::StatusCode;
use actix_web::middleware::Condition;
use actix_web::web::Data;
use actix_web::{get, web, HttpResponse};
use boxer_core::contracts::internal_token::v1::boxer_claims::BoxerClaims;
use boxer_core::services::audit::AuditService;
use boxer_core::services::validation_service::request_context::RequestContext;
use boxer_core::services::validation_service::ValidationService;
use log::{error, info};
use std::sync::Arc;

#[utoipa::path(
    context_path = "/token",
    responses((status = OK)),
    security(
        ("internal" = [])
    )
)]
#[get("/review")]
async fn token_review(
    boxer_claims: BoxerClaims,
    request_context: RequestContext,
    cedar_validation_service: Data<Arc<dyn ValidationService<BoxerClaims>>>,
) -> HttpResponse {
    let status_code = match cedar_validation_service.validate(boxer_claims, request_context).await {
        Ok(_) => {
            info!("Token validated successfully");
            StatusCode::OK
        }
        Err(e) => {
            error!("Failed to validate token: {:?}", e);
            StatusCode::UNAUTHORIZED
        }
    };
    HttpResponse::build(status_code).finish()
}

pub fn routes(
    production_mode: bool,
    authorizer: Arc<Authorizer>,
    audit_service: Arc<dyn AuditService>,
) -> impl actix_web::dev::HttpServiceFactory {
    let middleware = InternalTokenMiddlewareFactory::new(authorizer, audit_service.clone());
    web::scope("/token")
        .wrap(Condition::new(production_mode, middleware))
        .service(token_review)
}
