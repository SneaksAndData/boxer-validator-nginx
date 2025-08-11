use crate::models::request_context::RequestContext;
use crate::services::repositories::models::http_method::HTTPMethod;
use crate::services::repositories::models::path_segment::PathSegment;
use anyhow::anyhow;
use std::cmp::Ordering;
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
#[derive(Debug, Clone, Display)]
pub enum RequestSegment {
    Verb(HTTPMethod),
    Hostname(String),
    Path(PathSegment),
}

/// Implements the `Ord` trait for `PathSegment`.
/// VERB < HOSTNAME < (STATIC == PARAMETER)
/// NOTE: This ordering is not consistent with the `PartialOrd` trait, which is intentional.
/// This is because `PathSegment` is used in a Trie, and the Trie requires a total order for its keys.
/// Additionally, this implementation ensures that the `Parameter` segment is always considered less than any other segment,
/// and equal to `Static` segments.
/// Also, see the `PartialOrd` implementation for `PathSegment` below.
impl Ord for RequestSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            RequestSegment::Verb(v1) => match other {
                RequestSegment::Verb(v2) => v1.cmp(v2),
                RequestSegment::Hostname(_) => Ordering::Greater,
                RequestSegment::Path(PathSegment::Static(_)) => Ordering::Greater,
                RequestSegment::Path(PathSegment::Parameter) => Ordering::Greater,
            },
            RequestSegment::Hostname(p1) => match other {
                RequestSegment::Verb(_) => Ordering::Less,
                RequestSegment::Hostname(p2) => p1.cmp(p2),
                RequestSegment::Path(PathSegment::Static(_)) => Ordering::Greater,
                RequestSegment::Path(PathSegment::Parameter) => Ordering::Greater,
            },
            RequestSegment::Path(PathSegment::Static(s1)) => match other {
                RequestSegment::Verb(_) => Ordering::Less,
                RequestSegment::Hostname(_) => Ordering::Less,
                RequestSegment::Path(PathSegment::Static(s2)) => s1.cmp(s2),
                RequestSegment::Path(PathSegment::Parameter) => Ordering::Equal,
            },
            RequestSegment::Path(PathSegment::Parameter) => match other {
                RequestSegment::Verb(_) => Ordering::Less,
                RequestSegment::Hostname(_) => Ordering::Less,
                RequestSegment::Path(PathSegment::Static(_)) => Ordering::Equal,
                RequestSegment::Path(PathSegment::Parameter) => Ordering::Equal,
            },
        }
    }
}

impl PartialOrd for RequestSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for RequestSegment {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for RequestSegment {}

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
