use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::Resource;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PolicyDocument {
    pub(crate) policies: String,
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[resource(inherit = ConfigMap)]
pub struct PolicyResource {
    metadata: ObjectMeta,
    pub data: PolicyDocument,
}
