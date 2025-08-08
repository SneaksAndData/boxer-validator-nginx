use crate::http::controllers::policy_set::models::PolicySetRegistration;
use cedar_policy::PolicySet;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

impl Default for PolicySetRegistration {
    fn default() -> Self {
        PolicySetRegistration {
            policy: Default::default(),
        }
    }
}
