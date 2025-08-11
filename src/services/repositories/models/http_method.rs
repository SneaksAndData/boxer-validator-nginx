use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Clone, EnumString, Serialize, Deserialize, JsonSchema, Display)]
#[strum(serialize_all = "UPPERCASE")]
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
