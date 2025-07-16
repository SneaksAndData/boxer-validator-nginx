#[cfg(test)]
mod tests;

pub mod kubernetes_action_repository_backend;
pub mod models;

use crate::services::action_repository::models::RequestSegment;
use anyhow::anyhow;
use async_trait::async_trait;
use boxer_core::services::base::upsert_repository::{ReadOnlyRepository, UpsertRepository};
use cedar_policy::EntityUid;
use std::sync::Arc;
use tokio::sync::RwLock;
use trie_rs::map::{Trie, TrieBuilder};

pub type ActionReadOnlyRepository = dyn ReadOnlyRepository<Vec<RequestSegment>, EntityUid, ReadError = anyhow::Error>;

pub trait ActionRepository:
    ReadOnlyRepository<Vec<RequestSegment>, EntityUid, ReadError = anyhow::Error>
    + UpsertRepository<Vec<RequestSegment>, EntityUid, Error = anyhow::Error>
{
}

pub fn new() -> Arc<dyn ActionRepository> {
    Arc::new(ActionData {
        rw_lock: RwLock::new(TrieData {
            builder: Box::new(TrieBuilder::new()),
            maybe_trie: None,
        }),
    })
}

struct TrieData {
    builder: Box<TrieBuilder<RequestSegment, EntityUid>>,
    maybe_trie: Option<Arc<Trie<RequestSegment, EntityUid>>>,
}

struct ActionData {
    rw_lock: RwLock<TrieData>,
}

#[async_trait]
impl ReadOnlyRepository<Vec<RequestSegment>, EntityUid> for ActionData {
    type ReadError = anyhow::Error;

    async fn get(&self, key: Vec<RequestSegment>) -> Result<EntityUid, Self::ReadError> {
        let guard = self.rw_lock.read().await;
        guard
            .maybe_trie
            .clone()
            .and_then(|trie| trie.exact_match(&key).map(|e| e.clone()))
            .ok_or(anyhow!("Entity not found: {:?}", key))
    }
}

#[async_trait]
impl UpsertRepository<Vec<RequestSegment>, EntityUid> for ActionData {
    type Error = anyhow::Error;

    async fn upsert(&self, key: Vec<RequestSegment>, entity: EntityUid) -> Result<(), Self::Error> {
        let mut guard = self.rw_lock.write().await;
        let mut builder = guard.builder.clone();
        builder.push(key, entity.clone());
        guard.builder = builder.clone();
        guard.maybe_trie = Some(Arc::new(builder.build()));
        Ok(())
    }

    async fn exists(&self, key: Vec<RequestSegment>) -> Result<bool, Self::Error> {
        let guard = self.rw_lock.read().await;
        Ok(guard
            .maybe_trie
            .clone()
            .and_then(|trie| trie.exact_match(&key).map(|e| e.clone()))
            .is_some())
    }
}

impl ActionRepository for ActionData {}
