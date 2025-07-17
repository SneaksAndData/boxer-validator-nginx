mod test_data;
#[cfg(test)]
mod tests;

use crate::services::action_repository::models::RequestSegment::{Parameter, Verb};
use crate::services::action_repository::models::{HTTPMethod, RequestSegment};
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::{
    KubernetesResourceWatcher, ResourceUpdateHandler,
};
use cedar_policy::EntityUid;
use futures::stream::{self, StreamExt};
use futures::Stream;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::runtime::watcher::Config;
use kube::runtime::{reflector, watcher, WatchStreamExt};
use kube::{Api, Client, Resource};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ActionRoute {
    method: HTTPMethod,
    route_template: String,
    action_uid: String,
}

impl TryInto<Vec<RequestSegment>> for ActionRoute {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<RequestSegment>, Self::Error> {
        let mut segments: Vec<RequestSegment> = vec![Verb(self.method.try_into()?)];
        for segment in self.route_template.split('/') {
            if segment.is_empty() {
                continue;
            }
            if segment.starts_with('{') && segment.ends_with('}') {
                segments.push(Parameter)
            } else {
                segments.push(RequestSegment::Static(segment.to_string()))
            }
        }
        Ok(segments)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActionDiscoveryDocument {
    hostname: String,
    routes: Vec<ActionRoute>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActionDiscoveryDocumentData {
    pub(crate) actions: String,
}

impl ActionDiscoveryDocument {
    pub fn stream(self) -> impl Stream<Item = Result<(Vec<RequestSegment>, EntityUid), anyhow::Error>> {
        stream::iter(self.routes).map(move |route| {
            let action_uid: EntityUid = EntityUid::from_str(&route.action_uid).map_err(anyhow::Error::from)?;
            let mut key: Vec<RequestSegment> = vec![RequestSegment::Hostname(self.hostname.clone())];
            let segments: Vec<RequestSegment> = route.try_into()?;
            key.extend(segments);
            Ok((key, action_uid))
        })
    }
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[resource(inherit = ConfigMap)]
pub struct ActionDiscoveryResource {
    metadata: ObjectMeta,
    pub data: ActionDiscoveryDocumentData,
}

pub struct ActionRepositoryBackend {
    handle: JoinHandle<()>,
}

impl ActionRepositoryBackend {
    pub fn new(handle: JoinHandle<()>) -> Self {
        ActionRepositoryBackend { handle }
    }
}

#[async_trait]
impl KubernetesResourceWatcher<ActionDiscoveryResource> for ActionRepositoryBackend {
    async fn start<H>(config: KubernetesResourceManagerConfig, update_handler: Arc<H>) -> anyhow::Result<Self>
    where
        H: ResourceUpdateHandler<ActionDiscoveryResource> + Send + Sync + 'static,
    {
        let client = Client::try_from(config.kubeconfig)?;
        let api: Api<ActionDiscoveryResource> = Api::namespaced(client.clone(), config.namespace.as_str());
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

        Ok(ActionRepositoryBackend::new(handle))
    }

    fn stop(&self) -> anyhow::Result<()> {
        self.handle.abort();
        debug!("KubernetesResourceManager stopped");
        Ok(())
    }
}
