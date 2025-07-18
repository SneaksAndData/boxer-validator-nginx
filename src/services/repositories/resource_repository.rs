mod models;

use crate::services::repositories::common::TrieRepositoryData;
use crate::services::repositories::models::PathSegment;
use boxer_core::services::base::upsert_repository::{ReadOnlyRepository, UpsertRepository};
use cedar_policy::EntityUid;

pub trait ResourceUpsertRepository:
    ReadOnlyRepository<Vec<PathSegment>, EntityUid, ReadError = anyhow::Error>
    + UpsertRepository<Vec<PathSegment>, EntityUid, Error = anyhow::Error>
{
}

type TrieData = super::common::TrieData<PathSegment>;

pub type ResourceRepository = TrieRepositoryData<PathSegment>;

impl ResourceUpsertRepository for TrieRepositoryData<PathSegment> {}

pub type ResourceReadOnlyRepository = dyn ReadOnlyRepository<Vec<PathSegment>, EntityUid, ReadError = anyhow::Error>;
