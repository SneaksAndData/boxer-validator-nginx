use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ResourceRouteRegistration {
    pub method: String,
    pub route_template: String,
    pub action_uid: String,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct ResourceSetRegistration {
    pub hostname: String,
    pub routes: Vec<ResourceRouteRegistration>,
}
