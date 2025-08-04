mod configuration;

use crate::services::base::actions_repository_source::ActionRepositorySource;
use crate::services::base::policy_repository_source::PolicyRepositorySource;
use crate::services::base::resource_repository_source::ResourceRepositorySource;
use crate::services::repositories::action_repository::{ActionReadOnlyRepository, ActionRepository};
use crate::services::repositories::backend::ReadOnlyRepositoryBackend;
use crate::services::repositories::policy_repository::PolicyRepository;
use crate::services::repositories::resource_repository::ResourceRepository;
use boxer_core::services::backends::{Backend, SchemaRepositorySource};
use boxer_core::services::base::types::SchemaRepository;
use std::sync::Arc;

pub struct KubernetesBackend {
    schema_repository: Arc<SchemaRepository>,
    action_readonly_repository: Arc<ActionReadOnlyRepository>,
    action_data_repository: Arc<ActionRepository>,

    resource_repository: Arc<ResourceRepository>,
    policy_repository: Arc<PolicyRepository>,

    // This field is required since we want to hold the reference to the backend until
    // the backend is dropped.
    #[allow(dead_code)]
    action_lookup_watcher: Arc<ReadOnlyRepositoryBackend>,
    action_repository_watcher: Arc<ReadOnlyRepositoryBackend>,
    #[allow(dead_code)]
    resource_repository_backend: Arc<ReadOnlyRepositoryBackend>,
    #[allow(dead_code)]
    policy_repository_backend: Arc<ReadOnlyRepositoryBackend>,
}

impl SchemaRepositorySource for KubernetesBackend {
    fn get_schemas_repository(&self) -> Arc<SchemaRepository> {
        self.schema_repository.clone()
    }
}

impl ActionRepositorySource for KubernetesBackend {
    fn get_readonly_repository(&self) -> Arc<ActionReadOnlyRepository> {
        self.action_readonly_repository.clone()
    }

    fn get_action_data_repository(&self) -> Arc<ActionRepository> {
        self.action_data_repository.clone()
    }
}

impl ResourceRepositorySource for KubernetesBackend {
    fn get_resource_repository(&self) -> Arc<ResourceRepository> {
        self.resource_repository.clone()
    }
}

impl PolicyRepositorySource for KubernetesBackend {
    fn get_policy_repository(&self) -> Arc<PolicyRepository> {
        self.policy_repository.clone()
    }
}

impl Backend for KubernetesBackend {
    // This is marker trait, so no methods are required here
}
