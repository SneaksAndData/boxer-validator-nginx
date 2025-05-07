use crate::models::boxer_claims::v1::boxer_claims::BoxerClaims;
use crate::models::request_context::RequestContext;
use crate::services::cedar_validation_service::CedarValidationService;
use crate::services::base::validation_service::ValidationService;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{get, HttpResponse};
use log::debug;

#[get("/token/review")]
async fn token_review(
    boxer_claims: BoxerClaims,
    request_context: RequestContext,
    cedar_validation_service: Data<Box<CedarValidationService>>,
) -> HttpResponse {
    let status_code = match (*cedar_validation_service).validate(boxer_claims, request_context) {
        Ok(_) => {
            debug!("Token validated successfully");
            StatusCode::OK
        }
        Err(e) => {
            debug!("Failed to validate token: {:?}", e);
            StatusCode::UNAUTHORIZED
        }
    };
    HttpResponse::build(status_code).finish()
}
