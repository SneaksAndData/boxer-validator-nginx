use crate::services::prefix_tree::bucket::TrieBucket;
use async_trait::async_trait;

pub mod hash_tree;

pub mod bucket;
#[cfg(test)]
mod tests;

#[async_trait]
pub trait MutableTrie<K, V> {
    type Bucket: TrieBucket<K, V>;
    async fn insert(&mut self, key: impl AsRef<[K]> + Send, value: V);

    async fn get(&self, key: impl AsRef<[K]> + Send) -> Option<V>;
    async fn delete(&self, key: impl AsRef<[K]> + Send) -> Option<V>;
}
