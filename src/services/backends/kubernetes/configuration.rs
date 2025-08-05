use crate::services::backends::kubernetes::KubernetesBackend;
use crate::services::backends::BackendBuilder;
use crate::services::configuration::models::KubernetesBackendSettings;
use crate::services::repositories::backend::ReadOnlyRepositoryBackend;
use crate::services::repositories::policy_repository::PolicyRepositoryData;
use crate::services::repositories::{action_repository, resource_repository};
use anyhow::bail;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubeconfig_loader::{from_cluster, from_command, from_file};
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
            KubernetesBackendSettings { in_cluster: true, .. } => from_cluster().load()?,

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
            claimant: instance_name.clone(),
            kubeconfig: kubeconfig.clone(),
        };
        let action_lookup = action_repository::read_only::new();
        let action_lookup_watcher =
            ReadOnlyRepositoryBackend::start(repository_config.clone(), action_lookup.clone()).await?;

        let action_repository = action_repository::read_write::new(repository_config.clone()).await;
        let action_repository_watcher =
            ReadOnlyRepositoryBackend::start(repository_config, action_lookup.clone()).await?;

        let repository_config = KubernetesResourceManagerConfig {
            namespace: settings.namespace.clone(),
            label_selector_key: settings.resource_repository.label_selector_key.clone(),
            label_selector_value: settings.resource_repository.label_selector_value.clone(),
            lease_name: settings.lease_name.clone(),
            lease_duration: settings.lease_duration.into(),
            renew_deadline: settings.lease_renew_duration.into(),
            claimant: instance_name.clone(),
            kubeconfig: kubeconfig.clone(),
        };
        let resource_repository = ReadOnlyRepositoryBackend::start(repository_config, action_lookup.clone()).await?;

        let repository_config = KubernetesResourceManagerConfig {
            namespace: settings.namespace.clone(),
            label_selector_key: settings.policy_repository.label_selector_key.clone(),
            label_selector_value: settings.policy_repository.label_selector_value.clone(),
            lease_name: settings.lease_name.clone(),
            lease_duration: settings.lease_duration.into(),
            renew_deadline: settings.lease_renew_duration.into(),
            claimant: instance_name,
            kubeconfig,
        };
        let policy_data = Arc::new(PolicyRepositoryData::new());
        let policy_repository_backend =
            ReadOnlyRepositoryBackend::start(repository_config, policy_data.clone()).await?;

        Ok(Arc::new(KubernetesBackend {
            schema_repository: Arc::new(schema_repository),
            action_readonly_repository: action_lookup,
            resource_repository: Arc::new(resource_repository::read_only::new()),
            policy_repository: policy_data,

            action_lookup_watcher: Arc::new(action_lookup_watcher),
            action_repository_watcher: Arc::new(action_repository_watcher),
            action_data_repository: action_repository,

            resource_repository_backend: Arc::new(resource_repository),
            policy_repository_backend: Arc::new(policy_repository_backend),
        }))
    }
}
