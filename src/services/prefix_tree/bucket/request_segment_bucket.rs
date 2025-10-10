use crate::services::prefix_tree::bucket::TrieBucket;
use crate::services::prefix_tree::hash_tree::ParametrizedMatcher;
use crate::services::repositories::models::request_segment::RequestSegment;
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
struct NextReference<Value> {
    exact_match: RwLock<HashMap<RequestSegment, Arc<RequestBucket<Value>>>>,
    parameter: RwLock<Option<Arc<RequestBucket<Value>>>>,
}

impl<Value> NextReference<Value> {
    fn new() -> Self {
        NextReference {
            exact_match: RwLock::new(HashMap::new()),
            parameter: RwLock::new(None),
        }
    }
}

#[derive(Debug)]
pub struct RequestBucket<Value> {
    next: NextReference<Value>,
    exact_labels: RwLock<HashMap<RequestSegment, Value>>,
    parameter_value: RwLock<Option<Value>>,
}

impl<Value> Default for RequestBucket<Value> {
    fn default() -> Self {
        RequestBucket {
            next: NextReference::new(),
            exact_labels: RwLock::new(HashMap::new()),
            parameter_value: RwLock::new(None),
        }
    }
}

#[async_trait]
impl<Value> TrieBucket<RequestSegment, Value> for RequestBucket<Value>
where
    Value: Clone + Send + Sync + Debug,
{
    async fn child(&self, key: &RequestSegment) -> Option<Arc<Self>> {
        let exact_match = self.next.exact_match.read().await.get(key).map(|c| c.clone());
        if exact_match.is_some() {
            return exact_match;
        }
        self.next.parameter.read().await.clone()
    }

    async fn create_child(&self, key: &RequestSegment) {
        if key.is_parameter() {
            let mut lock = self.next.parameter.write().await;
            lock.replace(Arc::new(Self::default()));
        } else {
            let mut lock = self.next.exact_match.write().await;
            lock.insert(key.clone(), Arc::new(Self::default())).map(|_| ());
        }
    }

    async fn get_value(&self, key: &RequestSegment) -> Option<Value> {
        let exact_match = self.exact_labels.read().await.get(key).cloned();
        if exact_match.is_some() {
            return exact_match;
        }
        self.parameter_value.read().await.clone()
    }

    async fn clear(&self, key: &RequestSegment) -> Option<Value> {
        if key.is_parameter() {
            self.parameter_value.write().await.take()
        } else {
            self.exact_labels.write().await.remove(key)
        }
    }

    async fn set_value(&self, value: Value, key: &RequestSegment) {
        if key.is_parameter() {
            self.parameter_value.write().await.replace(value);
        } else {
            self.exact_labels.write().await.insert(key.clone(), value.clone());
        };
    }
}
