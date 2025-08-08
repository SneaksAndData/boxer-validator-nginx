use crate::services::backends::kubernetes::KubernetesBackend;
use crate::services::backends::BackendBuilder;
use crate::services::configuration::models::{KubernetesBackendSettings, RepositorySettings};
use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::repositories::lookup_trie::backend::ReadOnlyRepositoryBackend;
use crate::services::repositories::lookup_trie::TrieRepositoryData;
use crate::services::repositories::policy_repository;
use crate::services::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
use anyhow::bail;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubeconfig_loader::{from_cluster, from_command, from_file};
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
use boxer_core::services::backends::kubernetes::repositories::KubernetesRepository;
use boxer_core::services::backends::BackendConfiguration;
use kube::Config;
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
        let kubeconfig = Self::get_kubeconfig(settings).await?;

        let schema_repository = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            instance_name.clone(),
            &settings.schema_repository,
        )
        .await?;

        let action_lookup_watcher = Self::create_lookup_trie(
            &settings.namespace,
            kubeconfig.clone(),
            instance_name.clone(),
            &settings.actions_repository,
        )
        .await?;

        let action_repository: Arc<ActionDataRepository> = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            instance_name.clone(),
            &settings.actions_repository,
        )
        .await?;

        let resource_lookup_watcher = Self::create_lookup_trie(
            &settings.namespace,
            kubeconfig.clone(),
            instance_name.clone(),
            &settings.resource_repository,
        )
        .await?;

        let resource_repository: Arc<ResourceDiscoveryDocumentRepository> = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            instance_name.clone(),
            &settings.resource_repository,
        )
        .await?;

        let policy_lookup = Self::create_readonly_repository(
            &settings.namespace,
            kubeconfig,
            instance_name,
            &settings.policy_repository,
        )
        .await?;

        let policy_repository = Self::create_repository(
            &settings.namespace,
            kubeconfig,
            instance_name,
            &settings.policy_repository,
        )
        .await?;

        Ok(Arc::new(KubernetesBackend {
            schema_repository: Arc::new(schema_repository),
        }))
    }
}

impl BackendBuilder {
    async fn get_kubeconfig(settings: &KubernetesBackendSettings) -> anyhow::Result<Config> {
        match settings {
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
        }
    }

    pub async fn create_repository<R>(
        namespace: &str,
        kubeconfig: Config,
        instance_name: String,
        settings: &RepositorySettings,
    ) -> anyhow::Result<Arc<KubernetesRepository<R>>> {
        let config = KubernetesResourceManagerConfig {
            namespace: namespace.to_string(),
            kubeconfig: kubeconfig.clone(),
            field_manager: instance_name.clone(),
            listener_config: (&settings).into(),
        };
        KubernetesRepository::<R>::start(config)
            .await
            .map(Arc::new)
            .map_err(|e| e.into())
    }

    pub async fn create_lookup_trie<K>(
        namespace: &str,
        kubeconfig: Config,
        instance_name: String,
        settings: &RepositorySettings,
    ) -> anyhow::Result<ReadOnlyRepositoryBackend>
    where
        K: Ord,
    {
        let config = KubernetesResourceManagerConfig {
            namespace: namespace.to_string(),
            kubeconfig: kubeconfig.clone(),
            field_manager: instance_name.clone(),
            listener_config: (&settings).into(),
        };
        let lookup_trie = Arc::new(TrieRepositoryData::new());
        ReadOnlyRepositoryBackend::start(config, lookup_trie).await
    }

    pub async fn create_readonly_repository<K>(
        namespace: &str,
        kubeconfig: Config,
        instance_name: String,
        settings: &RepositorySettings,
    ) -> anyhow::Result<ReadOnlyRepositoryBackend>
    where
        K: Ord,
    {
        let config = KubernetesResourceManagerConfig {
            namespace: namespace.to_string(),
            kubeconfig: kubeconfig.clone(),
            field_manager: instance_name.clone(),
            listener_config: (&settings).into(),
        };
        let lookup_trie = policy_repository::read_only::new();
        ReadOnlyRepositoryBackend::start(config, lookup_trie).await
    }
}
