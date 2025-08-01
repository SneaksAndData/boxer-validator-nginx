use crate::http::controllers;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    controllers::schema::get_schema,
    controllers::schema::post_schema,
    controllers::schema::delete_schema,
))]
pub struct ApiDoc;
