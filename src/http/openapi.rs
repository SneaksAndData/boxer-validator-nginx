use crate::http::controllers;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    controllers::schema::get_schema,
    controllers::schema::post_schema,
    controllers::schema::delete_schema,
    controllers::action_set::get_action_set,
    controllers::action_set::post_action_set,
    controllers::action_set::delete_action_set,
    controllers::resource_set::get_resource_set,
    controllers::resource_set::post_resource_set,
    controllers::resource_set::delete_resource_set,
    controllers::policy_set::get_policy_set,
    controllers::policy_set::post_policy_set,
    controllers::policy_set::delete_policy_set,
    controllers::token_review::token_review,
))]
pub struct ApiDoc;
