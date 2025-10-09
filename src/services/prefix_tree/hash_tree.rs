use crate::services::prefix_tree::{MutableTrie, TrieBucket};
use async_trait::async_trait;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct HashTrie<Bucket> {
    root: Arc<Bucket>,
}

impl<Bucket> HashTrie<Bucket>
where
    Bucket: Default,
{
    pub fn new() -> Self {
        HashTrie {
            root: Arc::new(Bucket::default()),
        }
    }
}

#[async_trait]
impl<Key, Value, Bucket> MutableTrie<Key, Value> for HashTrie<Bucket>
where
    Key: Hash + Sync + Debug,
    Bucket: TrieBucket<Key, Value> + Send + Sync,
    Value: Send + Sync + 'static,
{
    type Bucket = Bucket;

    async fn insert(&mut self, key: impl AsRef<[Key]> + Send, value: Value) {
        if key.as_ref().is_empty() {
            return;
        }

        let mut current = self.root.clone();

        for k in key.as_ref() {
            if current.child(k).await.is_none() {
                current.create_child(k).await;
            }

            current = current.child(k).await.unwrap();
        }

        current.set_value(value).await;
    }

    async fn get(&self, key: impl AsRef<[Key]> + Send) -> Option<Value> {
        let mut current = self.root.clone();

        for k in key.as_ref() {
            match current.child(k).await {
                Some(child) => current = child,
                None => return None,
            }
        }

        current.get_value().await
    }

    async fn delete(&self, key: impl AsRef<[Key]> + Send) -> Option<Value> {
        let mut current = self.root.clone();

        for k in key.as_ref() {
            match current.child(k).await {
                Some(child) => current = child,
                None => return None,
            }
        }

        current.clear().await
    }
}
