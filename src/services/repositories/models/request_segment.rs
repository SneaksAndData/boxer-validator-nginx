use crate::models::request_context::RequestContext;
use crate::services::prefix_tree::naive_tree::ParametrizedMatcher;
use crate::services::repositories::models::http_method::HTTPMethod;
use crate::services::repositories::models::path_segment::PathSegment;
use anyhow::anyhow;
use strum_macros::Display;
use url::Url;

/// Represents a segment of a HTTP request.
/// Each segment can be a verb (HTTP method), hostname, static string, or a parameter.
/// e.g., in the request `GET https://example.com/api/resource/{id}/property`, the segments would be:
///   - Verb: `GET`
///   - Hostname: `example.com`
///   - Static: `api`
///   - Static: `resource`
///   - Parameter: `{id}`
/// Does not include the query string or fragment.
#[derive(Debug, Clone, Display, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum RequestSegment {
    Hostname(String),
    Verb(HTTPMethod),
    Path(PathSegment),
}

impl ParametrizedMatcher for RequestSegment {
    fn is_parameter(&self) -> bool {
        matches!(self, RequestSegment::Path(PathSegment::Parameter))
    }
}

impl TryFrom<RequestContext> for Vec<RequestSegment> {
    type Error = anyhow::Error;

    fn try_from(context: RequestContext) -> Result<Self, Self::Error> {
        let verb = context.original_method.clone();
        let uri = Url::parse(context.original_url.as_str())?;
        let hostname = uri
            .host_str()
            .ok_or_else(|| anyhow!("Invalid URL: missing host"))?
            .to_string();
        let mut segments = vec![];
        let method = verb.parse::<HTTPMethod>()?;
        segments.push(RequestSegment::Hostname(hostname));
        segments.push(RequestSegment::Verb(method));

        for part in uri.path().split('/') {
            if part.is_empty() {
                continue; // Skip empty segments
            }
            segments.push(RequestSegment::Path(PathSegment::Static(part.to_string())));
        }

        Ok(segments)
    }
}
