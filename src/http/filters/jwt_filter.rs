use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::ErrorUnauthorized;
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use jwt_authorizer::{Authorizer, JwtAuthorizer, Validation};
use std::sync::Arc;
use log::{debug, error};
use crate::models::token::BoxerToken;
use crate::models::validation_settings::ValidationSettings;

/// Middleware for external token validation factory
pub struct InternalTokenMiddlewareFactory {
}

/// The ExternalTokenMiddlewareFactory's own methods implementation
impl InternalTokenMiddlewareFactory {
    pub(crate) fn new() -> Self {
        InternalTokenMiddlewareFactory {
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
        Box::pin( async move { 
                let settings = ValidationSettings::new();
                let mut validation = Validation::new();
                validation.iss = Some(settings.valid_issuers.clone());
                validation.aud = Some(settings.valid_audiences.clone());
                
                // It's OK to unwrap here because we should panic if cannot build the authorizer
                let authorizer = JwtAuthorizer::from_secret(settings.secret)
                    .validation(validation)
                    .build()
                    .await
                    .unwrap(); 
                let mw = JwtAuthorizerMiddleware { service: Arc::new(service), authorizer: Arc::new(authorizer) };
                Ok(mw)
            }
        )
    }
}

/// The middleware object
pub struct JwtAuthorizerMiddleware<NextService> {
    service: Arc<NextService>,
    authorizer: Arc<Authorizer>,
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
        Box::pin(async move {
            let token_value = req
                .headers()
                .get("Authorization").ok_or(ErrorUnauthorized("Unauthorized"))?;
                

            let boxer_token = BoxerToken::try_from(token_value).map_err(|_| ErrorUnauthorized("Unauthorized"))?;
            let validation_result = authorizer.check_auth(boxer_token.token.clone().as_str()).await;
            if validation_result.is_err() {
                error!("Failed to validate token: {:?}", validation_result.err());
                return Err(ErrorUnauthorized("Unauthorized"));
            }
            debug!("Token validated successfully");

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}
