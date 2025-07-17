use crate::services::repositories::models::PathSegment::{Parameter, Static};
use crate::services::repositories::models::{HTTPMethod, PathSegment};
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
struct ResourceRoute {
    method: HTTPMethod,
    route_template: String,
    resource_uid: String,
}

impl TryInto<Vec<PathSegment>> for ResourceRoute {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Vec<PathSegment>, Self::Error> {
        let mut segments: Vec<PathSegment> = vec![];
        for segment in self.route_template.split('/') {
            if segment.is_empty() {
                continue;
            }
            if segment.starts_with('{') && segment.ends_with('}') {
                segments.push(Parameter)
            } else {
                segments.push(Static(segment.to_string()))
            }
        }
        Ok(segments)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResourceDiscoveryDocument {
    hostname: String,
    routes: Vec<ResourceRoute>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResourceDiscoveryDocumentData {
    pub(crate) resources: String,
}

impl ResourceDiscoveryDocument {
    pub fn stream(self) -> impl Stream<Item = Result<(Vec<PathSegment>, EntityUid), anyhow::Error>> {
        stream::iter(self.routes).map(move |route| {
            let action_uid: EntityUid = EntityUid::from_str(&route.resource_uid).map_err(anyhow::Error::from)?;
            let mut key: Vec<PathSegment> = vec![];
            let segments: Vec<PathSegment> = route.try_into()?;
            key.extend(segments);
            Ok((key, action_uid))
        })
    }
}

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[resource(inherit = ConfigMap)]
pub struct ResourcesDiscoveryResource {
    metadata: ObjectMeta,
    pub data: ResourceDiscoveryDocumentData,
}
