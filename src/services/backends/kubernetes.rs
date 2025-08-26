mod configuration;

use crate::services::repositories::action_repository::action_discovery_document::ActionDiscoveryDocument;
use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::repositories::action_repository::ActionReadOnlyRepository;
use crate::services::repositories::lookup_trie::backend::{AssociatedRepository, ReadOnlyRepositoryBackend};
use crate::services::repositories::models::path_segment::PathSegment;
use crate::services::repositories::models::request_segment::RequestSegment;
use crate::services::repositories::policy_repository::policy_document::PolicyDocument;
use crate::services::repositories::policy_repository::read_only::PolicyRepositoryData;
use crate::services::repositories::policy_repository::read_write::PolicyDataRepository;
use crate::services::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
use crate::services::repositories::resource_repository::resource_discovery_document::ResourceDiscoveryDocument;
use crate::services::repositories::resource_repository::ResourceReadOnlyRepository;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::SchemaRepository;
use boxer_core::services::backends::Backend;
use boxer_core::services::service_provider::ServiceProvider;
use cedar_policy::{EntityUid, PolicySet};
use std::sync::Arc;

pub struct KubernetesBackend {
    schema_repository: Arc<SchemaRepository>,
    action_repository: Arc<ActionDataRepository>,
    resource_repository: Arc<ResourceDiscoveryDocumentRepository>,
    policy_repository: Arc<PolicyDataRepository>,

    action_lookup_table_listener: Arc<
        ReadOnlyRepositoryBackend<
            ActionReadOnlyRepository,
            ActionDiscoveryDocument,
            (String, Vec<RequestSegment>),
            EntityUid,
        >,
    >,
    resource_lookup_table_listener: Arc<
        ReadOnlyRepositoryBackend<
            ResourceReadOnlyRepository,
            ResourceDiscoveryDocument,
            (String, Vec<PathSegment>),
            EntityUid,
        >,
    >,
    policy_lookup_watcher: Arc<ReadOnlyRepositoryBackend<PolicyRepositoryData, PolicyDocument, String, PolicySet>>,
}

impl ServiceProvider<Arc<SchemaRepository>> for KubernetesBackend {
    fn get(&self) -> Arc<SchemaRepository> {
        self.schema_repository.clone()
    }
}

impl ServiceProvider<Arc<AssociatedRepository<(String, Vec<RequestSegment>), EntityUid>>> for KubernetesBackend {
    fn get(&self) -> Arc<AssociatedRepository<(String, Vec<RequestSegment>), EntityUid>> {
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

impl ServiceProvider<Arc<AssociatedRepository<(String, Vec<PathSegment>), EntityUid>>> for KubernetesBackend {
    fn get(&self) -> Arc<AssociatedRepository<(String, Vec<PathSegment>), EntityUid>> {
        self.resource_lookup_table_listener.get().clone()
    }
}

impl ServiceProvider<Arc<AssociatedRepository<String, PolicySet>>> for KubernetesBackend {
    fn get(&self) -> Arc<AssociatedRepository<String, PolicySet>> {
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
