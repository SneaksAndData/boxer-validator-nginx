use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::{
    KubernetesResourceWatcher, ResourceUpdateHandler,
};
use futures::stream::StreamExt;
use k8s_openapi::NamespaceResourceScope;
use kube::runtime::{reflector, watcher, WatchStreamExt};
use kube::{Api, Client, Resource};
use log::debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct ReadOnlyRepositoryBackend<H, S>
where
    H: ResourceUpdateHandler<S> + Send + Sync + 'static,
    S: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    S::DynamicType: Hash + Eq + Clone + Default,
{
    handle: JoinHandle<()>,
    update_handler: Arc<H>,
    _marker: std::marker::PhantomData<S>,
}

impl<H, S> ReadOnlyRepositoryBackend<H, S>
where
    H: ResourceUpdateHandler<S> + Send + Sync + 'static,
    S: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync,
    S::DynamicType: Hash + Eq + Clone + Default,
{
    pub fn new(handle: JoinHandle<()>, update_handler: Arc<H>) -> Self {
        ReadOnlyRepositoryBackend {
            handle,
            update_handler,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get_update_handler(&self) -> Arc<H> {
        self.update_handler.clone()
    }
}

#[async_trait]
impl<H, S> KubernetesResourceWatcher<H, S> for ReadOnlyRepositoryBackend<H, S>
where
    S: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync + 'static,
    S::DynamicType: Hash + Eq + Clone + Default,
    H: ResourceUpdateHandler<S> + Send + Sync + 'static,
{
    async fn start(config: KubernetesResourceManagerConfig, update_handler: Arc<H>) -> anyhow::Result<Self> {
        let client = Client::try_from(config.kubeconfig)?;
        let api: Api<S> = Api::namespaced(client.clone(), config.namespace.as_str());
        let watcher_config = (&config.owner_mark).into();
        let stream = watcher(api, watcher_config);
        let (reader, writer) = reflector::store();

        let h = update_handler.clone();
        let reflector = reflector(writer, stream)
            .default_backoff()
            .touched_objects()
            .for_each(move |r| {
                let update_handler = update_handler.clone();
                async move {
                    update_handler.handle_update(r).await;
                }
            });

        let handle = tokio::spawn(reflector);
        reader.wait_until_ready().await?;

        Ok(ReadOnlyRepositoryBackend::new(handle, h))
    }

    fn stop(&self) -> anyhow::Result<()> {
        self.handle.abort();
        debug!("KubernetesResourceManager stopped");
        Ok(())
    }
}
