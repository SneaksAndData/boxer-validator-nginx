use crate::http::controllers::action_set::models::ActionSetRegistration;
use crate::http::controllers::policy_set::models::PolicySetRegistration;
use crate::services::repositories::policy_repository::models::{PolicyDocument, PolicyDocumentSpec};
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::{
    KubernetesResourceManagerConfig, UpdateLabels,
};
use boxer_core::services::backends::kubernetes::repositories::{
    KubernetesRepository, SoftDeleteResource, ToResource, TryFromResource,
};
use boxer_core::services::base::upsert_repository::{UpsertRepository, UpsertRepositoryWithDelete};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use std::collections::BTreeMap;
use std::sync::Arc;

type PolicyDataRepository = dyn UpsertRepositoryWithDelete<
    String,
    ActionSetRegistration,
    DeleteError = Status,
    Error = Status,
    ReadError = Status,
>;

impl TryFromResource<PolicyDocument> for PolicySetRegistration {
    type Error = Status;

    fn try_into_resource(resource: Arc<PolicyDocument>) -> Result<Self, Self::Error> {
        let spec = resource.spec.clone();
        spec.try_into()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
    }
}

impl SoftDeleteResource for PolicyDocument {
    fn is_deleted(&self) -> bool {
        !self.spec.active
    }

    fn set_deleted(&mut self) {
        self.spec.active = false;
    }

    fn clear_managed_fields(&mut self) {
        self.metadata.managed_fields = None;
    }
}

impl ToResource<PolicyDocument> for PolicySetRegistration {
    fn to_resource(&self, object_meta: &ObjectMeta) -> Result<PolicyDocument, Status> {
        let spec = PolicyDocumentSpec {
            active: true,
            policies: self.policy.clone(),
        };
        Ok(PolicyDocument {
            metadata: object_meta.clone(),
            spec,
        })
    }
}

impl UpdateLabels for PolicyDocument {
    fn update_labels(mut self, custom_labels: &mut BTreeMap<String, String>) -> Self {
        let mut labels = self.metadata.labels.unwrap_or_default();
        labels.append(custom_labels);
        self.metadata.labels = Some(labels);
        self
    }
}

impl UpsertRepositoryWithDelete<String, PolicySetRegistration> for KubernetesRepository<PolicyDocument> {}

pub async fn new(config: KubernetesResourceManagerConfig) -> anyhow::Result<Arc<PolicyDataRepository>> {
    let repository = KubernetesRepository::start(config.clone()).await?;
    Ok(Arc::new(repository))
}
