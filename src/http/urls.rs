use actix_web::{get, HttpRequest};

// Dummy implementation of the token endpoint
#[get("/token/review")]
async fn token_review(_: HttpRequest) -> String {
    "dummy endpoint".to_string()
}
