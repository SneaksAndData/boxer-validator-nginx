use actix_web::error::ErrorUnauthorized;
use actix_web::web::{Data, ReqData};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, get, web};
use boxer_core::contracts::internal_token::v2::boxer_claims::BoxerClaims;
use boxer_core::http::middleware::audit::audit_recorder::audit_writer::AuditWriter;
use boxer_core::http::middleware::audit::audit_scope::AuditScope;
use boxer_core::services::audit::chained::audit_event::AuditEvent;
use boxer_core::services::token_decryption_service::TokenDecryptionService;
use boxer_core::services::validation_service::ValidationService;
use boxer_core::services::validation_service::request_context::RequestContext;
use log::error;
use std::sync::Arc;

#[utoipa::path(
    context_path = "/token",
    responses((status = OK)),
    security(
        ("internal" = [])
    )
)]
#[get("/review")]
async fn token_review(
    boxer_claims: ReqData<BoxerClaims>,
    request_context: RequestContext,
    cedar_validation_service: Data<Arc<dyn ValidationService<BoxerClaims>>>,
    http_request: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let mut extensions = http_request.extensions_mut();
    let event = extensions.get_mut::<AuditEvent>().ok_or_else(|| {
        error!("AuditEvent not found in request extensions");
        ErrorUnauthorized("No audit event found in request extensions")
    })?;
    cedar_validation_service
        .validate(boxer_claims.into_inner(), request_context, event)
        .await
        .map_err(ErrorUnauthorized)?;
    Ok(HttpResponse::Ok().finish())
}

pub fn routes(
    audit_service: Arc<dyn AuditWriter>,
    decryptor: Arc<TokenDecryptionService>,
) -> impl actix_web::dev::HttpServiceFactory {
    web::scope("/token")
        .service(token_review)
        .continue_audit_scope::<TokenDecryptionService>(audit_service, decryptor)
}
