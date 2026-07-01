/// Represents an external JWT Token used by Boxer to validate the internal token
pub struct BoxerToken {
    pub token: String,
}

/// Allows `InternalToken` to be converted to a String
impl Into<String> for BoxerToken {
    fn into(self) -> String {
        self.token.clone()
    }
}

/// Allows a String to be converted to an `InternalToken`
impl From<String> for BoxerToken {
    fn from(token: String) -> Self {
        BoxerToken { token }
    }
}
