use crate::services::prefix_tree::naive_tree::ParametrizedMatcher;
use crate::services::prefix_tree::trie_bucket::TrieBucket;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
struct NextReference<Key, Value>
where
    Key: Send + Sync + Debug,
    Value: Send + Sync,
{
    exact_match: RwLock<HashMap<Key, Arc<RequestBucket<Key, Value>>>>,
    parameter: RwLock<Option<Arc<RequestBucket<Key, Value>>>>,
}

impl<Key, Value> NextReference<Key, Value>
where
    Key: Send + Sync + Debug,
    Value: Send + Sync,
{
    fn new() -> Self {
        NextReference {
            exact_match: RwLock::new(HashMap::new()),
            parameter: RwLock::new(None),
        }
    }
}

#[derive(Debug)]
pub struct RequestBucket<Key, Value>
where
    Key: Send + Sync + Debug,
    Value: Send + Sync,
{
    next: NextReference<Key, Value>,
    exact_labels: RwLock<HashMap<Key, Value>>,
    parameter_value: RwLock<Option<Value>>,
}

impl<Key, Value> Default for RequestBucket<Key, Value>
where
    Key: Send + Sync + Debug,
    Value: Send + Sync,
{
    fn default() -> Self {
        RequestBucket {
            next: NextReference::new(),
            exact_labels: RwLock::new(HashMap::new()),
            parameter_value: RwLock::new(None),
        }
    }
}

#[async_trait]
impl<Key, Value> TrieBucket<Key, Value> for RequestBucket<Key, Value>
where
    Value: Clone + Send + Sync,
    Key: ParametrizedMatcher + Send + Sync + Debug + Clone + Eq + Hash,
{
    async fn child(&self, key: &Key) -> Option<Arc<Self>> {
        let exact_match = self.next.exact_match.read().await.get(key).map(|c| c.clone());
        if exact_match.is_some() {
            return exact_match;
        }
        self.next.parameter.read().await.clone()
    }

    async fn create_child(&self, key: &Key) {
        if key.is_parameter() {
            let mut lock = self.next.parameter.write().await;
            lock.replace(Arc::new(Self::default()));
        } else {
            let mut lock = self.next.exact_match.write().await;
            lock.insert(key.clone(), Arc::new(Self::default())).map(|_| ());
        }
    }

    async fn get_value(&self, key: &Key) -> Option<Value> {
        let exact_match = self.exact_labels.read().await.get(key).cloned();
        if exact_match.is_some() {
            return exact_match;
        }
        self.parameter_value.read().await.clone()
    }

    async fn clear(&self, key: &Key) -> Option<Value> {
        if key.is_parameter() {
            self.parameter_value.write().await.take()
        } else {
            self.exact_labels.write().await.remove(key)
        }
    }

    async fn set_value(&self, value: Value, key: &Key) {
        if key.is_parameter() {
            self.parameter_value.write().await.replace(value);
        } else {
            self.exact_labels.write().await.insert(key.clone(), value.clone());
        };
    }
}
