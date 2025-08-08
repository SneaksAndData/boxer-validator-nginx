mod configuration;

use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::repositories::action_repository::ActionReadOnlyRepository;
use crate::services::repositories::lookup_trie::backend::ReadOnlyRepositoryBackend;
use crate::services::repositories::policy_repository::read_write::PolicyDataRepository;
use crate::services::repositories::policy_repository::PolicyReadOnlyRepository;
use crate::services::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
use crate::services::repositories::resource_repository::ResourceReadOnlyRepository;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use boxer_core::services::backends::Backend;
use boxer_core::services::service_provider::ServiceProvider;
use std::sync::Arc;

pub struct KubernetesBackend {
    schema_repository: Arc<SchemaRepository>,
    action_readonly_repository: Arc<ActionReadOnlyRepository>,
    action_data_repository: Arc<ActionDataRepository>,

    resource_read_only_repository: Arc<ResourceReadOnlyRepository>,
    resource_data_repository: Arc<ResourceDiscoveryDocumentRepository>,

    policy_repository: Arc<PolicyReadOnlyRepository>,
    policy_data_repository: Arc<PolicyDataRepository>,

    // This field is required since we want to hold the reference to the backend until
    // the backend is dropped.
    #[allow(dead_code)]
    action_lookup_watcher: Arc<ReadOnlyRepositoryBackend>,
    #[allow(dead_code)]
    action_repository_watcher: Arc<ReadOnlyRepositoryBackend>,
    #[allow(dead_code)]
    resource_lookup_watcher: Arc<ReadOnlyRepositoryBackend>,
    #[allow(dead_code)]
    resource_repository_watcher: Arc<ReadOnlyRepositoryBackend>,
    #[allow(dead_code)]
    policy_lookup_watcher: Arc<ReadOnlyRepositoryBackend>,
    #[allow(dead_code)]
    policy_repository_watcher: Arc<ReadOnlyRepositoryBackend>,
}

impl ServiceProvider<Arc<SchemaRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<SchemaRepository> {
        self.schema_repository.clone()
    }
}

impl ServiceProvider<Arc<ActionReadOnlyRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<ActionReadOnlyRepository> {
        self.action_readonly_repository.clone()
    }
}

impl ServiceProvider<Arc<ActionDataRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<ActionDataRepository> {
        self.action_data_repository.clone()
    }
}

impl ServiceProvider<Arc<ResourceDiscoveryDocumentRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<ResourceDiscoveryDocumentRepository> {
        self.resource_data_repository.clone()
    }
}

impl ServiceProvider<Arc<ResourceReadOnlyRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<ResourceReadOnlyRepository> {
        self.resource_read_only_repository.clone()
    }
}

impl ServiceProvider<Arc<PolicyReadOnlyRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<PolicyReadOnlyRepository> {
        self.policy_repository.clone()
    }
}

impl Backend for KubernetesBackend {
    // This is marker trait, so no methods are required here
}
