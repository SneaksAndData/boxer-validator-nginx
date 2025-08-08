use crate::http::controllers::action_set::models::{ActionRouteRegistration, ActionSetRegistration};
use crate::services::repositories::models::PathSegment::{Parameter, Static};
use crate::services::repositories::models::RequestSegment::{Path, Verb};
use crate::services::repositories::models::{HTTPMethod, RequestSegment};
use cedar_policy::EntityUid;
use futures::Stream;
use futures::StreamExt;
use futures_util::stream;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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

impl ActionDiscoveryDocumentSpec {
    pub fn stream(self) -> impl Stream<Item = Result<(Vec<RequestSegment>, EntityUid), anyhow::Error>> {
        let active = stream::repeat(self.active);
        stream::iter(self.routes).zip(active).map(move |(route, active)| {
            let action_uid: EntityUid = EntityUid::from_str(&route.action_uid).map_err(anyhow::Error::from)?;
            let mut key: Vec<RequestSegment> = vec![RequestSegment::Hostname(self.hostname.clone())];
            let segments: Vec<RequestSegment> = route.try_into()?;
            key.extend(segments);
            if !active {
                return Err(anyhow::anyhow!("ActionDiscoveryDocument is not active"));
            }
            Ok((key, action_uid))
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
