use boxer_core::services::backends::{Backend, SchemaRepositorySource};
use boxer_core::services::base::types::SchemaRepository;
use std::sync::Arc;

pub struct KubernetesBackend {
    pub schema_repository: Arc<SchemaRepository>,
}

impl SchemaRepositorySource for KubernetesBackend {
    fn get_schemas_repository(&self) -> Arc<SchemaRepository> {
        self.schema_repository.clone()
    }
}

impl Backend for KubernetesBackend {
    // This is marker trait, so no methods are required here
}
