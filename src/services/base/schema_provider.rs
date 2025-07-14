use crate::models::boxer_claims::v1::boxer_claims::BoxerClaims;
use anyhow::Result;
use async_trait::async_trait;
use cedar_policy::Schema;

#[async_trait]
pub trait SchemaProvider: Send + Sync {
    #[allow(dead_code)]
    async fn get_schema(&self, boxer_claims: &BoxerClaims) -> Result<Schema>;
}
