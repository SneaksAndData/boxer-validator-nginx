use crate::models::boxer_claims::v1::boxer_claims::BoxerClaims;
use crate::services::base::schema_provider::SchemaProvider;
use anyhow::Result;
use async_trait::async_trait;
use boxer_core::services::base::types::SchemaRepository;
use cedar_policy::Schema;
use std::sync::Arc;

pub struct KubernetesSchemaProvider {
    #[allow(dead_code)]
    schema_repository: Arc<SchemaRepository>,
}

#[async_trait]
impl SchemaProvider for KubernetesSchemaProvider {
    async fn get_schema(&self, _boxer_claims: &BoxerClaims) -> Result<Schema> {
        // Here you would implement the logic to retrieve the schema based on the boxer_claims.
        // For demonstration purposes, we return a dummy schema.
        Schema::from_json_str("").map_err(|e| e.into())
    }
}

impl KubernetesSchemaProvider {
    pub fn new(schema_repository: Arc<SchemaRepository>) -> Self {
        KubernetesSchemaProvider { schema_repository }
    }
}
