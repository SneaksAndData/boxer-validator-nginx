use crate::services::action_repository;
use crate::services::action_repository::models::RequestSegment::{Parameter, Verb};
use crate::services::action_repository::models::{HTTPMethod, RequestSegment};
use crate::services::action_repository::{ActionData, ActionRepository};
use actix_web::web::to;
use anyhow::Error;
use async_trait::async_trait;
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::KubernetesResourceManagerConfig;
use boxer_core::services::backends::kubernetes::kubernetes_resource_watcher::{
    KubernetesResourceWatcher, ResourceUpdateHandler,
};
use boxer_core::services::base::upsert_repository::UpsertRepository;
use cedar_policy::{ActionConstraint, EntityId, EntityTypeName, EntityUid, SchemaFragment};
use futures::stream::{self, StreamExt};
use futures::Stream;
use futures_util::future::{ready, Ready};
use futures_util::stream::{ForEachConcurrent, StreamFuture};
use futures_util::{future, FutureExt, TryFutureExt};
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::NamespaceResourceScope;
use kube::runtime::watcher::Config;
use kube::runtime::{reflector, watcher, WatchStreamExt};
use kube::{Api, Client, Resource};
use log::{debug, warn};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::future::Future;
use std::future::IntoFuture;
use std::hash::Hash;
use std::pin::Pin;
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
struct ActionDiscoveryDocument {
    hostname: String,
    routes: Vec<ActionRoute>,
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
pub struct ActionsConfigMap {
    metadata: ObjectMeta,
    data: ActionDiscoveryDocument,
}

pub struct ActionRepositoryBackend {
    repository: Arc<dyn ActionRepository>,
    handle: JoinHandle<()>,
}

impl ActionRepositoryBackend {
    pub fn new(handle: JoinHandle<()>) -> Self {
        let repository = action_repository::new();
        ActionRepositoryBackend { repository, handle }
    }

    pub fn repository(&self) -> Arc<dyn ActionRepository> {
        self.repository.clone()
    }
}

#[async_trait]
impl KubernetesResourceWatcher<ActionsConfigMap> for ActionRepositoryBackend {
    async fn start<H>(config: KubernetesResourceManagerConfig, update_handler: Arc<H>) -> anyhow::Result<Self>
    where
        H: ResourceUpdateHandler<ActionsConfigMap> + Send + Sync + 'static,
    {
        let client = Client::try_from(config.kubeconfig)?;
        let api: Api<ActionsConfigMap> = Api::namespaced(client.clone(), config.namespace.as_str());
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

impl ResourceUpdateHandler<ActionsConfigMap> for Arc<ActionData> {
    fn handle_update(&self, event: Result<ActionsConfigMap, watcher::Error>) -> impl Future<Output = ()> + Send {
        async {
            if event.is_err() {
                warn!("Failed to handle update: {:?}", event);
            }
            self.handle_async(event.unwrap()).await
        }
    }
}

impl ActionData {
    pub async fn handle_async(&self, event: ActionsConfigMap) {
        event
            .data
            .stream()
            .for_each(move |result| async move {
                match result {
                    Ok((segments, action_uid)) => {
                        if let Err(e) = self.upsert(segments, action_uid.clone()).await {
                            warn!("Failed to upsert action: {}", e);
                        } else {
                            debug!("Successfully upserted action with UID: {}", action_uid);
                        }
                    }
                    Err(e) => warn!("Error processing action route: {}", e),
                }
            })
            .await
    }
}
