use super::ActionReadOnlyRepositoryInterface;
use crate::services::repositories::lookup_trie::TrieRepositoryData;
use crate::services::repositories::models::request_segment::RequestSegment;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::{ReadOnlyRepository, UpsertRepository};
use futures::stream::StreamExt;
use std::sync::Arc;

#[cfg(test)]
mod tests;

pub fn new() -> Arc<TrieRepositoryData<RequestSegment>> {
    Arc::new(TrieRepositoryData::new())
}
