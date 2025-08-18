use crate::http::controllers::action_set::models::{ActionRouteRegistration, ActionSetRegistration};
use crate::services::repositories::lookup_trie::EntityCollectionResource;
use crate::services::repositories::models::http_method::HTTPMethod;
use crate::services::repositories::models::path_segment::PathSegment::{Parameter, Static};
use crate::services::repositories::models::request_segment::RequestSegment;
use crate::services::repositories::models::request_segment::RequestSegment::{Hostname, Path, Verb};
use boxer_core::services::backends::kubernetes::kubernetes_resource_manager::UpdateLabels;
use boxer_core::services::backends::kubernetes::repositories::SoftDeleteResource;
use cedar_policy::EntityUid;
use futures::Stream;
use futures::StreamExt;
use futures_util::stream;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActionRoute {
    pub method: HTTPMethod,
    pub route_template: String,
    pub action_uid: String,
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
                segments.push(Path(Parameter))
            } else {
                segments.push(Path(Static(segment.to_string())))
            }
        }
        Ok(segments)
    }
}

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "ActionDiscoveryDocument",
    plural = "action-discovery-documents",
    singular = "action-discovery-document",
    namespaced
)]

pub struct ActionDiscoveryDocumentSpec {
    pub active: bool,
    pub hostname: String,
    pub routes: Vec<ActionRoute>,
}

impl EntityCollectionResource<RequestSegment> for ActionDiscoveryDocument {
    fn stream(self) -> impl Stream<Item = Result<(Vec<RequestSegment>, EntityUid, bool), anyhow::Error>> + Send + Sync {
        let hostname = self.spec.hostname.clone();
        let active = self.spec.active;
        stream::iter(self.spec.routes)
            .zip(stream::repeat(hostname))
            .zip(stream::repeat(active))
            .map(move |((route, hostname), active)| {
                let action_uid: EntityUid = EntityUid::from_str(&route.action_uid).map_err(anyhow::Error::from)?;
                let mut key: Vec<RequestSegment> = vec![Hostname(hostname)];
                let segments: Vec<RequestSegment> = route.try_into()?;
                key.extend(segments);
                Ok((key, action_uid, active))
            })
    }
}

impl Default for ActionDiscoveryDocument {
    fn default() -> Self {
        ActionDiscoveryDocument {
            metadata: ObjectMeta::default(),
            spec: ActionDiscoveryDocumentSpec::default(),
        }
    }
}

impl TryFrom<&ActionSetRegistration> for ActionDiscoveryDocumentSpec {
    type Error = anyhow::Error;

    fn try_from(value: &ActionSetRegistration) -> Result<Self, Self::Error> {
        let mut routes = Vec::<ActionRoute>::new();

        for route in &value.routes {
            let method = HTTPMethod::from_str(&route.method)?;
            let action_route = ActionRoute {
                method,
                route_template: route.route_template.clone(),
                action_uid: route.action_uid.to_string(),
            };
            routes.push(action_route)
        }
        Ok(ActionDiscoveryDocumentSpec {
            active: true,
            hostname: value.hostname.clone(),
            routes,
        })
    }
}

impl Into<ActionSetRegistration> for ActionDiscoveryDocumentSpec {
    fn into(self) -> ActionSetRegistration {
        let routes: Vec<ActionRouteRegistration> = self
            .routes
            .into_iter()
            .map(|route| ActionRouteRegistration {
                method: route.method.to_string(),
                route_template: route.route_template,
                action_uid: route.action_uid,
            })
            .collect();

        ActionSetRegistration {
            hostname: self.hostname,
            routes,
        }
    }
}

impl SoftDeleteResource for ActionDiscoveryDocument {
    fn is_deleted(&self) -> bool {
        !self.spec.active
    }

    fn set_deleted(&mut self) {
        self.spec.active = false;
    }

    fn clear_managed_fields(&mut self) {
        self.metadata.managed_fields = None;
    }
}

impl UpdateLabels for ActionDiscoveryDocument {
    fn update_labels(mut self, custom_labels: &mut BTreeMap<String, String>) -> Self {
        let mut labels = self.metadata.labels.unwrap_or_default();
        labels.append(custom_labels);
        self.metadata.labels = Some(labels);
        self
    }
}
