pub mod resource_discovery_document;

pub mod read_write;

#[cfg(test)]
mod tests;

use crate::services::prefix_tree::bucket::path_segment_bucket::PathSegmentBucket;
use crate::services::repositories::lookup_trie::schema_bound_trie_repository::SchemaBoundedTrieRepositoryData;
use crate::services::repositories::models::path_segment::PathSegment;
use cedar_policy::EntityUid;

pub type ResourceReadOnlyRepository = SchemaBoundedTrieRepositoryData<PathSegment, PathSegmentBucket<EntityUid>>;
