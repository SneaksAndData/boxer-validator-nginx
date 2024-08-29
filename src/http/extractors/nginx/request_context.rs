use crate::models::request_context::RequestContext;
use actix_web::error::ErrorBadRequest;
use actix_web::{FromRequest, HttpRequest};
use futures_util::future::{ready, Ready};

const ORIGINAL_URL_HEADER: &str = "X-Original-URL";
const ORIGINAL_METHOD_HEADER: &str = "X-Original-Method";

impl FromRequest for RequestContext {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let result = match extract_headers(req) {
            Ok((original_url, original_method)) => {
                let request_context = RequestContext::new(original_url, original_method);
                Ok(request_context)
            }
            Err(e) => Err(ErrorBadRequest(e.to_string())),
        };
        ready(result)
    }
}

fn extract_headers(req: &HttpRequest) -> anyhow::Result<(String, String)> {
    let original_url = extract_header(req, ORIGINAL_URL_HEADER)?;
    let original_method = extract_header(req, ORIGINAL_METHOD_HEADER)?;
    Ok((original_url, original_method))
}

fn extract_header(req: &HttpRequest, header_name: &'static str) -> anyhow::Result<String> {
    let header_value = req
        .headers()
        .get(header_name)
        .ok_or(anyhow::Error::msg("Missing original URL header").context(header_name))?
        .to_str()?
        .to_owned();
    Ok(header_value)
}
