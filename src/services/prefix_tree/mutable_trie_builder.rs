use async_trait::async_trait;

/// A mutable prefix tree that supports insertion and deletion of key-value pairs.
#[async_trait]
pub trait MutablePrefixTree<K, V> {
    /// Inserts a key-value pair into the trie.
    async fn insert(&mut self, key: impl AsRef<[K]> + Send, value: V);

    /// Deletes a key-value pair from the trie and returns the associated value if it existed.
    async fn delete(&self, key: impl AsRef<[K]> + Send) -> Option<V>;
}
