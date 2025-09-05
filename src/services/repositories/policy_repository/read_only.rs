use crate::services::repositories::policy_repository::policy_document::PolicyDocument;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::ReadOnlyRepository;
use cedar_policy::{Policy, PolicyId, PolicySet};
use kube::core::object::HasSpec;
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
                let policy = Policy::from_str(&event.spec.policies);
                match policy {
                    Err(err) => warn!("Failed to parse policy set: {:?}", err),
                    Ok(new_policy) => {
                        // We use here unwrap because the name is guaranteed to be present by Kubernetes
                        // and PolicyId::from_str return Infallible as an error
                        let new_id = PolicyId::from_str(&event.metadata.name.clone().unwrap()).unwrap();
                        let new_policy = new_policy.new_id(new_id);
                        let mut guard = self.policy_set.write().await;
                        match guard.get_mut(&event.spec.schema) {
                            Some(existing) => {
                                let result = existing.add(new_policy);
                                if let Err(err) = result {
                                    warn!("Failed to add policy to an existing set: {:?}", err);
                                    return;
                                }
                            }
                            None => {
                                let mut new_set = PolicySet::new();
                                let result = new_set.add(new_policy);
                                if let Err(err) = result {
                                    warn!("Failed to add policy to new set: {:?}", err);
                                    return;
                                }
                                guard.insert(event.spec.schema, new_set);
                            }
                        }
                    }
                }
            }
        }
    }
}
