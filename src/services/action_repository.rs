// Use log crate when building application
#[cfg(not(test))]
use log::{debug, warn};

// Workaround to use prinltn! for logs.
#[cfg(test)]
use std::{println as warn, println as debug};

#[cfg(test)]
mod tests;

pub mod kubernetes_action_repository_backend;
pub mod models;

use crate::services::action_repository::kubernetes_action_repository_backend::{
    ActionDiscoveryDocument, ActionDiscoveryResource,
};
use crate::services::action_repository::models::RequestSegment;
use anyhow::anyhow;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::{ReadOnlyRepository, UpsertRepository};
use cedar_policy::EntityUid;
use futures::stream::StreamExt;
use kube::runtime::watcher;
use std::future::Future;
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

pub struct ActionData {
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

impl ResourceUpdateHandler<ActionDiscoveryResource> for ActionData {
    fn handle_update(&self, event: Result<ActionDiscoveryResource, watcher::Error>) -> impl Future<Output = ()> + Send {
        async {
            if event.is_err() {
                warn!("Failed to handle update: {:?}", event);
            }
            self.handle_async(event.unwrap()).await
        }
    }
}

impl ActionData {
    pub fn new() -> Arc<Self> {
        Arc::new(ActionData {
            rw_lock: RwLock::new(TrieData {
                builder: Box::new(TrieBuilder::new()),
                maybe_trie: None,
            }),
        })
    }
    pub async fn handle_async(&self, event: ActionDiscoveryResource) {
        let doc: ActionDiscoveryDocument = serde_json::from_str(&event.data.actions).unwrap();
        doc.stream()
            .for_each(move |result| async move {
                match result {
                    Ok((segments, action_uid)) => {
                        if let Err(e) = self.upsert(segments.clone(), action_uid.clone()).await {
                            warn!("Failed to upsert action: {}", e);
                        } else {
                            debug!(
                                "Successfully upserted action with key {:?} and UID: {}",
                                segments, action_uid
                            );
                        }
                    }
                    Err(e) => warn!("Error processing action route: {}", e),
                }
            })
            .await
    }
}
