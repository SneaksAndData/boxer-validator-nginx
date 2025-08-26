use crate::models::token::BoxerToken;
use crate::models::validation_settings::ValidationSettings;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, HttpMessage};
use boxer_core::services::observability::open_telemetry::tracing::{start_trace, ErrorExt};
use futures_util::future::LocalBoxFuture;
use jwt_authorizer::{Authorizer, JwtAuthorizer, Validation};
use log::debug;
use opentelemetry::context::FutureExt;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Middleware for external token validation factory
pub struct InternalTokenMiddlewareFactory {}

/// The ExternalTokenMiddlewareFactory's own methods implementation
impl InternalTokenMiddlewareFactory {
    pub(crate) fn new() -> Self {
        InternalTokenMiddlewareFactory {}
    }
}

pub type DynamicClaimsCollection = HashMap<String, Value>;

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
        Box::pin(async move {
            let settings = ValidationSettings::new();
            let mut validation = Validation::new();
            validation.iss = Some(settings.valid_issuers.clone());
            validation.aud = Some(settings.valid_audiences.clone());

            // It's OK to unwrap here because we should panic if cannot build the authorizer
            let authorizer: Authorizer<DynamicClaimsCollection> = JwtAuthorizer::from_secret(settings.secret)
                .validation(validation)
                .build()
                .await
                .expect("Failed to build JwtAuthorizer.");
            let mw = JwtAuthorizerMiddleware {
                service: Arc::new(service),
                authorizer: Arc::new(authorizer),
            };
            Ok(mw)
        })
    }
}

/// The middleware object
pub struct JwtAuthorizerMiddleware<NextService> {
    service: Arc<NextService>,
    authorizer: Arc<Authorizer<DynamicClaimsCollection>>,
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
        authorizer: Arc<Authorizer<DynamicClaimsCollection>>,
        boxer_token: &BoxerToken,
    ) -> Result<DynamicClaimsCollection, Error> {
        let ctx = start_trace("authorizer_check");
        let token_data = authorizer
            .check_auth(&boxer_token.token)
            .with_context(ctx.clone())
            .await
            .stop_trace(ctx)
            .map_err(|_| ErrorUnauthorized("Unauthorized"))?;
        Ok(token_data.claims)
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
        let authorizer = Arc::clone(&self.authorizer);
        // The async block that will be executed when the middleware is called
        let future = async move {
            let parent = start_trace("internal_token_validation");
            let boxer_token: BoxerToken = Self::get_token(&req)?.try_into()?;
            let validation_result = Self::authorize_with_authorizer(authorizer, &boxer_token)
                .with_context(parent.clone())
                .await?;
            debug!("Token validated successfully");

            // make nested block to avoid borrowing issues
            {
                let mut ext = req.extensions_mut();
                ext.insert(validation_result);
            }
            let res = service
                .call(req)
                .with_context(parent.clone())
                .await
                .stop_trace(parent)?;
            Ok(res)
        };
        Box::pin(future)
    }
}
