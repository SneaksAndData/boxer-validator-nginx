use crate::models::request_context::RequestContext;
use anyhow::anyhow;
use std::cmp::Ordering;
use strum_macros::EnumString;
use url::Url;
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
        segments.push(RequestSegment::Verb(method));
        segments.push(RequestSegment::Hostname(hostname));

        for part in uri.path().split('/') {
            if part.is_empty() {
                continue; // Skip empty segments
            }
            segments.push(RequestSegment::Static(part.to_string()));
        }

        Ok(segments)
    }
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, EnumString)]
pub enum HTTPMethod {
    #[strum(ascii_case_insensitive)]
    Get,
    #[strum(ascii_case_insensitive)]
    Post,
    #[strum(ascii_case_insensitive)]
    Put,
    #[strum(ascii_case_insensitive)]
    Delete,
    #[strum(ascii_case_insensitive)]
    Patch,
    #[strum(ascii_case_insensitive)]
    Head,
    #[strum(ascii_case_insensitive)]
    Options,
}

/// Represents a segment of a HTTP request.
/// Each segment can be a verb (HTTP method), hostname, static string, or a parameter.
/// e.g., in the request `GET https://example.com/api/resource/{id}/property`, the segments would be:
///   - Verb: `GET`
///   - Hostname: `example.com`
///   - Static: `api`
///   - Static: `resource`
///   - Parameter: `{id}`
/// Does not include the query string or fragment.
#[derive(Debug, Clone)]
pub enum RequestSegment {
    Verb(HTTPMethod),
    Hostname(String),
    Static(String),
    Parameter,
}

/// Implements the `Ord` trait for `PathSegment`.
/// VERB < HOSTNAME < (STATIC == PARAMETER)
/// NOTE: This ordering is not consistent with the `PartialOrd` trait, which is intentional.
/// This is because `PathSegment` is used in a Trie, and the Trie requires a total order for its keys.
/// Additionally, this implementation ensures that the `Parameter` segment is always considered less than any other segment,
/// and equal to `Static` segments.
impl Ord for RequestSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            RequestSegment::Verb(v1) => match other {
                RequestSegment::Verb(v2) => v1.cmp(v2),
                RequestSegment::Hostname(_) => Ordering::Greater,
                RequestSegment::Static(_) => Ordering::Greater,
                RequestSegment::Parameter => Ordering::Greater,
            },
            RequestSegment::Hostname(p1) => match other {
                RequestSegment::Verb(_) => Ordering::Less,
                RequestSegment::Hostname(p2) => p1.cmp(p2),
                RequestSegment::Static(_) => Ordering::Greater,
                RequestSegment::Parameter => Ordering::Greater,
            },
            RequestSegment::Static(s1) => match other {
                RequestSegment::Verb(_) => Ordering::Less,
                RequestSegment::Hostname(_) => Ordering::Less,
                RequestSegment::Static(s2) => s1.cmp(s2),
                RequestSegment::Parameter => Ordering::Equal,
            },
            RequestSegment::Parameter => match other {
                RequestSegment::Verb(_) => Ordering::Less,
                RequestSegment::Hostname(_) => Ordering::Less,
                RequestSegment::Static(_) => Ordering::Equal,
                RequestSegment::Parameter => Ordering::Equal,
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
