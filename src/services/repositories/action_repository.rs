pub mod action_discovery_document;
pub mod read_write;

use crate::services::repositories::action_repository::action_discovery_document::ActionDiscoveryDocument;
use crate::services::repositories::lookup_trie::TrieRepositoryData;
use crate::services::repositories::models::request_segment::RequestSegment;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::ReadOnlyRepository;
use cedar_policy::EntityUid;

pub trait ActionReadOnlyRepositoryInterface:
    ReadOnlyRepository<Vec<RequestSegment>, EntityUid, ReadError = anyhow::Error>
    + ResourceUpdateHandler<ActionDiscoveryDocument>
{
}

impl ActionReadOnlyRepositoryInterface for TrieRepositoryData<RequestSegment> {}

pub type ActionReadOnlyRepository = dyn ActionReadOnlyRepositoryInterface;
