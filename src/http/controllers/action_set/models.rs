use crate::services::repositories::action_repository::action_discovery_document::{
    ActionDiscoveryDocument, ActionDiscoveryDocumentSpec,
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

impl ActionSetRegistration {
    pub fn with_schema(self, schema: String) -> SchemaBoundActionSetRegistration {
        SchemaBoundActionSetRegistration {
            hostname: self.hostname,
            routes: self.routes,
            schema,
        }
    }
}

#[derive(Serialize)]
pub struct SchemaBoundActionSetRegistration {
    pub hostname: String,
    pub routes: Vec<ActionRouteRegistration>,
    pub schema: String,
}

impl TryFromResource<ActionDiscoveryDocument> for SchemaBoundActionSetRegistration {
    type Error = Status;

    fn try_into_resource(resource: Arc<ActionDiscoveryDocument>) -> Result<Self, Self::Error> {
        let spec = resource.spec.clone();
        spec.try_into()
            .map_err(|e| Status::ConversionError(anyhow::Error::from(e)))
    }
}

impl ToResource<ActionDiscoveryDocument> for SchemaBoundActionSetRegistration {
    fn to_resource(&self, object_meta: &ObjectMeta) -> Result<ActionDiscoveryDocument, Status> {
        let spec =
            ActionDiscoveryDocumentSpec::try_from(self).map_err(|e| Status::ConversionError(anyhow::Error::from(e)))?;
        Ok(ActionDiscoveryDocument {
            metadata: object_meta.clone(),
            spec,
        })
    }
}

impl Into<ActionSetRegistration> for SchemaBoundActionSetRegistration {
    fn into(self) -> ActionSetRegistration {
        ActionSetRegistration {
            hostname: self.hostname,
            routes: self.routes,
        }
    }
}

impl ToAuditRecord for SchemaBoundActionSetRegistration {
    fn to_audit_record(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "<failed to serialize to json>: {}".to_string())
    }
}
