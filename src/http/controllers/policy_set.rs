pub mod models;

use crate::http::controllers::policy_set::models::PolicySetRegistration;
use crate::http::errors::*;
use crate::services::repositories::policy_repository::PolicyRepository;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use cedar_policy::PolicySet;
use log::error;
use std::str::FromStr;
use std::sync::Arc;

#[utoipa::path(context_path = "/policy_set/", responses((status = OK)), request_body = PolicySetRegistration)]
#[post("{id}")]
async fn post_policy_set(
    id: Path<String>,
    request: Json<PolicySetRegistration>,
    data: Data<Arc<PolicyRepository>>,
) -> Result<impl Responder> {
    let policy_set = PolicySet::from_str(&request.into_inner().policy).map_err(anyhow::Error::from)?;
    data.upsert(id.to_string(), policy_set).await.map_err(|e| {
        error!("Failed ot insert policy_set: {:?}", e);
        e
    })?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/policy_set/", responses((status = OK, body = PolicySetRegistration)))]
#[get("{id}")]
async fn get_policy_set(id: Path<String>, data: Data<Arc<PolicyRepository>>) -> Result<impl Responder> {
    let policy_set = data.get(id.to_string()).await?;
    let policy_set = PolicySetRegistration {
        policy: policy_set.to_string(),
    };
    Ok(Json(policy_set))
}

#[utoipa::path(context_path = "/policy_set/", responses((status = OK)))]
#[delete("{id}")]
async fn delete_policy_set(id: Path<String>, data: Data<Arc<PolicyRepository>>) -> Result<impl Responder> {
    data.delete(id.to_string()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/policy_set")
        .service(post_policy_set)
        .service(get_policy_set)
        .service(delete_policy_set)
}
