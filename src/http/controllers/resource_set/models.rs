use crate::services::repositories::resource_repository::resource_discovery_document::{
    ResourceDiscoveryDocument, ResourceDiscoveryDocumentSpec,
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
pub struct ResourceRouteRegistration {
    pub route_template: String,
    pub resource_uid: String,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ResourceSetRegistration {
    pub hostname: String,
    pub routes: Vec<ResourceRouteRegistration>,
}

impl TryFromResource<ResourceDiscoveryDocument> for ResourceSetRegistration {
    type Error = Status;

    fn try_into_resource(resource: Arc<ResourceDiscoveryDocument>) -> Result<Self, Self::Error> {
        let spec = resource.spec.clone();
        spec.try_into()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
    }
}

impl ToResource<ResourceDiscoveryDocument> for ResourceSetRegistration {
    fn to_resource(&self, object_meta: &ObjectMeta) -> Result<ResourceDiscoveryDocument, Status> {
        let spec = ResourceDiscoveryDocumentSpec::try_from(self.clone())
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))?;
        Ok(ResourceDiscoveryDocument {
            metadata: object_meta.clone(),
            spec,
        })
    }
}
