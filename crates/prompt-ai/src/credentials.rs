use prompt_security::{CredentialKey, CredentialStore as SystemStore, SystemCredentialStore};
use secrecy::SecretString;
use thiserror::Error;

pub trait CredentialStore {
    fn save(&self, provider_id: &str, secret: SecretString) -> Result<(), CredentialError>;
    fn load(&self, provider_id: &str) -> Result<Option<SecretString>, CredentialError>;
    fn remove(&self, provider_id: &str) -> Result<(), CredentialError>;
}

pub struct SystemCredentialAdapter {
    store: SystemCredentialStore,
    profile: String,
}

impl SystemCredentialAdapter {
    pub fn new(
        service_name: impl Into<String>,
        profile: impl Into<String>,
    ) -> Result<Self, CredentialError> {
        let profile = profile.into();
        CredentialKey::new("validation", &profile)
            .map_err(|_| CredentialError::ProviderIdRequired)?;
        Ok(Self {
            store: SystemCredentialStore::new(service_name)
                .map_err(|_| CredentialError::Unavailable)?,
            profile,
        })
    }

    fn key(&self, provider_id: &str) -> Result<CredentialKey, CredentialError> {
        CredentialKey::new(provider_id, &self.profile)
            .map_err(|_| CredentialError::ProviderIdRequired)
    }
}

impl CredentialStore for SystemCredentialAdapter {
    fn save(&self, provider_id: &str, secret: SecretString) -> Result<(), CredentialError> {
        self.store
            .set(&self.key(provider_id)?, &secret)
            .map_err(|_| CredentialError::Unavailable)
    }
    fn load(&self, provider_id: &str) -> Result<Option<SecretString>, CredentialError> {
        self.store
            .get(&self.key(provider_id)?)
            .map_err(|_| CredentialError::Unavailable)
    }
    fn remove(&self, provider_id: &str) -> Result<(), CredentialError> {
        self.store
            .delete(&self.key(provider_id)?)
            .map_err(|_| CredentialError::Unavailable)
    }
}

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("credential storage is unavailable")]
    Unavailable,
    #[error("credential provider identifier is required")]
    ProviderIdRequired,
}
