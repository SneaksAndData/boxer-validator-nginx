mod configuration;

use crate::services::action_repository::backend::ActionRepositoryBackend;
use crate::services::action_repository::ActionRepository;
use crate::services::base::actions_repository_source::ActionRepositorySource;
use boxer_core::services::backends::{Backend, SchemaRepositorySource};
use boxer_core::services::base::types::SchemaRepository;
use std::sync::Arc;

pub struct KubernetesBackend {
    schema_repository: Arc<SchemaRepository>,
    action_repository: Arc<dyn ActionRepository>,

    // This field is required since we want to hold the reference to the backend until
    // the backend is dropped.
    #[allow(dead_code)]
    action_repository_backend: Arc<ActionRepositoryBackend>,
}

impl SchemaRepositorySource for KubernetesBackend {
    fn get_schemas_repository(&self) -> Arc<SchemaRepository> {
        self.schema_repository.clone()
    }
}

impl ActionRepositorySource for KubernetesBackend {
    fn get_actions_repository(&self) -> Arc<dyn ActionRepository> {
        self.action_repository.clone()
    }
}

impl Backend for KubernetesBackend {
    // This is marker trait, so no methods are required here
}
