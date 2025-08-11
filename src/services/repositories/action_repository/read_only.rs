use crate::services::repositories::lookup_trie::TrieRepositoryData;
use crate::services::repositories::models::request_segment::RequestSegment;
use boxer_core::services::base::upsert_repository::UpsertRepository;
use std::sync::Arc;

#[cfg(test)]
mod tests;

pub fn new() -> Arc<TrieRepositoryData<RequestSegment>> {
    Arc::new(TrieRepositoryData::new())
}
