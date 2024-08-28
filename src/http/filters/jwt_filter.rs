use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::ErrorUnauthorized;
use actix_web::Error;
use futures_util::future::LocalBoxFuture;
use jwt_authorizer::{Authorizer, JwtAuthorizer};
use std::sync::Arc;

// Middleware for external token validation factory
pub struct InternalTokenMiddlewareFactory {
    secret: &'static str
}

// The ExternalTokenMiddlewareFactory's own methods implementation
impl InternalTokenMiddlewareFactory {
    pub(crate) fn new(secret: &'static str) -> Self {
        InternalTokenMiddlewareFactory {
            secret
        }
    }
}

// Transform trait implementation
// `NextServiceType` - type of the next service
// `BodyType` - type of response's body
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
        Box::pin( async { 
                let authorizer = JwtAuthorizer::from_secret(self.secret).build().await.unwrap();
                let mw = JwtAuthorizerMiddleware { service: Arc::new(service), authorizer: Arc::new(authorizer) };
                Ok(mw)
            }
        )
    }
}

// The middleware object
pub struct JwtAuthorizerMiddleware<NextService> {
    service: Arc<NextService>,
    authorizer: Arc<Authorizer>,
}

// The middleware implementation
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
            let token = req
                .headers()
                .get("Authorization")
                .unwrap()
                .to_str()
                .unwrap(); // TODO

            let validation_result = authorizer.check_auth(token).await;
            if validation_result.is_err() {
                return Err(ErrorUnauthorized("Unauthorized"));
            }

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}
