use anyhow::Result;
use async_trait::async_trait;
use boxer_core::contracts::internal_token::v1::boxer_claims::BoxerClaims;
use cedar_policy::Schema;

#[async_trait]
pub trait SchemaProvider: Send + Sync {
    async fn get_schema(&self, boxer_claims: &BoxerClaims) -> Result<Schema>;
}
