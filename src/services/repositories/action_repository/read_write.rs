use crate::http::controllers::action_set::models::ActionSetRegistration;
use crate::services::repositories::action_repository::models::{ActionDiscoveryDocument, ActionDiscoveryDocumentSpec};
use anyhow::Result;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::{
    KubernetesResourceManagerConfig, UpdateLabels,
};
use boxer_core::services::backends::kubernetes::repositories::{
    KubernetesRepository, SoftDeleteResource, ToResource, TryFromResource,
};
use boxer_core::services::base::upsert_repository::UpsertRepositoryWithDelete;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use std::collections::BTreeMap;
use std::sync::Arc;

type ActionDataRepository = dyn UpsertRepositoryWithDelete<
    String,
    ActionSetRegistration,
    DeleteError = Status,
    Error = Status,
    ReadError = Status,
>;

impl TryFromResource<ActionDiscoveryDocument> for ActionSetRegistration {
    type Error = Status;

    fn try_into_resource(resource: Arc<ActionDiscoveryDocument>) -> std::result::Result<Self, Self::Error> {
        let spec = resource.spec.clone();
        spec.try_into()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
    }
}

impl SoftDeleteResource for ActionDiscoveryDocument {
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

impl ToResource<ActionDiscoveryDocument> for ActionSetRegistration {
    fn to_resource(&self, object_meta: &ObjectMeta) -> std::result::Result<ActionDiscoveryDocument, Status> {
        let spec = ActionDiscoveryDocumentSpec::try_from(self.clone())
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))?;
        Ok(ActionDiscoveryDocument {
            metadata: object_meta.clone(),
            spec,
        })
    }
}

impl UpdateLabels for ActionDiscoveryDocument {
    fn update_labels(mut self, custom_labels: &mut BTreeMap<String, String>) -> Self {
        let mut labels = self.metadata.labels.unwrap_or_default();
        labels.append(custom_labels);
        self.metadata.labels = Some(labels);
        self
    }
}

impl UpsertRepositoryWithDelete<String, ActionSetRegistration> for KubernetesRepository<ActionDiscoveryDocument> {}

pub async fn new(config: KubernetesResourceManagerConfig) -> Result<Arc<ActionDataRepository>> {
    let repository = KubernetesRepository::start(config.clone()).await?;
    Ok(Arc::new(repository))
}
