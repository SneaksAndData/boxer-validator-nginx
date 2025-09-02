use crate::models::request_context::RequestContext;
use crate::services::base::validation_service::ValidationService;
use crate::services::cedar_validation_service::CedarValidationService;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{get, HttpResponse};
use boxer_core::contracts::internal_token::v1::boxer_claims::BoxerClaims;
use log::info;
use std::sync::Arc;

#[utoipa::path(
    context_path = "",
    responses((status = OK)),
    security(
        ("internal" = [])
    )
)]
#[get("/review")]
async fn token_review(
    boxer_claims: BoxerClaims,
    request_context: RequestContext,
    cedar_validation_service: Data<Arc<CedarValidationService>>,
) -> HttpResponse {
    let status_code = match cedar_validation_service.validate(boxer_claims, request_context).await {
        Ok(_) => {
            info!("Token validated successfully");
            StatusCode::OK
        }
        Err(e) => {
            info!("Failed to validate token: {:?}", e);
            StatusCode::UNAUTHORIZED
        }
    };
    HttpResponse::build(status_code).finish()
}
