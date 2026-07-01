pub mod resource_discovery_document;

pub mod read_write;

#[cfg(test)]
mod tests;

use crate::services::repositories::lookup_trie::schema_bound_trie_repository::SchemaBoundedTrieRepositoryData;
use crate::services::repositories::models::path_segment::PathSegment;

pub type ResourceReadOnlyRepository = SchemaBoundedTrieRepositoryData<PathSegment>;
