use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

use prompt_import::{UrlPolicy, UrlPolicyError};
use url::Url;

#[test]
fn centralizes_fixed_network_resource_limits() {
    let policy = UrlPolicy::default();

    assert_eq!(policy.request_timeout(), Duration::from_secs(15));
    assert_eq!(policy.max_response_bytes(), 2 * 1024 * 1024);
    assert_eq!(policy.max_redirects(), 5);
}

#[test]
fn accepts_https_only_when_every_resolved_address_is_public() {
    let policy = UrlPolicy::default();
    let url = Url::parse("https://example.com/prompts").unwrap();

    policy
        .validate_resolved_target(&url, &[IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))])
        .expect("public HTTPS destination should be allowed");
}

#[test]
fn rejects_non_http_schemes_and_embedded_credentials() {
    let policy = UrlPolicy::default();

    assert_eq!(
        policy
            .validate_resolved_target(&Url::parse("file:///C:/secret.txt").unwrap(), &[])
            .unwrap_err(),
        UrlPolicyError::UnsupportedScheme
    );
    assert_eq!(
        policy
            .validate_resolved_target(
                &Url::parse("https://user:password@example.com/").unwrap(),
                &[IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))],
            )
            .unwrap_err(),
        UrlPolicyError::EmbeddedCredentials
    );
}

#[test]
fn rejects_loopback_private_link_local_reserved_and_mixed_dns_answers() {
    let policy = UrlPolicy::default();
    let url = Url::parse("https://example.com/").unwrap();
    let denied = [
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)),
        IpAddr::V6(Ipv6Addr::LOCALHOST),
        "fe80::1".parse().unwrap(),
        "fc00::1".parse().unwrap(),
    ];

    for address in denied {
        assert_eq!(
            policy
                .validate_resolved_target(&url, &[address])
                .unwrap_err(),
            UrlPolicyError::NonPublicAddress(address)
        );
    }

    let mixed = [
        IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    ];
    assert!(matches!(
        policy.validate_resolved_target(&url, &mixed),
        Err(UrlPolicyError::NonPublicAddress(_))
    ));
}

#[test]
fn requires_fresh_resolution_for_every_redirect() {
    let policy = UrlPolicy::default();
    let initial = Url::parse("https://example.com/start").unwrap();
    let redirected = Url::parse("https://redirect.example/final").unwrap();

    policy
        .validate_redirect(
            &initial,
            &redirected,
            &[IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))],
            1,
        )
        .unwrap();
    assert!(matches!(
        policy.validate_redirect(
            &initial,
            &redirected,
            &[IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))],
            1,
        ),
        Err(UrlPolicyError::NonPublicAddress(_))
    ));
    assert_eq!(
        policy
            .validate_redirect(
                &initial,
                &redirected,
                &[IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))],
                6,
            )
            .unwrap_err(),
        UrlPolicyError::TooManyRedirects
    );
}
