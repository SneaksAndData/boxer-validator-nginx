use crate::services::repositories::common::TrieRepositoryData;
use crate::services::repositories::models::PathSegment;

pub fn new() -> TrieRepositoryData<PathSegment> {
    TrieRepositoryData::new()
}
