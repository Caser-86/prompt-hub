use secrecy::SecretString;
use thiserror::Error;
use time::OffsetDateTime;

use crate::{AiDraft, CredentialStore, DraftError};

pub trait AiProvider {
    fn generate(
        &self,
        request: GenerationRequest,
        credential: SecretString,
    ) -> Result<GenerationOutput, ProviderError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationRequest {
    pub instruction: String,
    pub input_summary: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationOutput {
    pub title: String,
    pub body: String,
}

pub struct DraftGenerator<P, C> {
    provider: P,
    credentials: C,
}

impl<P: AiProvider, C: CredentialStore> DraftGenerator<P, C> {
    #[must_use]
    pub const fn new(provider: P, credentials: C) -> Self {
        Self {
            provider,
            credentials,
        }
    }

    pub fn generate(
        &self,
        provider_id: &str,
        request: GenerationRequest,
        generated_at: OffsetDateTime,
    ) -> Result<AiDraft, GenerationError> {
        let credential = self
            .credentials
            .load(provider_id)
            .map_err(|_| GenerationError::CredentialUnavailable)?
            .ok_or(GenerationError::CredentialMissing)?;
        let output = self
            .provider
            .generate(request.clone(), credential)
            .map_err(GenerationError::Provider)?;
        AiDraft::new(
            output.title,
            output.body,
            request.model,
            request.input_summary,
            generated_at,
        )
        .map_err(GenerationError::Draft)
    }
}

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("provider timed out")]
    Timeout,
    #[error("provider rate limited the request")]
    RateLimited,
    #[error("provider returned an invalid response")]
    InvalidResponse,
    #[error("provider request failed")]
    RequestFailed,
}
#[derive(Debug, Error)]
pub enum GenerationError {
    #[error("AI credential storage is unavailable")]
    CredentialUnavailable,
    #[error("AI credential is not configured")]
    CredentialMissing,
    #[error(transparent)]
    Provider(ProviderError),
    #[error(transparent)]
    Draft(DraftError),
}
