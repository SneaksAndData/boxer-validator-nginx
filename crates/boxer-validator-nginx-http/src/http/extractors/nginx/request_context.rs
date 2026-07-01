use crate::models::request_context::RequestContext;
use actix_web::error::ErrorBadRequest;
use actix_web::{FromRequest, HttpRequest};
use boxer_core::services::observability::open_telemetry::tracing::{ErrorExt, start_trace};
use futures_util::future::{Ready, ready};

const ORIGINAL_URL_NGINX_HEADER: &str = "X-Original-URL";
const ORIGINAL_METHOD_NGINX_HEADER: &str = "X-Original-Method";

const ORIGINAL_METHOD_TRAEFIK_HEADER: &str = "X-Forwarded-Method";
const ORIGINAL_PROTOCOL_TRAEFIK_HEADER: &str = "X-Forwarded-Proto";
const ORIGINAL_HOST_TRAEFIK_HEADER: &str = "X-Forwarded-Host";
const ORIGINAL_URL_TRAEFIK_HEADER: &str = "X-Forwarded-Uri";

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
    let original_url = extract_url(req, ORIGINAL_URL_NGINX_HEADER)?;
    let original_method = extract_header(req, ORIGINAL_METHOD_NGINX_HEADER, ORIGINAL_METHOD_TRAEFIK_HEADER)?;
    Ok((original_url, original_method))
}

fn extract_header(req: &HttpRequest, header_name: &'static str, fallback: &'static str) -> anyhow::Result<String> {
    let header = req.headers().get(header_name);
    let fallback_header = req.headers().get(fallback);
    let result = match header {
        Some(value) => Some(value.to_str()?.to_string()),
        None => match fallback_header {
            Some(value) => Some(value.to_str()?.to_string()),
            None => None,
        },
    };

    Ok(result.ok_or_else(|| anyhow::anyhow!("Missing required header: {header_name} or {fallback} in request"))?)
}

fn extract_url(req: &HttpRequest, header_name: &'static str) -> anyhow::Result<String> {
    let header = req.headers().get(header_name);
    let result = match header {
        Some(value) => Some(value.to_str()?.to_string()),
        None => match extract_traefik_headers(req) {
            Some(value) => Some(value),
            None => None,
        },
    };

    Ok(result.ok_or_else(|| anyhow::anyhow!("Missing required headers"))?)
}

fn extract_traefik_headers(req: &HttpRequest) -> Option<String> {
    let headers = req.headers();

    let proto = headers.get(ORIGINAL_PROTOCOL_TRAEFIK_HEADER)?.to_str().ok()?;
    let host = headers.get(ORIGINAL_HOST_TRAEFIK_HEADER)?.to_str().ok()?;
    let uri = headers.get(ORIGINAL_URL_TRAEFIK_HEADER)?.to_str().ok()?;

    let uri = uri.trim_start_matches('/');
    Some(format!("{proto}://{host}/{uri}"))
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
