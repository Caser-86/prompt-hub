use prompt_security::{
    CredentialKey, CredentialStore, RedactedBody, Redactor, SystemCredentialStore,
};
use secrecy::{ExposeSecret, SecretString};

#[test]
fn secret_values_and_bodies_are_redacted_in_debug_and_display_output() {
    let secret = SecretString::from("sk-production-secret".to_owned());
    let body = RedactedBody::new("private prompt body");

    assert!(!format!("{secret:?}").contains(secret.expose_secret()));
    assert_eq!(format!("{body}"), "[REDACTED PROMPT BODY]");
    assert!(!format!("{body:?}").contains("private prompt body"));
}

#[test]
fn redacts_url_credentials_query_values_and_authorization_headers() {
    let redactor = Redactor::default();
    let url = redactor
        .url("https://user:password@example.com/path?api_key=secret&trace=123")
        .unwrap();
    let header = redactor.header("Authorization", "Bearer secret-token");

    assert_eq!(
        url,
        "https://example.com/path?api_key=%5BREDACTED%5D&trace=%5BREDACTED%5D"
    );
    assert_eq!(header, "Authorization: [REDACTED]");
    assert!(!url.contains("password"));
    assert!(!url.contains("secret"));
    assert!(!header.contains("secret-token"));
}

#[test]
fn system_store_uses_a_stable_service_and_provider_scoped_key() {
    let store = SystemCredentialStore::new("app.prompthub.desktop").unwrap();
    let key = CredentialKey::new("openai-compatible", "default").unwrap();

    assert_eq!(store.service_name(), "app.prompthub.desktop");
    assert_eq!(key.account_name(), "openai-compatible/default");
    assert!(!format!("{store:?}").contains("password"));

    fn assert_store_contract<T: CredentialStore>(_store: &T) {}
    assert_store_contract(&store);
}
