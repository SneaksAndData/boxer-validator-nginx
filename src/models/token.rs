/// Represents an external JWT Token used to authorize the `ExternalIdentity` and issue an `InternalToken`
pub struct BoxerToken {
    pub token: String,
}

/// Allows `ExternalToken` to be converted to a String
impl Into<String> for BoxerToken {
    fn into(self) -> String {
        self.token.clone()
    }
}

/// Allows a String to be converted to an `ExternalToken`
impl From<String> for BoxerToken {
    fn from(token: String) -> Self {
        BoxerToken { token }
    }
}
