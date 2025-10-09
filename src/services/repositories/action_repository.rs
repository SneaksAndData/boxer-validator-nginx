pub mod action_discovery_document;
pub mod read_write;

#[cfg(test)]
mod tests;

use crate::services::prefix_tree::bucket::request_segment_bucket::RequestBucket;
use crate::services::repositories::lookup_trie::schema_bound_trie_repository::SchemaBoundedTrieRepositoryData;
use crate::services::repositories::models::request_segment::RequestSegment;
use cedar_policy::EntityUid;

pub type ActionReadOnlyRepository = SchemaBoundedTrieRepositoryData<RequestSegment, RequestBucket<EntityUid>>;
