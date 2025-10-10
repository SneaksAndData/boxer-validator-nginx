use crate::services::prefix_tree::bucket::TrieBucket;
use async_trait::async_trait;

pub mod hash_tree;

pub mod bucket;
pub mod mutable_trie_builder;
#[cfg(test)]
mod tests;

#[async_trait]
pub trait MutableTrie<K, V> {
    async fn get(&self, key: impl AsRef<[K]> + Send) -> Option<V>;
}
