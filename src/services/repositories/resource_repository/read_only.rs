use crate::services::repositories::lookup_trie::TrieRepositoryData;
use crate::services::repositories::models::path_segment::PathSegment;
use std::sync::Arc;

pub fn new() -> Arc<TrieRepositoryData<PathSegment>> {
    Arc::new(TrieRepositoryData::new())
}
