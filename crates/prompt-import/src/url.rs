use std::io::Read;
use std::net::{SocketAddr, ToSocketAddrs};

use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use thiserror::Error;
use time::OffsetDateTime;
use url::Url;

use crate::{ReadableText, UrlPolicy, UrlPolicyError, extract_readable_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlImportResult {
    pub canonical_url: String,
    pub retrieved_at: OffsetDateTime,
    pub title: Option<String>,
    pub text: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Error)]
pub enum UrlImportError {
    #[error(transparent)]
    Policy(#[from] UrlPolicyError),
    #[error("URL is invalid")]
    InvalidUrl,
    #[error("URL host could not be resolved")]
    Resolution,
    #[error("URL request timed out")]
    Timeout,
    #[error("URL request failed")]
    Request,
    #[error("URL redirect did not provide a location")]
    RedirectLocationMissing,
    #[error("URL returned too many redirects")]
    TooManyRedirects,
    #[error("URL content type is not supported")]
    UnsupportedContentType,
    #[error("URL response exceeds the configured size limit")]
    ResponseTooLarge,
    #[error("URL response body could not be read")]
    Read,
}

pub fn fetch_url(input: &str, policy: UrlPolicy) -> Result<UrlImportResult, UrlImportError> {
    let mut current = Url::parse(input).map_err(|_| UrlImportError::InvalidUrl)?;
    policy.validate_url(&current)?;
    for redirect_count in 0..=policy.max_redirects() {
        let addresses = resolve(&current)?;
        policy.validate_resolved_target(
            &current,
            &addresses.iter().map(SocketAddr::ip).collect::<Vec<_>>(),
        )?;
        let host = current.host_str().ok_or(UrlImportError::InvalidUrl)?;
        let client = Client::builder()
            .redirect(Policy::none())
            .timeout(policy.request_timeout())
            .connect_timeout(policy.request_timeout())
            .resolve_to_addrs(host, &addresses)
            .build()
            .map_err(|_| UrlImportError::Request)?;
        let response = client.get(current.clone()).send().map_err(|error| {
            if error.is_timeout() {
                UrlImportError::Timeout
            } else {
                UrlImportError::Request
            }
        })?;
        if response.status().is_redirection() {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or(UrlImportError::RedirectLocationMissing)?;
            if redirect_count == policy.max_redirects() {
                return Err(UrlImportError::TooManyRedirects);
            }
            let target = current
                .join(location)
                .map_err(|_| UrlImportError::InvalidUrl)?;
            let target_addresses = resolve(&target)?;
            policy.validate_redirect(
                &current,
                &target,
                &target_addresses
                    .iter()
                    .map(SocketAddr::ip)
                    .collect::<Vec<_>>(),
                redirect_count + 1,
            )?;
            current = target;
            continue;
        }
        if response.status() != StatusCode::OK {
            return Err(UrlImportError::Request);
        }
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        if !is_textual(content_type) {
            return Err(UrlImportError::UnsupportedContentType);
        }
        if response
            .content_length()
            .is_some_and(|length| length > policy.max_response_bytes())
        {
            return Err(UrlImportError::ResponseTooLarge);
        }
        let mut body = Vec::new();
        response
            .take(policy.max_response_bytes() + 1)
            .read_to_end(&mut body)
            .map_err(|_| UrlImportError::Read)?;
        if body.len() as u64 > policy.max_response_bytes() {
            return Err(UrlImportError::ResponseTooLarge);
        }
        let body = String::from_utf8_lossy(&body);
        let ReadableText {
            title,
            text,
            warnings,
        } = extract_readable_text(&body);
        return Ok(UrlImportResult {
            canonical_url: current.into(),
            retrieved_at: OffsetDateTime::now_utc(),
            title,
            text,
            warnings,
        });
    }
    Err(UrlImportError::TooManyRedirects)
}

fn resolve(url: &Url) -> Result<Vec<SocketAddr>, UrlImportError> {
    let host = url.host_str().ok_or(UrlImportError::InvalidUrl)?;
    let port = url
        .port_or_known_default()
        .ok_or(UrlImportError::InvalidUrl)?;
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|_| UrlImportError::Resolution)?
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err(UrlImportError::Resolution);
    }
    Ok(addresses)
}

fn is_textual(content_type: &str) -> bool {
    let content_type = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    content_type.starts_with("text/")
        || matches!(
            content_type.as_str(),
            "application/json" | "application/xhtml+xml" | "application/xml"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_file_and_loopback_urls_before_a_network_request() {
        assert!(matches!(
            fetch_url("file:///C:/secret.txt", UrlPolicy::default()),
            Err(UrlImportError::Policy(UrlPolicyError::UnsupportedScheme))
        ));
        assert!(matches!(
            fetch_url("http://127.0.0.1/", UrlPolicy::default()),
            Err(UrlImportError::Policy(UrlPolicyError::NonPublicAddress(_)))
        ));
    }
}
