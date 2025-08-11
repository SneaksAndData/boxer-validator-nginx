pub mod models;

use crate::http::controllers::action_set::models::ActionSetRegistration;
use crate::services::repositories::action_repository::read_write::ActionDataRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, web, HttpResponse, Responder, Result};
use std::sync::Arc;

#[utoipa::path(context_path = "/action_set/", responses((status = OK)), request_body = ActionSetRegistration)]
#[post("{id}")]
async fn post_action_set(
    id: Path<String>,
    request: Json<ActionSetRegistration>,
    data: Data<Arc<ActionDataRepository>>,
) -> Result<impl Responder> {
    data.upsert(id.to_string(), request.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/action_set/", responses((status = OK, body = ActionSetRegistration)))]
#[get("{id}")]
async fn get_action_set(id: Path<String>, data: Data<Arc<ActionDataRepository>>) -> Result<impl Responder> {
    let action_set = data.get(id.to_string()).await?;
    Ok(Json(action_set))
}

#[utoipa::path(context_path = "/action_set/", responses((status = OK)))]
#[delete("{id}")]
async fn delete_action_set(id: Path<String>, data: Data<Arc<ActionDataRepository>>) -> Result<impl Responder> {
    data.delete(id.to_string()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/action_set")
        .service(post_action_set)
        .service(get_action_set)
        .service(delete_action_set)
}
