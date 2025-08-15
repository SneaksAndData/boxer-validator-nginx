use duration_string::DurationString;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct SchemaRepositorySettings {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct KubernetesBackendSettings {
    pub kubeconfig: Option<String>,
    pub exec: Option<String>,
    pub in_cluster: bool,
    pub namespace: String,
    pub schema_repository: SchemaRepositorySettings,
    pub resource_owner_label: String,
    pub operation_timeout: DurationString,
}

#[derive(Debug, Deserialize)]
pub struct BackendSettings {
    pub kubernetes: KubernetesBackendSettings,
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub listen_address: SocketAddr,
    pub instance_name: String,
    pub backend: BackendSettings,
}
