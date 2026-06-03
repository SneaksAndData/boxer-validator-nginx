use crate::models::request_context::RequestContext;
use actix_web::error::ErrorBadRequest;
use actix_web::{FromRequest, HttpRequest};
use boxer_core::services::observability::open_telemetry::tracing::{start_trace, ErrorExt};
use futures_util::future::{ready, Ready};

const ORIGINAL_URL_NGINX_HEADER: &str = "X-Original-URL";
const ORIGINAL_URL_TRAEFIK_HEADER: &str = "X-Forwarded-Uri";
const ORIGINAL_METHOD_NGINX_HEADER: &str = "X-Original-Method";
const ORIGINAL_METHOD_TRAEFIK_HEADER: &str = "X-Forwarded-Method";

impl FromRequest for RequestContext {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let cx = start_trace("extract_request_context", None);
        let result = extract_headers(req)
            .stop_trace(cx)
            .map(|(url, method)| RequestContext::new(url, method))
            .map_err(|err| ErrorBadRequest(err.to_string()));

        ready(result)
    }
}

fn extract_headers(req: &HttpRequest) -> anyhow::Result<(String, String)> {
    let original_url = extract_header(req, ORIGINAL_URL_NGINX_HEADER, ORIGINAL_URL_TRAEFIK_HEADER)?;
    let original_method = extract_header(req, ORIGINAL_METHOD_NGINX_HEADER, ORIGINAL_METHOD_TRAEFIK_HEADER)?;
    Ok((original_url, original_method))
}

fn extract_header(req: &HttpRequest, header_name: &'static str, fallback: &'static str) -> anyhow::Result<String> {
    let ctx = format!("{0}|{1}", header_name, fallback);
    let header_value = req
        .headers()
        .get(header_name)
        .or_else(|| req.headers().get(fallback))
        .ok_or(anyhow::Error::msg("Missing original URL header").context(ctx))?
        .to_str()?
        .to_owned();
    Ok(header_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    fn make_req(headers: &[(&str, &str)]) -> HttpRequest {
        let mut builder = TestRequest::get();
        for (name, value) in headers {
            builder = builder.insert_header((*name, *value));
        }
        builder.to_http_request()
    }

    // extract_header tests

    #[test]
    fn test_extract_header_primary() {
        let req = make_req(&[(ORIGINAL_URL_NGINX_HEADER, "http://example.com/path")]);
        let result = extract_header(&req, ORIGINAL_URL_NGINX_HEADER, ORIGINAL_URL_TRAEFIK_HEADER);
        assert_eq!(result.unwrap(), "http://example.com/path");
    }

    #[test]
    fn test_extract_header_fallback() {
        let req = make_req(&[(ORIGINAL_URL_TRAEFIK_HEADER, "/traefik-path")]);
        let result = extract_header(&req, ORIGINAL_URL_NGINX_HEADER, ORIGINAL_URL_TRAEFIK_HEADER);
        assert_eq!(result.unwrap(), "/traefik-path");
    }

    #[test]
    fn test_extract_header_primary_takes_precedence_over_fallback() {
        let req = make_req(&[
            (ORIGINAL_URL_NGINX_HEADER, "http://primary.com"),
            (ORIGINAL_URL_TRAEFIK_HEADER, "http://fallback.com"),
        ]);
        let result = extract_header(&req, ORIGINAL_URL_NGINX_HEADER, ORIGINAL_URL_TRAEFIK_HEADER);
        assert_eq!(result.unwrap(), "http://primary.com");
    }

    #[test]
    fn test_extract_header_missing_returns_error() {
        let req = make_req(&[]);
        let result = extract_header(&req, ORIGINAL_URL_NGINX_HEADER, ORIGINAL_URL_TRAEFIK_HEADER);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(ORIGINAL_URL_NGINX_HEADER));
    }

    // extract_headers tests

    #[test]
    fn test_extract_headers_nginx_headers() {
        let req = make_req(&[
            (ORIGINAL_URL_NGINX_HEADER, "http://example.com/api"),
            (ORIGINAL_METHOD_NGINX_HEADER, "GET"),
        ]);
        let (url, method) = extract_headers(&req).unwrap();
        assert_eq!(url, "http://example.com/api");
        assert_eq!(method, "GET");
    }

    #[test]
    fn test_extract_headers_traefik_headers() {
        let req = make_req(&[
            (ORIGINAL_URL_TRAEFIK_HEADER, "/forwarded-uri"),
            (ORIGINAL_METHOD_TRAEFIK_HEADER, "POST"),
        ]);
        let (url, method) = extract_headers(&req).unwrap();
        assert_eq!(url, "/forwarded-uri");
        assert_eq!(method, "POST");
    }

    #[test]
    fn test_extract_headers_missing_url_returns_error() {
        let req = make_req(&[(ORIGINAL_METHOD_NGINX_HEADER, "GET")]);
        let result = extract_headers(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_headers_missing_method_returns_error() {
        let req = make_req(&[(ORIGINAL_URL_NGINX_HEADER, "http://example.com")]);
        let result = extract_headers(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_headers_both_missing_returns_error() {
        let req = make_req(&[]);
        let result = extract_headers(&req);
        assert!(result.is_err());
    }
}
