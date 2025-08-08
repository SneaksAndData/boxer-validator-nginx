use crate::http::controllers::policy_set::models::PolicySetRegistration;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::UpdateLabels;
use boxer_core::services::backends::kubernetes::repositories::SoftDeleteResource;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "PolicyDocument",
    plural = "policy-documents",
    singular = "policy-document",
    namespaced
)]
pub struct PolicyDocumentSpec {
    pub active: bool,
    pub policies: String,
}

impl Default for PolicyDocument {
    fn default() -> Self {
        PolicyDocument {
            metadata: ObjectMeta {
                name: None,
                namespace: None,
                ..Default::default()
            },
            spec: PolicyDocumentSpec {
                active: true,
                policies: String::new(),
            },
        }
    }
}

impl From<PolicySetRegistration> for PolicyDocumentSpec {
    fn from(value: PolicySetRegistration) -> Self {
        PolicyDocumentSpec {
            active: true,
            policies: value.policy.to_string(),
        }
    }
}

impl Into<PolicySetRegistration> for PolicyDocumentSpec {
    fn into(self) -> PolicySetRegistration {
        PolicySetRegistration { policy: self.policies }
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

impl UpdateLabels for PolicyDocument {
    fn update_labels(mut self, custom_labels: &mut BTreeMap<String, String>) -> Self {
        let mut labels = self.metadata.labels.unwrap_or_default();
        labels.append(custom_labels);
        self.metadata.labels = Some(labels);
        self
    }
}
