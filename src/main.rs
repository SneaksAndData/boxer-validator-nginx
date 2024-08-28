mod http;

use crate::http::urls::token_review;
use actix_web::{App, HttpServer};
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    let addr = ("127.0.0.1", 8080);
    println!("listening on {}:{}", &addr.0, &addr.1);
    HttpServer::new(move || App::new().service(token_review))
        .bind(addr)?
        .run()
        .await
}
