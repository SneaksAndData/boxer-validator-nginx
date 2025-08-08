use crate::services::repositories::policy_repository::policy_document::{PolicyDocument, PolicyDocumentSpec};
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::{ToResource, TryFromResource};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct PolicySetRegistration {
    pub policy: String,
}

impl Default for PolicySetRegistration {
    fn default() -> Self {
        PolicySetRegistration {
            policy: Default::default(),
        }
    }
}

impl TryFromResource<PolicyDocument> for PolicySetRegistration {
    type Error = Status;

    fn try_into_resource(resource: Arc<PolicyDocument>) -> Result<Self, Self::Error> {
        let spec = resource.spec.clone();
        spec.try_into()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
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
