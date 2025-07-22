pub fn test_policy() -> String {
    r#"permit (
    principal == PhotoApp::User::"alice",
    action == PhotoApp::Action::"viewPhoto",
    resource == PhotoApp::Photo::"vacationPhoto.jpg"
);

permit (
    principal == PhotoApp::User::"stacey",
    action == PhotoApp::Action::"viewPhoto",
    resource
)
when {
    resource in PhotoApp::Account::"stacey"
};"#
    .to_string()
}

pub fn test_updated_policy() -> String {
    r#"permit (
    principal == PhotoApp::User::"stacey",
    action == PhotoApp::Action::"deletePhoto",
    resource == PhotoApp::Photo::"vacationPhoto.jpg"
);

permit (
    principal == PhotoApp::User::"stacey",
    action == PhotoApp::Action::"viewPhoto",
    resource
)
when {
    resource in PhotoApp::Account::"stacey"
};"#
    .to_string()
}
