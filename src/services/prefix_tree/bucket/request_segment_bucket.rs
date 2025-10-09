use crate::services::prefix_tree::bucket::TrieBucket;
use crate::services::repositories::models::path_segment::PathSegment;
use crate::services::repositories::models::request_segment::RequestSegment;
use async_trait::async_trait;
use log::warn;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct RequestBucket<Value> {
    exact_match: RwLock<HashMap<RequestSegment, Arc<RequestBucket<Value>>>>,
    parameter: RwLock<Option<Arc<RequestBucket<Value>>>>,
    value: RwLock<Option<Value>>,
}

impl<Value> Default for RequestBucket<Value> {
    fn default() -> Self {
        RequestBucket {
            exact_match: RwLock::new(HashMap::new()),
            parameter: RwLock::new(None),
            value: RwLock::new(None),
        }
    }
}

#[async_trait]
impl<Value> TrieBucket<RequestSegment, Value> for RequestBucket<Value>
where
    Value: Clone + Send + Sync + Debug,
{
    async fn child(&self, key: &RequestSegment) -> Option<Arc<Self>> {
        match key {
            RequestSegment::Path(PathSegment::Static(_)) => {
                let exact_match = self.exact_match.read().await.get(key).map(|c| c.clone());
                match exact_match {
                    Some(c) => Some(c),
                    None => self.parameter.read().await.clone(),
                }
            }
            RequestSegment::Path(PathSegment::Parameter) => self.parameter.read().await.clone(),
            other => self.exact_match.read().await.get(other).map(|c| c.clone()),
        }
    }

    async fn create_child(&self, key: &RequestSegment) {
        match key {
            RequestSegment::Path(PathSegment::Parameter) => {
                let mut lock = self.parameter.write().await;
                lock.replace(Arc::new(Self::default()));
            }
            _ => {
                let mut lock = self.exact_match.write().await;
                lock.insert(key.clone(), Arc::new(Self::default())).map(|_| ());
            }
        }
    }

    async fn get_value(&self) -> Option<Value> {
        self.value.read().await.clone()
    }

    async fn clear(&self) -> Option<Value> {
        self.value.write().await.take()
    }

    async fn set_value(&self, value: Value) {
        let mut prev = self.value.write().await; //.replace(value);
        if prev.is_some() {
            warn!("Overwriting existing value in RequestBucket at segment: {:?}", self);
        }
        prev.replace(value);
    }
}
