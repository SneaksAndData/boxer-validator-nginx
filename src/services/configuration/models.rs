use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::ListenerConfig;
use duration_string::DurationString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RepositorySettings {
    pub label_selector_key: String,
    pub label_selector_value: String,
    pub operation_timeout: DurationString,
}
impl Into<ListenerConfig> for &RepositorySettings {
    fn into(self) -> ListenerConfig {
        ListenerConfig {
            label_selector_key: self.label_selector_key.clone(),
            label_selector_value: self.label_selector_value.clone(),
            operation_timeout: self.operation_timeout.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SchemaRepositorySettings {
    pub label_selector_key: String,
    pub label_selector_value: String,
    pub name: String,
    pub operation_timeout: DurationString,
}

impl Into<ListenerConfig> for &SchemaRepositorySettings {
    fn into(self) -> ListenerConfig {
        ListenerConfig {
            label_selector_key: self.label_selector_key.clone(),
            label_selector_value: self.label_selector_value.clone(),
            operation_timeout: self.operation_timeout.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct KubernetesBackendSettings {
    pub kubeconfig: Option<String>,
    pub exec: Option<String>,
    pub in_cluster: bool,
    pub namespace: String,

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
