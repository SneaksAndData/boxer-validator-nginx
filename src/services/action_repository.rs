#[cfg(test)]
mod tests;

use anyhow::anyhow;
use async_trait::async_trait;
use boxer_core::services::base::upsert_repository::UpsertRepository;
use cedar_policy::EntityUid;
use std::sync::Arc;
use tokio::sync::RwLock;
use trie_rs::map::{Trie, TrieBuilder};
use crate::models::request_context::RequestContext;
use url::Url;

#[derive(Debug, Clone)]
struct ActionMapping {
    pub verb: String,
    pub hostname: String,
    pub path: String,
}

impl TryFrom<RequestContext> for ActionMapping {
    type Error = anyhow::Error;

    fn try_from(context: RequestContext) -> Result<Self, Self::Error> {
        let verb = context.original_method.clone();
        let uri = Url::parse(context.original_url.as_str())?;
        let host = uri.host_str()
            .ok_or_else(|| anyhow!("Invalid URL: missing host"))?
            .to_string();
        
        Ok(ActionMapping{ verb, hostname: host, path: uri.path().to_string() })
    }
}

struct ActionRepositoryData {
    builder: Box<TrieBuilder<u8, EntityUid>>,
    maybe_trie: Option<Arc<Trie<u8, EntityUid>>>
}

struct ActionRepository {
    rw_lock: RwLock<ActionRepositoryData>,
}

impl ActionRepository {
    pub fn new() -> ActionRepository {
        ActionRepository {
            rw_lock: RwLock::new(ActionRepositoryData {
                builder: Box::new(TrieBuilder::new()),
                maybe_trie: None,
            }),
        }
    }
}

#[async_trait]
impl ReadOnlyRepository<ActionMapping, EntityUid> for ActionRepository {
    type Error = anyhow::Error;

    async fn get(&self, key: ActionMapping) -> Result<EntityUid, Self::Error> {
        let line = format!("{} {} {}", key.verb, key.hostname, key.path);
        let guard = self.rw_lock.read().await;
        guard.maybe_trie.clone()
            .and_then(|trie| trie.exact_match(&line).map(|e|e.clone()))
            .ok_or(anyhow!("Entity not found: {:?}", key))
    }

    async fn upsert(&self, key: ActionMapping, entity: EntityUid) -> Result<(), Self::Error> {
        let line = format!("{} {} {}", key.verb, key.hostname, key.path);
        let mut guard = self.rw_lock.write().await;
        let mut builder = guard.builder.clone();
        builder.push(line, entity.clone());
        guard.builder = builder.clone();
        guard.maybe_trie = Some(Arc::new(builder.build()));
        Ok(())
    }

    async fn delete(&self, key: ActionMapping) -> Result<(), Self::Error> {
        todo!()
    }

    async fn exists(&self, key: ActionMapping) -> Result<bool, Self::Error> {
        todo!()
    }
}

// struct ResourceMapping {
//     pub verb: String,
//     pub hostname: String,
//     pub path: String,
// }
// 
// struct ResourceMapper {
// 
// }
// 
// impl ResourceMapper {
//     pub fn new() -> ResourceMapper {
//         ResourceMapper {}
//     }
// 
//     pub fn add_mapping(&mut self, action_mapping: ActionMapping, action: EntityUid) {
// 
//     }
//     
//     pub fn get_mapping(&self, action_mapping: ActionMapping) -> Option<EntityUid> {
//         None
//     }
// }
