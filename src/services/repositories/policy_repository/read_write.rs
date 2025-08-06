use crate::services::repositories::policy_repository::models::{PolicyDocument, PolicyDocumentSpec};
use crate::services::repositories::policy_repository::PolicyRepositoryInterface;
use anyhow::anyhow;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::synchronized::SynchronizedKubernetesResourceManager;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::logging_update_handler::LoggingUpdateHandler;
use boxer_core::services::base::upsert_repository::{CanDelete, ReadOnlyRepository, UpsertRepository};
use cedar_policy::PolicySet;
use collection_macros::btreemap;
use kube::runtime::reflector::ObjectRef;
use std::sync::Arc;

pub struct PolicyDataRepository {
    pub resource_manager: SynchronizedKubernetesResourceManager<PolicyDocument>,
    label_selector_key: String,
    label_selector_value: String,
}

impl PolicyDataRepository {
    pub async fn start(config: KubernetesResourceManagerConfig) -> anyhow::Result<Self> {
        let label_selector_key = config.label_selector_key.clone();
        let label_selector_value = config.label_selector_value.clone();
        let resource_manager =
            SynchronizedKubernetesResourceManager::start(config, Arc::new(LoggingUpdateHandler)).await?;
        Ok(PolicyDataRepository {
            resource_manager,
            label_selector_key,
            label_selector_value,
        })
    }
}

pub async fn new(config: KubernetesResourceManagerConfig) -> Arc<PolicyDataRepository> {
    let repository = PolicyDataRepository::start(config)
        .await
        .expect("Failed to start PolicyDataRepository");
    Arc::new(repository)
}

#[async_trait]
impl ReadOnlyRepository<String, PolicySet> for PolicyDataRepository {
    type ReadError = anyhow::Error;

    async fn get(&self, key: String) -> Result<PolicySet, Self::ReadError> {
        let or = ObjectRef::new(key.as_str()).within(self.resource_manager.namespace().as_str());
        let resource_object = self.resource_manager.get(or);
        let resource_object = match resource_object {
            Some(r) => r,
            None => return Err(anyhow!("Resource not found: {}", key)),
        };
        if !resource_object.spec.active {
            return Err(anyhow!("Schema is not active"));
        }
        let result: PolicySet = resource_object.spec.clone().into();
        Ok(result)
    }
}

#[async_trait]
impl UpsertRepository<String, PolicySet> for PolicyDataRepository {
    type Error = anyhow::Error;

    async fn upsert(&self, key: String, entity: PolicySet) -> Result<(), Self::Error> {
        let or = ObjectRef::new(key.as_str()).within(self.resource_manager.namespace().as_str());
        let mut resource_ref = self.resource_manager.get(or).unwrap_or_default();
        let resource_ref = Arc::make_mut(&mut resource_ref);
        resource_ref.metadata.name = Some(key.clone());
        resource_ref.metadata.labels = Some(btreemap! {
            self.label_selector_key.clone() => self.label_selector_value.clone(),
        });
        resource_ref.metadata.namespace = Some(self.resource_manager.namespace().clone());
        resource_ref.spec = PolicyDocumentSpec::try_from(entity)?;
        resource_ref.spec.active = true;
        self.resource_manager.replace(&key, resource_ref.clone()).await
    }

    async fn exists(&self, key: String) -> bool {
        let or: ObjectRef<PolicyDocument> =
            ObjectRef::new(key.as_str()).within(self.resource_manager.namespace().as_str());
        self.resource_manager.get(or).map(|r| r.spec.active).unwrap_or(false)
    }
}

#[async_trait]
impl CanDelete<String, PolicySet> for PolicyDataRepository {
    type DeleteError = anyhow::Error;

    async fn delete(&self, key: String) -> Result<(), Self::DeleteError> {
        let or = ObjectRef::new(key.as_str()).within(self.resource_manager.namespace().as_str());
        let resource_ref = self.resource_manager.get(or);
        let mut resource_ref = match resource_ref {
            Some(r) => r,
            None => return Err(anyhow!("Resource not found: {}", key)),
        };
        let resource_object = Arc::make_mut(&mut resource_ref);
        resource_object.spec.active = false;
        self.resource_manager.replace(&key, resource_object.clone()).await
    }
}

impl PolicyRepositoryInterface for PolicyDataRepository {}
