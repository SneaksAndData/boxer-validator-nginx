use super::models::PolicyDocument;
use crate::services::repositories::policy_repository::PolicyReadOnlyRepositoryInterface;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::ResourceUpdateHandler;
use boxer_core::services::base::upsert_repository::ReadOnlyRepository;
use cedar_policy::PolicySet;
use kube::runtime::watcher;
use log::{debug, warn};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PolicyRepositoryData {
    policy_set: RwLock<PolicySet>,
}

pub(crate) fn new() -> Arc<PolicyRepositoryData> {
    Arc::new(PolicyRepositoryData {
        policy_set: RwLock::new(PolicySet::default()),
    })
}

impl PolicyRepositoryData {
    async fn set_policies(&self, policy_set: PolicySet) -> () {
        let mut guard = self.policy_set.write().await;
        *(guard) = policy_set.clone();
    }
}

#[async_trait]
impl ReadOnlyRepository<(), PolicySet> for PolicyRepositoryData {
    type ReadError = anyhow::Error;

    async fn get(&self, _key: ()) -> Result<PolicySet, Self::ReadError> {
        let guard = self.policy_set.read().await;
        Ok((*guard).clone())
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
                    Ok(policies) => self.set_policies(policies).await,
                }
            }
        }
    }
}

impl PolicyReadOnlyRepositoryInterface for PolicyRepositoryData {}
