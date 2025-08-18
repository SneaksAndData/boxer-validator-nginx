pub mod models;

use crate::http::controllers::policy_set::models::PolicySetRegistration;
use crate::services::repositories::policy_repository::read_write::PolicyDataRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::Result;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use std::sync::Arc;

#[utoipa::path(context_path = "/policy_set/", responses((status = OK)), request_body = PolicySetRegistration)]
#[post("{schema}/{id}")]
async fn post_policy_set(
    id: Path<(String, String)>,
    request: Json<PolicySetRegistration>,
    data: Data<Arc<PolicyDataRepository>>,
) -> Result<impl Responder> {
    data.upsert(id.into_inner(), request.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/policy_set/", responses((status = OK, body = PolicySetRegistration)))]
#[get("{schema}/{id}")]
async fn get_policy_set(id: Path<(String, String)>, data: Data<Arc<PolicyDataRepository>>) -> Result<impl Responder> {
    let policy_set = data.get(id.into_inner()).await?;
    Ok(Json(policy_set))
}

#[utoipa::path(context_path = "/policy_set/", responses((status = OK)))]
#[delete("{schema}/{id}")]
async fn delete_policy_set(
    id: Path<(String, String)>,
    data: Data<Arc<PolicyDataRepository>>,
) -> Result<impl Responder> {
    data.delete(id.into_inner()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/policy_set")
        .service(post_policy_set)
        .service(get_policy_set)
        .service(delete_policy_set)
}
