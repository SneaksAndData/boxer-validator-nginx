#[cfg(not(test))]
use log::{debug, info, warn};

#[cfg(test)]
use std::{println as warn, println as debug, println as info};

use anyhow::anyhow;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::{CanDelete, ReadOnlyRepository, UpsertRepository};
use cedar_policy::EntityUid;
use futures::StreamExt;
use kube::runtime::watcher::Error;
use kube::Resource;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;
use trie_rs::map::Trie;

pub mod backend;

pub struct TrieData<Key> {
    items: HashMap<Vec<Key>, EntityUid>,
    maybe_trie: Option<Arc<Trie<Key, EntityUid>>>,
}

pub struct TrieRepositoryData<Key> {
    pub rw_lock: RwLock<TrieData<Key>>,
}

impl<Key> TrieRepositoryData<Key>
where
    Key: Ord,
{
    pub fn new() -> Self {
        TrieRepositoryData {
            rw_lock: RwLock::new(TrieData {
                items: HashMap::new(),
                maybe_trie: None,
            }),
        }
    }
}

pub trait EntityCollectionResource<Key> {
    fn stream(
        self,
    ) -> impl futures::Stream<Item = Result<(Vec<Key>, EntityUid, bool), anyhow::Error>> + Send + Sync + 'static;
}

#[async_trait]
impl<Key> ReadOnlyRepository<Vec<Key>, EntityUid> for TrieRepositoryData<Key>
where
    Key: Ord + Send + Sync + Debug + Hash,
{
    type ReadError = anyhow::Error;

    async fn get(&self, key: Vec<Key>) -> Result<EntityUid, Self::ReadError> {
        let guard = self.rw_lock.read().await;
        guard
            .maybe_trie
            .clone()
            .and_then(|trie| trie.exact_match(&key).map(|e| e.clone()))
            .ok_or(anyhow!("Entity not found: {:?}", key))
    }
}

#[async_trait]
impl<Key> UpsertRepository<Vec<Key>, EntityUid> for TrieRepositoryData<Key>
where
    Key: Ord + Send + Sync + Debug + Clone + Hash,
{
    type Error = anyhow::Error;

    async fn upsert(&self, key: Vec<Key>, entity: EntityUid) -> Result<(), Self::Error> {
        let mut guard = self.rw_lock.write().await;
        guard.items.insert(key, entity);
        guard.maybe_trie = Some(Arc::new(Trie::from_iter(guard.items.clone())));
        Ok(())
    }

    async fn exists(&self, key: Vec<Key>) -> Result<bool, Self::Error> {
        let guard = self.rw_lock.read().await;
        Ok(guard
            .maybe_trie
            .clone()
            .and_then(|trie| trie.exact_match(&key).map(|e| e.clone()))
            .is_some())
    }
}

#[async_trait]
impl<Key> CanDelete<Vec<Key>, EntityUid> for TrieRepositoryData<Key>
where
    Key: Ord + Send + Sync + Debug + Clone + Hash,
{
    type DeleteError = anyhow::Error;

    async fn delete(&self, key: Vec<Key>) -> Result<(), Self::DeleteError> {
        let mut guard = self.rw_lock.write().await;
        guard.items.remove(&key);
        guard.maybe_trie = Some(Arc::new(Trie::from_iter(guard.items.clone())));
        Ok(())
    }
}

#[async_trait]
impl<R, K> ResourceUpdateHandler<R> for TrieRepositoryData<K>
where
    R: EntityCollectionResource<K> + Debug + Resource + Send + Sync + 'static,
    K: Ord + Send + Sync + Debug + Clone + Hash + 'static,
{
    async fn handle_update(&self, event: Result<R, Error>) -> () {
        match event {
            Err(e) => warn!("Failed to handle update: {:?}", e),
            Ok(resource) => {
                resource
                    .stream()
                    .for_each(|result| async move {
                        match result {
                            Ok((segments, action_uid, active)) => {
                                let result = if active {
                                    self.upsert(segments.clone(), action_uid.clone()).await
                                } else {
                                    self.delete(segments.clone()).await
                                };
                                if let Err(e) = result {
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
                    .await;
                info!("Finished updating action discovery trie");
            }
        }
    }
}
