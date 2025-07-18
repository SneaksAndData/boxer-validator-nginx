use anyhow::anyhow;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::{ReadOnlyRepository, UpsertRepository};
use cedar_policy::EntityUid;
use futures::Stream;
use futures::StreamExt;
use futures_util::stream;
use kube::runtime::watcher;
use kube::Resource;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockWriteGuard};
use trie_rs::map::{Trie, TrieBuilder};

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

#[allow(dead_code)]
trait CollectionResource<'de, In, Out>: Resource + Serialize + Deserialize<'de> + Clone
where
    In: TryInto<EntityUid, Error = anyhow::Error>
        + TryInto<Vec<Out>, Error = anyhow::Error>
        + Clone
        + Send
        + Sync
        + Debug,
{
    fn deserialize(&self) -> Result<Vec<In>, anyhow::Error>;

    fn stream(self) -> impl Stream<Item = Result<(Vec<Out>, EntityUid), anyhow::Error>> + Send {
        let items = self.deserialize().unwrap();
        stream::iter(items).map(move |o| {
            let action_uid: EntityUid = o.clone().try_into()?;
            let mut key: Vec<Out> = vec![];
            let segments: Vec<Out> = o.try_into()?;
            key.extend(segments);
            Ok((key, action_uid))
        })
    }
}

#[allow(dead_code)]
trait CollectionResourceUpdateHandler<Key, In, R>: ResourceUpdateHandler<R>
where
    In: TryInto<EntityUid, Error = anyhow::Error>
        + TryInto<Vec<Key>, Error = anyhow::Error>
        + Clone
        + Send
        + Sync
        + Debug,
    R: CollectionResource<'static, In, Key> + Send + Sync + Debug + 'static,
    Key: Ord + Send + Sync + Debug + Clone + 'static,
    Self: TrieRepository<Key> + Send + Sync + Debug,
{
    fn handle_update(&self, event: Result<R, watcher::Error>) -> impl Future<Output = ()> + Send {
        async {
            match event {
                Ok(resource) => {
                    debug!("Received update for resource: {:?}", resource);
                    self.refresh_trie().await;
                    self.handle_async(resource).await;
                }
                Err(e) => {
                    warn!("Watcher error: {}", e);
                }
            }
        }
    }
    fn handle_async(&self, event: R) -> impl Future<Output = ()> + Send {
        event.stream().for_each(move |result| async {
            match result {
                Ok((segments, action_uid)) => match self.upsert(segments, action_uid.clone()).await {
                    Ok(_) => {
                        debug!("Upserted action: {:?}", action_uid);
                    }
                    Err(e) => {
                        warn!("Failed to upsert action: {}", e);
                    }
                },
                Err(err) => {
                    warn!("Failed to process action: {}", err);
                }
            }
        })
    }
}
