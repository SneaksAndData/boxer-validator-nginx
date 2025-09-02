use crate::http::controllers;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

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
), modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme("internal", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)));
    }
}
