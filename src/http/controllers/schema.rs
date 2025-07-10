use crate::http::errors::*;
use actix_web::dev::HttpServiceFactory;
use actix_web::web::{Data, Path};
use actix_web::{delete, get, post, web, HttpResponse};
use boxer_core::services::base::types::SchemaRepository;
use cedar_policy::SchemaFragment;
use std::sync::Arc;

#[utoipa::path(context_path = "/schema/", responses((status = OK)))]
#[post("{id}")]
async fn post(id: Path<String>, schema_json: String, data: Data<Arc<SchemaRepository>>) -> Result<HttpResponse> {
    let schema = SchemaFragment::from_json_str(&schema_json)?;
    data.upsert(id.to_string(), schema).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(context_path = "/schema/", responses((status = OK)))]
#[get("{id}")]
async fn get(id: Path<String>, data: Data<Arc<SchemaRepository>>) -> Result<String> {
    let schema = data.get(id.to_string()).await?;
    let result = schema.to_json_string()?;
    Ok(result)
}

#[utoipa::path(context_path = "/schema/", responses((status = OK)))]
#[delete("{id}")]
async fn delete(id: Path<String>, data: Data<Arc<SchemaRepository>>) -> Result<HttpResponse> {
    data.delete(id.to_string()).await?;
    Ok(HttpResponse::Ok().finish())
}

pub fn crud() -> impl HttpServiceFactory {
    web::scope("/schema").service(post).service(get).service(delete)
}
