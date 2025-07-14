use crate::models::boxer_claims::v1::boxer_claims::BoxerClaims;
use crate::services::base::schema_provider::SchemaProvider;
use anyhow::Result;
use async_trait::async_trait;
use boxer_core::services::base::types::SchemaRepository;
use cedar_policy::{Schema, SchemaFragment};
use std::sync::Arc;

const SCHEMA_LOCATION: &str = "boxer-validator-schema";

pub struct KubernetesSchemaProvider {
    #[allow(dead_code)]
    schema_repository: Arc<SchemaRepository>,
}

#[async_trait]
impl SchemaProvider for KubernetesSchemaProvider {
    async fn get_schema(&self, boxer_claims: &BoxerClaims) -> Result<Schema> {
        let actions_schema = self.schema_repository.get(SCHEMA_LOCATION.to_string()).await?;
        let principal_schema = SchemaFragment::from_json_str(&boxer_claims.schema)?;
        Schema::from_schema_fragments(vec![actions_schema, principal_schema]).map_err(anyhow::Error::from)
    }
}

impl KubernetesSchemaProvider {
    pub fn new(schema_repository: Arc<SchemaRepository>) -> Self {
        KubernetesSchemaProvider { schema_repository }
    }
}
