pub mod resource_discovery_document;

pub mod read_write;

#[cfg(test)]
mod tests;

use crate::services::repositories::lookup_trie::TrieRepositoryData;
use crate::services::repositories::models::path_segment::PathSegment;
use crate::services::repositories::resource_repository::resource_discovery_document::ResourceDiscoveryDocument;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::ReadOnlyRepository;
use cedar_policy::EntityUid;

pub trait ResourceReadOnlyRepositoryInterface:
    ReadOnlyRepository<Vec<PathSegment>, EntityUid, ReadError = anyhow::Error>
    + ResourceUpdateHandler<ResourceDiscoveryDocument>
{
}

impl ResourceReadOnlyRepositoryInterface for TrieRepositoryData<PathSegment> {}

pub type ResourceReadOnlyRepository = dyn ResourceReadOnlyRepositoryInterface;
