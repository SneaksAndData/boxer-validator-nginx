use crate::models::boxer_claims::v1::boxer_claims::BoxerClaims;
use actix_web::get;
use log::debug;

// Dummy implementation of the token endpoint
#[get("/token/review")]
async fn token_review(boxer_claims: BoxerClaims) -> String {
    debug!("Boxer claims: {:?}", boxer_claims);
    "dummy endpoint".to_string()
}
