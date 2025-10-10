use crate::models::request_context::RequestContext;
use crate::services::prefix_tree::hash_tree::ParametrizedMatcher;
use std::cmp::Ordering;
use strum_macros::Display;
use url::Url;

#[derive(Debug, Clone, Display, Hash)]
pub enum PathSegment {
    Static(String),
    Parameter,
}

impl TryFrom<RequestContext> for Vec<PathSegment> {
    type Error = anyhow::Error;

    fn try_from(context: RequestContext) -> Result<Self, Self::Error> {
        let uri = Url::parse(context.original_url.as_str())?;
        let mut segments = vec![];
        for part in uri.path().split('/') {
            if part.is_empty() {
                continue; // Skip empty segments
            }
            segments.push(PathSegment::Static(part.to_string()));
        }
        Ok(segments)
    }
}

impl ParametrizedMatcher for PathSegment {
    fn is_parameter(&self) -> bool {
        matches!(self, PathSegment::Parameter)
    }
}

impl Ord for PathSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            PathSegment::Static(s1) => match other {
                PathSegment::Static(s2) => s1.cmp(s2),
                PathSegment::Parameter => Ordering::Equal,
            },
            PathSegment::Parameter => match other {
                PathSegment::Static(_) => Ordering::Equal,
                PathSegment::Parameter => Ordering::Equal,
            },
        }
    }
}

impl PartialOrd for PathSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for PathSegment {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for PathSegment {}
