use crate::services::action_repository::backend::ActionRepositoryBackend;
use crate::services::action_repository::ActionData;
use crate::services::backends::kubernetes::KubernetesBackend;
use crate::services::backends::BackendBuilder;
use crate::services::configuration::models::KubernetesBackendSettings;
use anyhow::bail;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubeconfig_loader::{from_command, from_file};
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
use boxer_core::services::backends::kubernetes::repositories::schema_repository::KubernetesSchemaRepository;
use boxer_core::services::backends::BackendConfiguration;
use std::sync::Arc;

#[async_trait]
impl BackendConfiguration for BackendBuilder {
    type BackendSettings = KubernetesBackendSettings;
    type InitializedBackend = KubernetesBackend;

    async fn configure(
        self,
        settings: &Self::BackendSettings,
        instance_name: String,
    ) -> anyhow::Result<Arc<Self::InitializedBackend>> {
        let kubeconfig = match settings {
            KubernetesBackendSettings {
                kubeconfig: Some(path), ..
            } => from_file().load(&path).await?,
            KubernetesBackendSettings {
                exec: Some(command), ..
            } => from_command().load(&command).await?,
            KubernetesBackendSettings {
                kubeconfig: None,
                exec: None,
                ..
            } => {
                bail!("Kubernetes backend configuration is missing")
            }
        };

        let repository_config = KubernetesResourceManagerConfig {
            namespace: settings.namespace.clone(),
            label_selector_key: settings.schema_repository.label_selector_key.clone(),
            label_selector_value: settings.schema_repository.label_selector_value.clone(),
            lease_name: settings.lease_name.clone(),
            lease_duration: settings.lease_duration.into(),
            renew_deadline: settings.lease_renew_duration.into(),
            claimant: instance_name.clone(),
            kubeconfig: kubeconfig.clone(),
        };

        let schema_repository = KubernetesSchemaRepository::start(repository_config).await?;

        let repository_config = KubernetesResourceManagerConfig {
            namespace: settings.namespace.clone(),
            label_selector_key: settings.actions_repository.label_selector_key.clone(),
            label_selector_value: settings.actions_repository.label_selector_value.clone(),
            lease_name: settings.lease_name.clone(),
            lease_duration: settings.lease_duration.into(),
            renew_deadline: settings.lease_renew_duration.into(),
            claimant: instance_name,
            kubeconfig,
        };
        let action_data = ActionData::new();
        let action_repository = ActionRepositoryBackend::start(repository_config, action_data.clone()).await?;

        Ok(Arc::new(KubernetesBackend {
            schema_repository: Arc::new(schema_repository),
            action_repository_backend: Arc::new(action_repository),
            action_repository: action_data,
        }))
    }
}
