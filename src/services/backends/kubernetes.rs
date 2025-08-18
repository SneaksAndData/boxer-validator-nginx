mod configuration;

use crate::services::repositories::action_repository::action_discovery_document::ActionDiscoveryDocument;
use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::repositories::action_repository::ActionReadOnlyRepository;
use crate::services::repositories::lookup_trie::backend::ReadOnlyRepositoryBackend;
use crate::services::repositories::policy_repository::policy_document::PolicyDocument;
use crate::services::repositories::policy_repository::read_only::PolicyRepositoryData;
use crate::services::repositories::policy_repository::read_write::PolicyDataRepository;
use crate::services::repositories::policy_repository::PolicyReadOnlyRepository;
use crate::services::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
use crate::services::repositories::resource_repository::resource_discovery_document::ResourceDiscoveryDocument;
use crate::services::repositories::resource_repository::ResourceReadOnlyRepository;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use boxer_core::services::backends::Backend;
use boxer_core::services::service_provider::ServiceProvider;
use std::sync::Arc;

pub struct KubernetesBackend {
    schema_repository: Arc<SchemaRepository>,
    action_repository: Arc<ActionDataRepository>,
    resource_repository: Arc<ResourceDiscoveryDocumentRepository>,
    policy_repository: Arc<PolicyDataRepository>,

    action_lookup_table_listener: Arc<ReadOnlyRepositoryBackend<ActionReadOnlyRepository, ActionDiscoveryDocument>>,
    resource_lookup_table_listener:
        Arc<ReadOnlyRepositoryBackend<ResourceReadOnlyRepository, ResourceDiscoveryDocument>>,
    policy_lookup_watcher: Arc<ReadOnlyRepositoryBackend<PolicyRepositoryData, PolicyDocument>>,
}

impl ServiceProvider<Arc<SchemaRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<SchemaRepository> {
        self.schema_repository.clone()
    }
}

impl ServiceProvider<Arc<ActionReadOnlyRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<ActionReadOnlyRepository> {
        self.action_lookup_table_listener.get().clone()
    }
}

impl ServiceProvider<Arc<ActionDataRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<ActionDataRepository> {
        self.action_repository.clone()
    }
}

impl ServiceProvider<Arc<ResourceDiscoveryDocumentRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<ResourceDiscoveryDocumentRepository> {
        self.resource_repository.clone()
    }
}

impl ServiceProvider<Arc<ResourceReadOnlyRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<ResourceReadOnlyRepository> {
        self.resource_lookup_table_listener.get().clone()
    }
}

impl ServiceProvider<Arc<PolicyReadOnlyRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<PolicyReadOnlyRepository> {
        self.policy_lookup_watcher.get().clone()
    }
}

impl ServiceProvider<Arc<PolicyDataRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<PolicyDataRepository> {
        self.policy_repository.clone()
    }
}

impl Backend for KubernetesBackend {
    // This is marker trait, so no methods are required here
}
