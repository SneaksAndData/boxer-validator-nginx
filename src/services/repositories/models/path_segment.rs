use crate::models::request_context::RequestContext;
use crate::services::prefix_tree::naive_tree::ParametrizedMatcher;
use strum_macros::Display;
use url::Url;

#[derive(Debug, Clone, Display, Hash, Ord, PartialOrd, Eq, PartialEq)]
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
