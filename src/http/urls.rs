use actix_web::dev::WebService;
use crate::models::boxer_claims::v1::boxer_claims::BoxerClaims;
use actix_web::{get, web};
use log::debug;
use crate::services::validation_service::{CedarValidationService, ValidationService};

// Dummy implementation of the token endpoint
#[get("/token/review")]
async fn token_review(boxer_claims: BoxerClaims, cedar_validation_service: web::Data<CedarValidationService>) -> String {
    debug!("Boxer claims: {:?}", boxer_claims);;
    cedar_validation_service.validate_token("").unwrap();
    "dummy endpoint".to_string()
}
