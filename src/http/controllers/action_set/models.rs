use crate::services::repositories::action_repository::action_discovery_document::{
    ActionDiscoveryDocument, ActionDiscoveryDocumentSpec,
};
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::status::Status;
use boxer_core::services::backends::kubernetes::repositories::{ToResource, TryFromResource};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ActionRouteRegistration {
    pub method: String,
    pub route_template: String,
    pub action_uid: String,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ActionSetRegistration {
    pub hostname: String,
    pub routes: Vec<ActionRouteRegistration>,
}

impl TryFromResource<ActionDiscoveryDocument> for ActionSetRegistration {
    type Error = Status;

    fn try_into_resource(resource: Arc<ActionDiscoveryDocument>) -> std::result::Result<Self, Self::Error> {
        let spec = resource.spec.clone();
        spec.try_into()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
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
