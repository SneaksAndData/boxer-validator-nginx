use crate::models::token::BoxerToken;
use crate::services::authorizer::Authorizer;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, HttpMessage};
use boxer_core::contracts::dynamic_claims_collection::DynamicClaimsCollection;
use boxer_core::services::audit::events::token_validation_event::TokenValidationEvent;
use boxer_core::services::audit::AuditService;
use boxer_core::services::observability::open_telemetry::tracing::{start_trace, ErrorExt};
use collection_macros::hashset;
use futures_util::future::LocalBoxFuture;
use log::{debug, error};
use opentelemetry::context::FutureExt;
use std::collections::HashSet;
use std::sync::Arc;

/// Middleware for external token validation factory
pub struct InternalTokenMiddlewareFactory {
    pub audit_service: Arc<dyn AuditService>,
    pub authorizer: Arc<Authorizer>,
}

/// The ExternalTokenMiddlewareFactory's own methods implementation
impl InternalTokenMiddlewareFactory {
    pub(crate) fn new(authorizer: Arc<Authorizer>, audit_service: Arc<dyn AuditService>) -> Self {
        InternalTokenMiddlewareFactory {
            audit_service,
            authorizer,
        }
    }
}

/// Transform trait implementation
/// `NextServiceType` - type of the next service
/// `BodyType` - type of response's body
impl<NextService, BodyType> Transform<NextService, ServiceRequest> for InternalTokenMiddlewareFactory
where
    NextService: Service<ServiceRequest, Response = ServiceResponse<BodyType>, Error = Error> + 'static,
    NextService::Future: 'static,
    BodyType: 'static,
{
    type Response = ServiceResponse<BodyType>;
    type Error = Error;
    type Transform = JwtAuthorizerMiddleware<NextService>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<JwtAuthorizerMiddleware<NextService>, Self::InitError>>;

    fn new_transform(&self, service: NextService) -> Self::Future {
        let audit_service = self.audit_service.clone();
        let authorizer = self.authorizer.clone();
        Box::pin(async move {
            let mw = JwtAuthorizerMiddleware {
                service: Arc::new(service),
                audit_service,
                authorizer,
            };
            Ok(mw)
        })
    }
}

/// The middleware object
pub struct JwtAuthorizerMiddleware<NextService> {
    service: Arc<NextService>,
    pub audit_service: Arc<dyn AuditService>,
    pub authorizer: Arc<Authorizer>,
}

impl<Next> JwtAuthorizerMiddleware<Next> {
    fn get_token(req: &ServiceRequest) -> Result<String, Error> {
        let token_value = req
            .headers()
            .get("Authorization")
            .ok_or(ErrorUnauthorized("Unauthorized"))?;

        let boxer_token = BoxerToken::try_from(token_value).map_err(|_| ErrorUnauthorized("Unauthorized"))?;
        Ok(boxer_token.token)
    }

    async fn authorize_with_authorizer(
        boxer_token: &BoxerToken,
        authorizer: Arc<Authorizer>,
    ) -> Result<DynamicClaimsCollection, anyhow::Error> {
        let ctx = start_trace("authorizer_check", None);
        authorizer.validate(&boxer_token.token).stop_trace(ctx)
    }
}

/// The middleware implementation
impl<NextService, BodyType> Service<ServiceRequest> for JwtAuthorizerMiddleware<NextService>
where
    NextService: Service<ServiceRequest, Response = ServiceResponse<BodyType>, Error = Error> + 'static,
    NextService::Future: 'static,
    BodyType: 'static,
{
    type Response = ServiceResponse<BodyType>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    // Asynchronously handle the request and bypass it to the next service
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Clone the service and validator to be able to use them in the async block
        let service = Arc::clone(&self.service);
        let audit_service = Arc::clone(&self.audit_service);
        let authorizer = self.authorizer.clone();
        // The async block that will be executed when the middleware is called
        let future = async move {
            let parent = start_trace("internal_token_validation", None);
            let boxer_token: BoxerToken = Self::get_token(&req)?.try_into()?;
            let validation_result = Self::authorize_with_authorizer(&boxer_token, authorizer)
                .with_context(parent.clone())
                .await;

            let event = TokenValidationEvent::internal(
                &boxer_token.token,
                validation_result.is_ok(),
                extract_validation_reason(&validation_result),
            );

            audit_service.record_token_validation(event).map_err(|err| {
                error!("Failed to audit token validation: {}", err);
                ErrorUnauthorized("Unauthorized")
            })?;
            debug!("Token validated successfully");

            match validation_result {
                Err(_) => {
                    return Err(ErrorUnauthorized("Unauthorized"));
                }
                Ok(claims) => {
                    // make nested block to avoid borrowing issues
                    {
                        let mut ext = req.extensions_mut();
                        ext.insert(claims);
                    }
                    let res = service
                        .call(req)
                        .with_context(parent.clone())
                        .await
                        .stop_trace(parent)?;
                    Ok(res)
                }
            }
        };
        Box::pin(future)
    }
}

fn extract_validation_reason(result: &Result<DynamicClaimsCollection, anyhow::Error>) -> HashSet<String> {
    match result {
        Ok(_) => HashSet::new(),
        Err(e) => hashset![e.to_string()],
    }
}
