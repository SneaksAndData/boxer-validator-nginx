use crate::services::authorizer::Authorizer;
use actix_web::dev::HttpServiceFactory;
use actix_web::web;
use boxer_core::services::audit::AuditService;
use std::sync::Arc;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

pub mod action_set;
pub mod policy_set;
pub mod resource_set;
pub mod schema;
pub mod token_review;

#[derive(OpenApi)]
#[openapi(paths(
        schema::get_schema,
        schema::post_schema,
        schema::delete_schema,
        action_set::get_action_set,
        action_set::post_action_set,
        action_set::delete_action_set,
        resource_set::get_resource_set,
        resource_set::post_resource_set,
        resource_set::delete_resource_set,
        policy_set::get_policy_set,
        policy_set::post_policy_set,
        policy_set::delete_policy_set,
        token_review::token_review,
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiV1;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme("internal", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)));
    }
}
pub fn urls(
    production_mode: bool,
    authorizer: Arc<Authorizer>,
    audit_service: Arc<dyn AuditService>,
) -> impl HttpServiceFactory {
    web::scope("/api/v1")
        .service(schema::crud())
        .service(action_set::crud())
        .service(resource_set::crud())
        .service(policy_set::crud())
        .service(token_review::routes(production_mode, authorizer, audit_service))
}
