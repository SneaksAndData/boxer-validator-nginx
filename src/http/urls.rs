use crate::models::boxer_claims::v1::boxer_claims::BoxerClaims;
use crate::models::request_context::RequestContext;
use actix_web::get;
use log::debug;

// Dummy implementation of the token endpoint
#[get("/token/review")]
async fn token_review(boxer_claims: BoxerClaims, request_context: RequestContext) -> String {
    debug!("Boxer claims: {:?}", boxer_claims);
    debug!("Request context: {:?}", request_context);
    "dummy endpoint".to_string()
}
