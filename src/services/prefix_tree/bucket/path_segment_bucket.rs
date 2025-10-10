use crate::services::prefix_tree::bucket::TrieBucket;
use crate::services::repositories::models::path_segment::PathSegment;
use async_trait::async_trait;
use log::warn;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::Path;

#[derive(Debug)]
pub struct PathSegmentBucket<Value> {
    exact_match: RwLock<HashMap<PathSegment, Arc<PathSegmentBucket<Value>>>>,
    parameter: RwLock<Option<Arc<PathSegmentBucket<Value>>>>,
    value: RwLock<Option<Value>>,
}

impl<Value> Default for PathSegmentBucket<Value> {
    fn default() -> Self {
        PathSegmentBucket {
            exact_match: RwLock::new(HashMap::new()),
            parameter: RwLock::new(None),
            value: RwLock::new(None),
        }
    }
}

#[async_trait]
impl<Value> TrieBucket<PathSegment, Value> for PathSegmentBucket<Value>
where
    Value: Clone + Send + Sync + Debug,
{
    async fn child(&self, key: &PathSegment) -> Option<Arc<Self>> {
        match key {
            PathSegment::Static(_) => {
                let exact_match = self.exact_match.read().await.get(key).map(|c| c.clone());
                match exact_match {
                    Some(c) => Some(c),
                    None => self.parameter.read().await.clone(),
                }
            }
            PathSegment::Parameter => self.parameter.read().await.clone(),
        }
    }

    async fn create_child(&self, key: &PathSegment) {
        match key {
            PathSegment::Parameter => {
                let mut lock = self.parameter.write().await;
                lock.replace(Arc::new(Self::default()));
            }
            _ => {
                let mut lock = self.exact_match.write().await;
                lock.insert(key.clone(), Arc::new(Self::default())).map(|_| ());
            }
        }
    }

    async fn get_value(&self, key: &PathSegment) -> Option<Value> {
        self.value.read().await.clone()
    }

    async fn clear(&self, key: &PathSegment) -> Option<Value> {
        self.value.write().await.take()
    }

    async fn set_value(&self, value: Value, key: &PathSegment) {
        let mut prev = self.value.write().await; //.replace(value);
        if prev.is_some() {
            warn!("Overwriting existing value in PathSegmentBucket at segment: {:?}", self);
        }
        prev.replace(value);
    }
}
