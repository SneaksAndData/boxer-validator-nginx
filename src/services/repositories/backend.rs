#[cfg(test)]
mod test_data;
#[cfg(test)]
mod tests;

use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::{
    KubernetesResourceWatcher, ResourceUpdateHandler,
};
use futures::stream::StreamExt;
use k8s_openapi::NamespaceResourceScope;
use kube::runtime::watcher::Config;
use kube::runtime::{reflector, watcher, WatchStreamExt};
use kube::{Api, Client, Resource};
use log::debug;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use tokio::task::JoinHandle;

pub struct ReadOnlyRepositoryBackend {
    handle: JoinHandle<()>,
}

impl ReadOnlyRepositoryBackend {
    pub fn new(handle: JoinHandle<()>) -> Self {
        ReadOnlyRepositoryBackend { handle }
    }
}

#[async_trait]
impl<S> KubernetesResourceWatcher<S> for ReadOnlyRepositoryBackend
where
    S: Resource<Scope = NamespaceResourceScope> + Clone + Debug + Serialize + DeserializeOwned + Send + Sync + 'static,
    S::DynamicType: Hash + Eq + Clone + Default,
{
    async fn start<H>(config: KubernetesResourceManagerConfig, update_handler: Arc<H>) -> anyhow::Result<Self>
    where
        H: ResourceUpdateHandler<S> + Send + Sync + 'static,
    {
        let client = Client::try_from(config.kubeconfig)?;
        let api: Api<S> = Api::namespaced(client.clone(), config.namespace.as_str());
        let watcher_config = Config {
            label_selector: Some(format!("{}={}", config.label_selector_key, config.label_selector_value)),
            ..Default::default()
        };
        let stream = watcher(api, watcher_config);
        let (reader, writer) = reflector::store();

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

        Ok(ReadOnlyRepositoryBackend::new(handle))
    }

    fn stop(&self) -> anyhow::Result<()> {
        self.handle.abort();
        debug!("KubernetesResourceManager stopped");
        Ok(())
    }
}
