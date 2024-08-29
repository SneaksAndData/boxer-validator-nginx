mod http;
mod models;

use crate::http::filters::jwt_filter::InternalTokenMiddlewareFactory;
use crate::http::urls::token_review;
use actix_web::{App, HttpServer};
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");

    env_logger::init();
    let addr = ("127.0.0.1", 8081);

    println!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || {
        App::new()
            .wrap(InternalTokenMiddlewareFactory::new())
            .service(token_review)
    })
    .bind(addr)?
    .run()
    .await
}
