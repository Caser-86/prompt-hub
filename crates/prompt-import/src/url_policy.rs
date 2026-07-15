use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UrlPolicy {
    request_timeout: Duration,
    max_response_bytes: u64,
    max_redirects: u8,
}

impl Default for UrlPolicy {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(15),
            max_response_bytes: 2 * 1024 * 1024,
            max_redirects: 5,
        }
    }
}

impl UrlPolicy {
    pub fn validate_url(&self, url: &Url) -> Result<(), UrlPolicyError> {
        if !matches!(url.scheme(), "http" | "https") {
            return Err(UrlPolicyError::UnsupportedScheme);
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err(UrlPolicyError::EmbeddedCredentials);
        }
        if url.host().is_none() {
            return Err(UrlPolicyError::MissingHost);
        }
        Ok(())
    }

    #[must_use]
    pub const fn request_timeout(&self) -> Duration {
        self.request_timeout
    }

    #[must_use]
    pub const fn max_response_bytes(&self) -> u64 {
        self.max_response_bytes
    }

    #[must_use]
    pub const fn max_redirects(&self) -> u8 {
        self.max_redirects
    }

    pub fn validate_resolved_target(
        &self,
        url: &Url,
        addresses: &[IpAddr],
    ) -> Result<(), UrlPolicyError> {
        self.validate_url(url)?;
        if addresses.is_empty() {
            return Err(UrlPolicyError::ResolutionRequired);
        }
        for address in addresses {
            if !is_public_address(*address) {
                return Err(UrlPolicyError::NonPublicAddress(*address));
            }
        }
        Ok(())
    }

    pub fn validate_redirect(
        &self,
        _from: &Url,
        target: &Url,
        freshly_resolved_addresses: &[IpAddr],
        redirect_count: u8,
    ) -> Result<(), UrlPolicyError> {
        if redirect_count > self.max_redirects {
            return Err(UrlPolicyError::TooManyRedirects);
        }
        self.validate_resolved_target(target, freshly_resolved_addresses)
    }
}

fn is_public_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => is_public_ipv4(address),
        IpAddr::V6(address) => is_public_ipv6(address),
    }
}

fn is_public_ipv4(address: Ipv4Addr) -> bool {
    let [a, b, c, d] = address.octets();
    !(a == 0
        || a == 10
        || a == 127
        || (a == 100 && (64..=127).contains(&b))
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && b == 0 && c == 0)
        || (a == 192 && b == 0 && c == 2)
        || (a == 192 && b == 88 && c == 99)
        || (a == 192 && b == 168)
        || (a == 198 && (b == 18 || b == 19))
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113)
        || a >= 224
        || (a == 255 && b == 255 && c == 255 && d == 255))
}

fn is_public_ipv6(address: Ipv6Addr) -> bool {
    if address.is_unspecified() || address.is_loopback() || address.is_multicast() {
        return false;
    }
    let segments = address.segments();
    if (segments[0] & 0xfe00) == 0xfc00
        || (segments[0] & 0xffc0) == 0xfe80
        || (segments[0] & 0xffc0) == 0xfec0
        || (segments[0] == 0x2001 && segments[1] == 0x0db8)
    {
        return false;
    }
    address.to_ipv4_mapped().is_none()
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UrlPolicyError {
    #[error("only HTTP and HTTPS URLs are supported")]
    UnsupportedScheme,
    #[error("URL credentials are not allowed")]
    EmbeddedCredentials,
    #[error("URL host is required")]
    MissingHost,
    #[error("a fresh DNS resolution is required")]
    ResolutionRequired,
    #[error("resolved address is not publicly routable: {0}")]
    NonPublicAddress(IpAddr),
    #[error("redirect limit exceeded")]
    TooManyRedirects,
}
