use crate::services::repositories::policy_repository::policy_document::PolicyDocument;
use crate::services::repositories::policy_repository::PolicyReadOnlyRepositoryInterface;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::ReadOnlyRepository;
use cedar_policy::PolicySet;
use kube::runtime::watcher;
use log::{debug, warn};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PolicyRepositoryData {
    policy_set: RwLock<HashMap<String, PolicySet>>,
}

pub(crate) fn new() -> Arc<PolicyRepositoryData> {
    Arc::new(PolicyRepositoryData {
        policy_set: RwLock::new(HashMap::default()),
    })
}

#[async_trait]
impl ReadOnlyRepository<String, PolicySet> for PolicyRepositoryData {
    type ReadError = anyhow::Error;

    async fn get(&self, key: String) -> Result<PolicySet, Self::ReadError> {
        let guard = self.policy_set.read().await;
        match guard.get(&key) {
            Some(policy_set) => Ok(policy_set.clone()),
            None => Err(anyhow::anyhow!("Policy set not found for key: {}", key)),
        }
    }
}

#[async_trait]
impl ResourceUpdateHandler<PolicyDocument> for PolicyRepositoryData {
    async fn handle_update(&self, event: Result<PolicyDocument, watcher::Error>) -> () {
        match event {
            Err(err) => warn!("Error while fetching policy: {:?}", err),
            Ok(event) => {
                debug!("Received policy update: {:?}", event);
                let policies = PolicySet::from_str(&event.spec.policies);
                match policies {
                    Err(err) => warn!("Failed to parse policy set: {:?}", err),
                    Ok(policies) => {
                        let mut guard = self.policy_set.write().await;
                        guard
                            .entry(event.spec.schema)
                            .and_modify(|existing| {
                                *existing = policies.clone();
                            })
                            .or_insert(policies.clone());
                    }
                }
            }
        }
    }
}

impl PolicyReadOnlyRepositoryInterface for PolicyRepositoryData {}
