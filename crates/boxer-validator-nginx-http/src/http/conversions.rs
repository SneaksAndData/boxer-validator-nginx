#[cfg(test)]
mod tests;

use crate::models::token::BoxerToken;
use actix_web::http::header::HeaderValue;
use anyhow::bail;

impl TryFrom<&HeaderValue> for BoxerToken {
    type Error = anyhow::Error;

    // Compiler will complain that `return Err` statements in this code are unreachable,
    // but this is not true. See test cases below.
    #[allow(unreachable_code)]
    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        match value.to_str() {
            Ok(string_value) => {
                let tokens = string_value.split(" ").collect::<Vec<&str>>();

                if tokens.len() != 2 {
                    return Err(bail!("Invalid token format"));
                }

                if tokens[0] != "Bearer" {
                    return Err(bail!("Invalid token format"));
                }

                Ok(BoxerToken::from(tokens[1].to_owned()))
            }
            Err(_) => bail!("Invalid token format"),
        }
    }
}
