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

impl From<PolicySet> for PolicyDocumentSpec {
    fn from(value: PolicySet) -> Self {
        PolicyDocumentSpec {
            active: true,
            policies: value.to_string(),
        }
    }
}

impl Into<PolicySet> for PolicyDocumentSpec {
    fn into(self) -> PolicySet {
        PolicySet::from_str(&self.policies).unwrap_or_else(|_| PolicySet::default())
    }
}
