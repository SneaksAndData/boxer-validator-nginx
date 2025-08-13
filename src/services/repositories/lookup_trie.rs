use anyhow::anyhow;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::{ReadOnlyRepository, UpsertRepository};
use cedar_policy::EntityUid;
use futures::StreamExt;
use kube::runtime::watcher::Error;
use kube::Resource;
use log::{debug, info, warn};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockWriteGuard};
use trie_rs::map::{Trie, TrieBuilder};

pub mod backend;

#[async_trait]
pub trait TrieRepository<Key>:
    ReadOnlyRepository<Vec<Key>, EntityUid, ReadError = anyhow::Error>
    + UpsertRepository<Vec<Key>, EntityUid, Error = anyhow::Error>
where
    Key: Ord + 'static + Send + Sync + Debug + Clone,
{
    #[allow(dead_code)]
    async fn write_lock(&self) -> RwLockWriteGuard<'_, TrieData<Key>>;

    #[allow(dead_code)]
    async fn refresh_trie(&self) -> () {
        info!("Updating action discovery trie");
        let mut guard = self.write_lock().await;
        guard.builder = Box::new(TrieBuilder::new());
        guard.maybe_trie = None;
    }
}

pub struct TrieData<Key> {
    builder: Box<TrieBuilder<Key, EntityUid>>,
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
                builder: Box::new(TrieBuilder::new()),
                maybe_trie: None,
            }),
        }
    }
}

pub trait EntityCollectionResource<Key> {
    fn stream(
        self,
    ) -> impl futures::Stream<Item = Result<(Vec<Key>, EntityUid), anyhow::Error>> + Send + Sync + 'static;
}

#[async_trait]
impl<Key> TrieRepository<Key> for TrieRepositoryData<Key>
where
    Key: Send + Sync + Debug + Ord + Clone + 'static,
{
    async fn write_lock(&self) -> RwLockWriteGuard<'_, TrieData<Key>> {
        self.rw_lock.write().await
    }
}

#[async_trait]
impl<Key> ReadOnlyRepository<Vec<Key>, EntityUid> for TrieRepositoryData<Key>
where
    Key: Ord + Send + Sync + Debug,
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
    Key: Ord + Send + Sync + Debug + Clone,
{
    type Error = anyhow::Error;

    async fn upsert(&self, key: Vec<Key>, entity: EntityUid) -> Result<(), Self::Error> {
        let mut guard = self.rw_lock.write().await;
        let mut builder = guard.builder.clone();
        builder.push(key, entity.clone());
        guard.builder = builder.clone();
        guard.maybe_trie = Some(Arc::new(builder.build()));
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
impl<R, K> ResourceUpdateHandler<R> for TrieRepositoryData<K>
where
    R: EntityCollectionResource<K> + Debug + Resource + Send + Sync + 'static,
    K: Ord + Send + Sync + Debug + Clone + 'static,
{
    async fn handle_update(&self, event: Result<R, Error>) -> () {
        match event {
            Err(e) => warn!("Failed to handle update: {:?}", e),
            Ok(resource) => {
                {
                    info!("Received resource update: {:?}", resource);
                    let mut guard = self.rw_lock.write().await;
                    guard.builder = Box::new(TrieBuilder::new());
                    guard.maybe_trie = None;
                }
                resource
                    .stream()
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
                    .await;
                info!("Finished updating action discovery trie");
            }
        }
    }
}
