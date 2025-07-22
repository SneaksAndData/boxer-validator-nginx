use duration_string::DurationString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RepositorySettings {
    pub label_selector_key: String,
    pub label_selector_value: String,
}

#[derive(Debug, Deserialize)]
pub struct SchemaRepositorySettings {
    pub label_selector_key: String,
    pub label_selector_value: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct KubernetesBackendSettings {
    pub kubeconfig: Option<String>,
    pub exec: Option<String>,
    pub in_cluster: bool,
    pub namespace: String,

    pub lease_name: String,
    pub lease_duration: DurationString,
    pub lease_renew_duration: DurationString,

    pub schema_repository: SchemaRepositorySettings,
    pub actions_repository: RepositorySettings,
    pub resource_repository: RepositorySettings,
    pub policy_repository: RepositorySettings,
}

#[derive(Debug, Deserialize)]
pub struct BackendSettings {
    pub kubernetes: KubernetesBackendSettings,
}

#[derive(Debug, Deserialize)]
pub struct AppSettings {
    pub instance_name: String,
    pub backend: BackendSettings,
}
