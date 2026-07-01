use crate::services::repositories::resource_repository::resource_discovery_document::{
    ResourceDiscoveryDocument, ResourceDiscoveryDocumentSpec,
};
use boxer_core::services::audit::audit_facade::to_audit_record::ToAuditRecord;
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

impl ResourceSetRegistration {
    pub fn with_schema(self, schema: String) -> SchemaBoundResourceSetRegistration {
        SchemaBoundResourceSetRegistration {
            hostname: self.hostname,
            routes: self.routes,
            schema,
        }
    }
}

#[derive(Serialize)]
pub struct SchemaBoundResourceSetRegistration {
    pub hostname: String,
    pub routes: Vec<ResourceRouteRegistration>,
    pub schema: String,
}

impl TryFromResource<ResourceDiscoveryDocument> for SchemaBoundResourceSetRegistration {
    type Error = Status;

    fn try_from_resource(resource: Arc<ResourceDiscoveryDocument>) -> Result<Self, Self::Error> {
        let spec = resource.spec.clone();
        spec.try_into()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
    }
}

impl ToResource<ResourceDiscoveryDocument> for SchemaBoundResourceSetRegistration {
    fn to_resource(&self, object_meta: &ObjectMeta) -> Result<ResourceDiscoveryDocument, Status> {
        let spec = ResourceDiscoveryDocumentSpec::try_from(self)
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))?;
        Ok(ResourceDiscoveryDocument {
            metadata: object_meta.clone(),
            spec,
        })
    }
}

impl Into<ResourceSetRegistration> for SchemaBoundResourceSetRegistration {
    fn into(self) -> ResourceSetRegistration {
        let routes: Vec<ResourceRouteRegistration> = self
            .routes
            .into_iter()
            .map(|route| ResourceRouteRegistration {
                route_template: route.route_template,
                resource_uid: route.resource_uid,
            })
            .collect();

        ResourceSetRegistration {
            hostname: self.hostname,
            routes,
        }
    }
}

impl ToAuditRecord for SchemaBoundResourceSetRegistration {
    fn to_audit_record(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "<failed to serialize to json>: {}".to_string())
    }
}
