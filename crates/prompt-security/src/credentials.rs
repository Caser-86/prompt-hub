use std::fmt;

use keyring::{Entry, Error as KeyringError};
use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialKey {
    provider: String,
    profile: String,
}

impl CredentialKey {
    pub fn new(
        provider: impl Into<String>,
        profile: impl Into<String>,
    ) -> Result<Self, CredentialError> {
        let provider = component(provider)?;
        let profile = component(profile)?;
        Ok(Self { provider, profile })
    }

    #[must_use]
    pub fn account_name(&self) -> String {
        format!("{}/{}", self.provider, self.profile)
    }
}

pub trait CredentialStore {
    fn set(&self, key: &CredentialKey, secret: &SecretString) -> Result<(), CredentialError>;
    fn get(&self, key: &CredentialKey) -> Result<Option<SecretString>, CredentialError>;
    fn delete(&self, key: &CredentialKey) -> Result<(), CredentialError>;
}

pub struct SystemCredentialStore {
    service_name: String,
}

impl SystemCredentialStore {
    pub fn new(service_name: impl Into<String>) -> Result<Self, CredentialError> {
        let service_name = component(service_name)?;
        Ok(Self { service_name })
    }

    #[must_use]
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    fn entry(&self, key: &CredentialKey) -> Result<Entry, CredentialError> {
        Entry::new(&self.service_name, &key.account_name()).map_err(CredentialError::from)
    }
}

impl fmt::Debug for SystemCredentialStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SystemCredentialStore")
            .field("service_name", &self.service_name)
            .finish_non_exhaustive()
    }
}

impl CredentialStore for SystemCredentialStore {
    fn set(&self, key: &CredentialKey, secret: &SecretString) -> Result<(), CredentialError> {
        self.entry(key)?.set_password(secret.expose_secret())?;
        Ok(())
    }

    fn get(&self, key: &CredentialKey) -> Result<Option<SecretString>, CredentialError> {
        match self.entry(key)?.get_password() {
            Ok(secret) => Ok(Some(SecretString::from(secret))),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(error) => Err(CredentialError::Keyring(error)),
        }
    }

    fn delete(&self, key: &CredentialKey) -> Result<(), CredentialError> {
        match self.entry(key)?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(error) => Err(CredentialError::Keyring(error)),
        }
    }
}

fn component(value: impl Into<String>) -> Result<String, CredentialError> {
    let value = value.into().trim().to_owned();
    if value.is_empty() || value.contains(['/', '\\', '\0']) {
        Err(CredentialError::InvalidKeyComponent)
    } else {
        Ok(value)
    }
}

#[derive(Debug, Error)]
pub enum CredentialError {
    #[error("credential key components must be non-empty and path-free")]
    InvalidKeyComponent,
    #[error("operating-system credential store failed: {0}")]
    Keyring(#[from] KeyringError),
}
