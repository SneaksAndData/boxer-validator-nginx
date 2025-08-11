pub mod resource_discovery_document;

pub mod read_write;

use crate::http::controllers::resource_set::models::ResourceSetRegistration;
use crate::services::repositories::lookup_trie::TrieRepositoryData;
use crate::services::repositories::models::path_segment::PathSegment;
use crate::services::repositories::resource_repository::resource_discovery_document::ResourceDiscoveryDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::{CanDelete, ReadOnlyRepository, UpsertRepository};
use cedar_policy::EntityUid;

pub trait ResourceReadOnlyRepositoryInterface:
    ReadOnlyRepository<Vec<PathSegment>, EntityUid, ReadError = anyhow::Error>
    + ResourceUpdateHandler<ResourceDiscoveryDocument>
{
}

pub trait ResourceRepositoryInterface:
    ReadOnlyRepository<String, ResourceSetRegistration, ReadError = anyhow::Error>
    + UpsertRepository<String, ResourceSetRegistration, Error = anyhow::Error>
    + CanDelete<String, ResourceSetRegistration, DeleteError = anyhow::Error>
{
}

impl ResourceReadOnlyRepositoryInterface for TrieRepositoryData<PathSegment> {}

pub type ResourceReadOnlyRepository = dyn ResourceReadOnlyRepositoryInterface;

pub type ResourceRepository = dyn ResourceRepositoryInterface + Send + Sync;
