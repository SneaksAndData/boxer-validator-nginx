pub struct BoxerToken {
    pub token: String,
}

impl Into<String> for BoxerToken {
    fn into(self) -> String {
        self.token.clone()
    }
}

impl From<String> for BoxerToken {
    fn from(token: String) -> Self {
        BoxerToken { token }
    }
}
