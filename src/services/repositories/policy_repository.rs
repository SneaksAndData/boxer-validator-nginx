pub mod models;
mod tests;

use crate::services::repositories::policy_repository::models::PolicyResource;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::ReadOnlyRepository;
use cedar_policy::PolicySet;
use kube::runtime::watcher;
use log::{info, warn};
use std::future::Future;
use std::str::FromStr;
use tokio::sync::RwLock;

pub struct PolicyRepositoryData {
    policy_set: RwLock<PolicySet>,
}

impl PolicyRepositoryData {
    pub(crate) fn new() -> PolicyRepositoryData {
        PolicyRepositoryData {
            policy_set: RwLock::new(PolicySet::default()),
        }
    }
}

impl PolicyRepositoryData {
    async fn set_policies(&self, policy_set: PolicySet) -> () {
        let mut guard = self.policy_set.write().await;
        *(guard) = policy_set.clone();
    }
}

pub type PolicyRepository = dyn ReadOnlyRepository<(), PolicySet, ReadError = anyhow::Error>;

#[async_trait]
impl ReadOnlyRepository<(), PolicySet> for PolicyRepositoryData {
    type ReadError = anyhow::Error;

    async fn get(&self, _key: ()) -> Result<PolicySet, Self::ReadError> {
        let guard = self.policy_set.read().await;
        Ok((*guard).clone())
    }
}

impl ResourceUpdateHandler<PolicyResource> for PolicyRepositoryData {
    fn handle_update(&self, event: Result<PolicyResource, watcher::Error>) -> impl Future<Output = ()> + Send {
        async {
            match event {
                Err(err) => warn!("Error while fetching policy: {:?}", err),
                Ok(event) => {
                    info!("Received policy update: {:?}", event);
                    let policies = PolicySet::from_str(&event.data.policies);
                    match policies {
                        Err(err) => warn!("Failed to parse policy set: {:?}", err),
                        Ok(policies) => self.set_policies(policies).await,
                    }
                }
            }
            info!("Finished updating action discovery trie");
        }
    }
}
