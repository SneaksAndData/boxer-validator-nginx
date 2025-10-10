use crate::services::prefix_tree::trie_bucket::TrieBucket;
use crate::services::prefix_tree::MutablePrefixTree;
use crate::services::prefix_tree::PrefixTree;
use async_trait::async_trait;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct NaiveTrie<Bucket> {
    root: Arc<Bucket>,
}

impl<Bucket> NaiveTrie<Bucket>
where
    Bucket: Default,
{
    pub fn new() -> Self {
        NaiveTrie {
            root: Arc::new(Bucket::default()),
        }
    }
}

#[async_trait]
impl<Key, Value, Bucket> PrefixTree<Key, Value> for NaiveTrie<Bucket>
where
    Key: Hash + ParametrizedMatcher + Sync + Debug,
    Bucket: TrieBucket<Key, Value> + Send + Sync + Debug,
    Value: Send + Sync + 'static,
{
    async fn get(&self, key: impl AsRef<[Key]> + Send) -> Option<Value> {
        let keys = key.as_ref();
        if keys.is_empty() {
            return None;
        }
        let mut current = self.root.clone();

        for k in key.as_ref() {
            let child = current.child(k).await;
            match child {
                Some(child) => current = child,
                None => return None,
            }
        }

        let last = keys.last().unwrap();
        current.get_value(last).await
    }
}

#[async_trait]
impl<Key, Value, Bucket> MutablePrefixTree<Key, Value> for NaiveTrie<Bucket>
where
    Key: Hash + ParametrizedMatcher + Sync + Debug,
    Bucket: TrieBucket<Key, Value> + Send + Sync + Debug,
    Value: Send + Sync + 'static,
{
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

        let keys = key.as_ref();
        let last = keys.last().unwrap();
        current.set_value(value, last).await;
    }
    async fn delete(&self, key: impl AsRef<[Key]> + Send) -> Option<Value> {
        let mut current = self.root.clone();

        for k in key.as_ref() {
            match current.child(k).await {
                Some(child) => current = child,
                None => return None,
            }
        }

        let keys = key.as_ref();
        let last = keys.last().unwrap();
        current.clear(last).await
    }
}

pub trait ParametrizedMatcher {
    fn is_parameter(&self) -> bool;
}
