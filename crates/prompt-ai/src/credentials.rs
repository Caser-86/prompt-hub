use secrecy::SecretString;
use thiserror::Error;

pub trait CredentialStore {
    fn save(&self, provider_id: &str, secret: SecretString) -> Result<(), CredentialError>;
    fn load(&self, provider_id: &str) -> Result<Option<SecretString>, CredentialError>;
    fn remove(&self, provider_id: &str) -> Result<(), CredentialError>;
}

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("credential storage is unavailable")]
    Unavailable,
    #[error("credential provider identifier is required")]
    ProviderIdRequired,
}
