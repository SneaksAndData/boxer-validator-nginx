use crate::models::request_context::RequestContext;
use async_trait::async_trait;
use boxer_core::contracts::internal_token::v1::boxer_claims::BoxerClaims;

#[async_trait]
pub trait ValidationService {
    async fn validate(&self, boxer_claims: BoxerClaims, request_context: RequestContext) -> Result<(), anyhow::Error>;
}
