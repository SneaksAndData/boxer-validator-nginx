use crate::services::repositories::common::TrieRepositoryData;
use crate::services::repositories::models::PathSegment;
use std::sync::Arc;

pub fn new() -> Arc<TrieRepositoryData<PathSegment>> {
    Arc::new(TrieRepositoryData::new())
}
