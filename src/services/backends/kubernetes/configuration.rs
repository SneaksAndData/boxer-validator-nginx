use crate::services::backends::kubernetes::KubernetesBackend;
use crate::services::backends::BackendBuilder;
use crate::services::configuration::models::KubernetesBackendSettings;
use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use crate::services::repositories::lookup_trie::backend::ReadOnlyRepositoryBackend;
use crate::services::repositories::lookup_trie::{EntityCollectionResource, TrieRepositoryData};
use crate::services::repositories::models::path_segment::PathSegment;
use crate::services::repositories::policy_repository;
use crate::services::repositories::policy_repository::policy_document::PolicyDocument;
use crate::services::repositories::policy_repository::read_only::PolicyRepositoryData;
use crate::services::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
use anyhow::bail;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubeconfig_loader::{from_cluster, from_command, from_file};
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::object_owner_mark::ObjectOwnerMark;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::{
    KubernetesResourceManagerConfig, UpdateLabels,
};
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::KubernetesResourceWatcher;
use boxer_core::services::backends::kubernetes::repositories::{KubernetesRepository, SoftDeleteResource};
use boxer_core::services::backends::BackendConfiguration;
use k8s_openapi::NamespaceResourceScope;
use kube::Config;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

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
        let owner_mark = ObjectOwnerMark::new(&instance_name, &settings.resource_owner_label);

        let schema_repository = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.schema_repository.operation_timeout.into(),
        )
        .await?;

        let action_lookup_table_listener = Self::create_lookup_trie(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.actions_repository.operation_timeout.into(),
        )
        .await?;

        let action_repository: Arc<ActionDataRepository> = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.actions_repository.operation_timeout.into(),
        )
        .await?;

        let resource_lookup_table_listener = Self::create_lookup_trie(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.resource_repository.operation_timeout.into(),
        )
        .await?;

        let resource_repository: Arc<ResourceDiscoveryDocumentRepository> = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.resource_repository.operation_timeout.into(),
        )
        .await?;

        let policy_lookup_watcher = Self::create_readonly_repository::<PathSegment>(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.policy_repository.operation_timeout.into(),
        )
        .await?;

        let policy_repository = Self::create_repository(
            &settings.namespace,
            kubeconfig.clone(),
            owner_mark.clone(),
            settings.policy_repository.operation_timeout.into(),
        )
        .await?;

        Ok(Arc::new(KubernetesBackend {
            schema_repository,
            action_repository,
            resource_repository,
            policy_repository,
            action_lookup_table_listener,
            resource_lookup_table_listener,
            policy_lookup_watcher,
        }))
    }
}

impl BackendBuilder {
    async fn get_kubeconfig(settings: &KubernetesBackendSettings) -> anyhow::Result<Config> {
        match settings {
            KubernetesBackendSettings { in_cluster: true, .. } => from_cluster().load(),

            KubernetesBackendSettings {
                kubeconfig: Some(path), ..
            } => from_file().load(&path).await,

            KubernetesBackendSettings {
                exec: Some(command), ..
            } => from_command().load(&command).await,

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
        owner_mark: ObjectOwnerMark,
        operation_timeout: Duration,
    ) -> anyhow::Result<Arc<KubernetesRepository<R>>>
    where
        R: kube::Resource<Scope = NamespaceResourceScope>
            + SoftDeleteResource
            + UpdateLabels
            + Clone
            + Send
            + Sync
            + 'static,
        R::DynamicType: Hash + Eq + Clone + Default,
    {
        let config = KubernetesResourceManagerConfig {
            namespace: namespace.to_string(),
            kubeconfig: kubeconfig.clone(),
            owner_mark,
            operation_timeout,
        };
        KubernetesRepository::<R>::start(config)
            .await
            .map(Arc::new)
            .map_err(|e| e.into())
    }

    pub async fn create_lookup_trie<R, K>(
        namespace: &str,
        kubeconfig: Config,
        owner_mark: ObjectOwnerMark,
        operation_timeout: Duration,
    ) -> anyhow::Result<Arc<ReadOnlyRepositoryBackend<TrieRepositoryData<K>, R>>>
    where
        K: Debug + Ord + Clone + Send + Sync + 'static,
        R: kube::Resource<Scope = NamespaceResourceScope>
            + SoftDeleteResource
            + EntityCollectionResource<K>
            + UpdateLabels
            + Clone
            + Send
            + Sync
            + 'static,
        R::DynamicType: Hash + Eq + Clone + Default,
    {
        let config = KubernetesResourceManagerConfig {
            namespace: namespace.to_string(),
            kubeconfig: kubeconfig.clone(),
            owner_mark,
            operation_timeout,
        };
        let lookup_trie = Arc::new(TrieRepositoryData::<K>::new());
        let r = ReadOnlyRepositoryBackend::<TrieRepositoryData<K>, R>::start(config, lookup_trie).await?;
        Ok(Arc::new(r))
    }

    pub async fn create_readonly_repository<K>(
        namespace: &str,
        kubeconfig: Config,
        owner_mark: ObjectOwnerMark,
        operation_timeout: Duration,
    ) -> anyhow::Result<Arc<ReadOnlyRepositoryBackend<PolicyRepositoryData, PolicyDocument>>>
    where
        K: Ord,
    {
        let config = KubernetesResourceManagerConfig {
            namespace: namespace.to_string(),
            kubeconfig: kubeconfig.clone(),
            owner_mark,
            operation_timeout,
        };
        let lookup_trie = policy_repository::read_only::new();
        let r = ReadOnlyRepositoryBackend::start(config, lookup_trie).await?;
        Ok(Arc::new(r))
    }
}
