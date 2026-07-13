use anyhow::{Error, Result};
use async_trait::async_trait;
use boxer_core::contracts::internal_token::v2::boxer_claims::BoxerClaims;
use boxer_core::services::backends::kubernetes::kubernetes_repository::schema_repository::SchemaRepository;
use boxer_core::services::validation_service::required_claims::RequiredClaims;
use boxer_core::services::validation_service::schema_provider::SchemaProvider;
use cedar_policy::Schema;
use log::debug;
use std::sync::Arc;

pub struct KubernetesSchemaProvider {
    schema_repository: Arc<SchemaRepository>,
}

#[async_trait]
impl SchemaProvider<BoxerClaims> for KubernetesSchemaProvider {
    async fn get_schema(&self, boxer_claims: &BoxerClaims) -> Result<Schema> {
        let actions_schema = self
            .schema_repository
            .get(boxer_claims.get_validator_schema_id().clone())
            .await
            .map_err(Error::from)?;
        let principal_schema = boxer_claims.get_schema().clone();
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
