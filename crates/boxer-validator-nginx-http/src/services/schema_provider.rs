use crate::services::base::schema_provider::SchemaProvider;
use anyhow::{Error, Result};
use async_trait::async_trait;
use boxer_core::contracts::internal_token::v1::boxer_claims::BoxerClaims;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use cedar_policy::Schema;
use log::debug;
use std::sync::Arc;

pub struct KubernetesSchemaProvider {
    schema_repository: Arc<SchemaRepository>,
}

#[async_trait]
impl SchemaProvider for KubernetesSchemaProvider {
    async fn get_schema(&self, boxer_claims: &BoxerClaims) -> Result<Schema> {
        let actions_schema = self
            .schema_repository
            .get(boxer_claims.validator_schema_id.clone())
            .await
            .map_err(Error::from)?;
        let principal_schema = boxer_claims.schema.clone();
        debug!("Kubernetes schema actions: {:?}", actions_schema.to_json_string());
        debug!("Kubernetes schema principal: {:?}", principal_schema.to_json_string());
        Schema::from_schema_fragments(vec![actions_schema, principal_schema]).map_err(anyhow::Error::from)
    }
}

impl KubernetesSchemaProvider {
    pub fn new(schema_repository: Arc<SchemaRepository>) -> Self {
        KubernetesSchemaProvider { schema_repository }
    }
}
