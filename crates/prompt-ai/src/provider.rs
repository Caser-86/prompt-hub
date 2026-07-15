use std::time::Duration;

use reqwest::blocking::Client;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde_json::{Value, json};
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

/// OpenAI-compatible `/v1/chat/completions` adapter.
///
/// This type never serializes or logs credentials. Callers should create it with a trusted
/// HTTPS endpoint and store credentials exclusively in `CredentialStore`.
pub struct OpenAiCompatibleProvider {
    endpoint: String,
    client: Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(endpoint: impl Into<String>, timeout: Duration) -> Result<Self, ProviderError> {
        let endpoint = endpoint.into().trim_end_matches('/').to_owned();
        if !endpoint.starts_with("https://") {
            return Err(ProviderError::InvalidConfiguration);
        }
        let client = Client::builder()
            .timeout(timeout)
            .connect_timeout(timeout.min(Duration::from_secs(10)))
            .build()
            .map_err(|_| ProviderError::RequestFailed)?;
        Ok(Self { endpoint, client })
    }

    fn parse_output(value: Value) -> Result<GenerationOutput, ProviderError> {
        let content = value
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
            .ok_or(ProviderError::InvalidResponse)?;
        let output: Value =
            serde_json::from_str(content).map_err(|_| ProviderError::InvalidResponse)?;
        let title = output
            .get("title")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .ok_or(ProviderError::InvalidResponse)?;
        let body = output
            .get("body")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|body| !body.is_empty())
            .ok_or(ProviderError::InvalidResponse)?;
        Ok(GenerationOutput {
            title: title.to_owned(),
            body: body.to_owned(),
        })
    }

    pub fn test_connection(&self, credential: SecretString) -> Result<(), ProviderError> {
        let response = self
            .client
            .get(format!("{}/v1/models", self.endpoint))
            .bearer_auth(credential.expose_secret())
            .send()
            .map_err(|error| {
                if error.is_timeout() {
                    ProviderError::Timeout
                } else {
                    ProviderError::RequestFailed
                }
            })?;
        Self::ensure_success(response.status().as_u16())
    }

    fn ensure_success(status: u16) -> Result<(), ProviderError> {
        if status == 429 {
            Err(ProviderError::RateLimited)
        } else if (400..600).contains(&status) {
            Err(ProviderError::RequestFailed)
        } else {
            Ok(())
        }
    }
}

impl AiProvider for OpenAiCompatibleProvider {
    fn generate(
        &self,
        request: GenerationRequest,
        credential: SecretString,
    ) -> Result<GenerationOutput, ProviderError> {
        let response = self.client
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .bearer_auth(credential.expose_secret())
            .json(&json!({
                "model": request.model,
                "messages": [
                    { "role": "system", "content": format!("{}\nReturn only a JSON object with non-empty title and body string fields.", request.instruction) },
                    { "role": "user", "content": request.input_summary }
                ],
                "response_format": { "type": "json_object" }
            }))
            .send()
            .map_err(|error| if error.is_timeout() { ProviderError::Timeout } else { ProviderError::RequestFailed })?;
        Self::ensure_success(response.status().as_u16())?;
        response
            .json::<Value>()
            .map_err(|_| ProviderError::InvalidResponse)
            .and_then(Self::parse_output)
    }
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
    #[error("AI provider configuration is invalid")]
    InvalidConfiguration,
    #[error("provider timed out")]
    Timeout,
    #[error("provider rate limited the request")]
    RateLimited,
    #[error("provider returned an invalid response")]
    InvalidResponse,
    #[error("provider request failed")]
    RequestFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_structured_non_empty_openai_output() {
        let output = OpenAiCompatibleProvider::parse_output(json!({
            "choices": [{ "message": { "content": r#"{"title":"审查","body":"请审查变更"}"# } }]
        }))
        .unwrap();
        assert_eq!(output.title, "审查");
        assert_eq!(output.body, "请审查变更");
        assert!(OpenAiCompatibleProvider::parse_output(json!({"choices": []})).is_err());
        assert!(matches!(
            OpenAiCompatibleProvider::new("http://insecure.example", Duration::from_secs(1)),
            Err(ProviderError::InvalidConfiguration)
        ));
        assert!(OpenAiCompatibleProvider::ensure_success(200).is_ok());
        assert!(matches!(
            OpenAiCompatibleProvider::ensure_success(429),
            Err(ProviderError::RateLimited)
        ));
        assert!(matches!(
            OpenAiCompatibleProvider::ensure_success(401),
            Err(ProviderError::RequestFailed)
        ));
    }
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
