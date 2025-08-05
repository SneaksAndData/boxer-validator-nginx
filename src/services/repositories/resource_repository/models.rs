use crate::services::repositories::models::PathSegment;
use crate::services::repositories::models::PathSegment::{Parameter, Static};
use cedar_policy::EntityUid;
use futures::Stream;
use futures::StreamExt;
use futures_util::stream;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRoute {
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

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "auth.sneaksanddata.com",
    version = "v1beta1",
    kind = "ResourceDiscoveryDocument",
    plural = "resource-discovery-documents",
    singular = "resource-discovery-document",
    namespaced
)]

pub struct ResourceDiscoveryDocumentSpec {
    pub active: bool,
    pub hostname: String,
    pub routes: Vec<ResourceRoute>,
}

impl ResourceDiscoveryDocumentSpec {
    #[allow(dead_code)]
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
