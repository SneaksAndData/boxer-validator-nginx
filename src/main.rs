mod http;
mod models;

use crate::http::filters::jwt_filter::InternalTokenMiddlewareFactory;
use crate::http::urls::token_review;
use actix_web::{App, HttpServer};
use jwt_authorizer::{Authorizer, AuthorizerBuilder, IntoLayer, JwtAuthorizer, RegisteredClaims};
use std::io::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

#[actix_web::main]
async fn main() -> Result<()> {
    let addr = ("127.0.0.1", 8081);

    println!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || {
        App::new()
            .wrap(InternalTokenMiddlewareFactory::new("secret"))
            .service(token_review)
    })
    .bind(addr)?
    .run()
    .await
}
