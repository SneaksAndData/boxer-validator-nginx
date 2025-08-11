pub mod models;

use crate::http::controllers::resource_set::models::ResourceSetRegistration;
use crate::services::repositories::resource_repository::read_write::ResourceDiscoveryDocumentRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, web, HttpResponse, Responder, Result};
use std::sync::Arc;

#[utoipa::path(context_path = "/resource_set/", responses((status = OK)), request_body = ResourceSetRegistration)]
#[post("{id}")]
async fn post_resource_set(
    id: Path<String>,
    request: Json<ResourceSetRegistration>,
    data: Data<Arc<ResourceDiscoveryDocumentRepository>>,
) -> Result<impl Responder> {
    data.upsert(id.to_string(), request.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/resource_set/", responses((status = OK, body = ResourceSetRegistration)))]
#[get("{id}")]
async fn get_resource_set(
    id: Path<String>,
    data: Data<Arc<ResourceDiscoveryDocumentRepository>>,
) -> Result<impl Responder> {
    let resource_set = data.get(id.to_string()).await?;
    Ok(Json(resource_set))
}

#[utoipa::path(context_path = "/resource_set/", responses((status = OK)))]
#[delete("{id}")]
async fn delete_resource_set(
    id: Path<String>,
    data: Data<Arc<ResourceDiscoveryDocumentRepository>>,
) -> Result<impl Responder> {
    data.delete(id.to_string()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/resource_set")
        .service(post_resource_set)
        .service(get_resource_set)
        .service(delete_resource_set)
}
