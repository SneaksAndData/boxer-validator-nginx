use crate::services::repositories::models::PathSegment::{Parameter, Static};
use crate::services::repositories::models::RequestSegment::{Path, Verb};
use crate::services::repositories::models::{HTTPMethod, RequestSegment};
use cedar_policy::EntityUid;
use futures::Stream;
use futures::StreamExt;
use futures_util::stream;
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::Resource;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
                segments.push(Path(Parameter))
            } else {
                segments.push(Path(Static(segment.to_string())))
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
