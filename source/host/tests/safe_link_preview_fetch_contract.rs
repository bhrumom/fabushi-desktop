use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use mahayana_host_runtime::extensions::attachments::safe_link_preview_fetch::{
    MAX_URL_LENGTH, SandLinkPreviewError, get_safe_link_preview_connection_target_with,
    is_blocked_hostname, is_blocked_link_preview_ip, normalize_hostname,
    parse_safe_link_preview_url, read_limited_body, resolve_safe_link_preview_redirect,
};

#[test]
fn link_preview_url_policy_matches_frozen_https_and_hostname_rules() {
    assert_eq!(normalize_hostname("Example.COM."), "example.com");
    assert!(is_blocked_hostname("localhost"));
    assert!(is_blocked_hostname("service"));
    assert!(is_blocked_hostname("box.internal"));
    assert!(is_blocked_hostname("foo.svc"));
    assert!(is_blocked_hostname("hidden.onion"));
    assert!(!is_blocked_hostname("example.com"));

    assert!(matches!(
        parse_safe_link_preview_url("http://example.com"),
        Err(SandLinkPreviewError::RequiresHttps)
    ));
    assert!(matches!(
        parse_safe_link_preview_url("https://user:pass@example.com"),
        Err(SandLinkPreviewError::Credentials)
    ));
    assert!(matches!(
        parse_safe_link_preview_url("https://example.com:8443"),
        Err(SandLinkPreviewError::CustomPort)
    ));
    assert!(matches!(
        parse_safe_link_preview_url("https://localhost"),
        Err(SandLinkPreviewError::NonPublicHostname)
    ));
    assert!(matches!(
        parse_safe_link_preview_url("https://127.0.0.1"),
        Err(SandLinkPreviewError::NonPublicIp)
    ));
    assert!(parse_safe_link_preview_url("https://example.com/path?q=1").is_ok());

    let too_long = format!("https://example.com/{}", "a".repeat(MAX_URL_LENGTH));
    assert!(matches!(
        parse_safe_link_preview_url(&too_long),
        Err(SandLinkPreviewError::UrlTooLong)
    ));
}

#[test]
fn link_preview_ip_blocklist_matches_frozen_private_reserved_and_public_ranges() {
    for address in [
        IpAddr::V4(Ipv4Addr::new(10, 1, 2, 3)),
        IpAddr::V4(Ipv4Addr::new(100, 64, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(172, 31, 255, 255)),
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        IpAddr::V4(Ipv4Addr::new(198, 51, 100, 7)),
        IpAddr::V4(Ipv4Addr::new(224, 0, 0, 1)),
        IpAddr::V6(Ipv6Addr::LOCALHOST),
        "fc00::1".parse().unwrap(),
        "fe80::1".parse().unwrap(),
        "2001:db8::1".parse().unwrap(),
        "ff02::1".parse().unwrap(),
    ] {
        assert!(is_blocked_link_preview_ip(address), "{address} should be blocked");
    }

    assert!(!is_blocked_link_preview_ip(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))));
    assert!(!is_blocked_link_preview_ip("2606:4700:4700::1111".parse().unwrap()));
}

#[test]
fn dns_resolution_rejects_any_non_public_answer_and_pins_public_ipv4_first() {
    let url = parse_safe_link_preview_url("https://example.com/page").unwrap();
    let blocked = get_safe_link_preview_connection_target_with(&url, |_| {
        Ok(vec![
            IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 7)),
        ])
    });
    assert!(matches!(blocked, Err(SandLinkPreviewError::NonPublicIp)));

    let target = get_safe_link_preview_connection_target_with(&url, |_| {
        Ok(vec![
            "2606:4700:4700::1111".parse().unwrap(),
            IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
        ])
    })
    .unwrap();
    assert_eq!(target.connect_host, "1.1.1.1");
    assert_eq!(target.host_header, "example.com");
    assert_eq!(target.servername.as_deref(), Some("example.com"));
    assert_eq!(target.socket_addr.ip(), IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));

    let empty = get_safe_link_preview_connection_target_with(&url, |_| Ok(Vec::new()));
    assert!(matches!(
        empty,
        Err(SandLinkPreviewError::HostnameDidNotResolve)
    ));
}

#[test]
fn redirect_resolution_revalidates_destination_and_never_relaxes_ssrf_policy() {
    let current = parse_safe_link_preview_url("https://example.com/a/b").unwrap();
    assert_eq!(
        resolve_safe_link_preview_redirect(&current, "../next")
            .unwrap()
            .as_str(),
        "https://example.com/next"
    );
    assert!(matches!(
        resolve_safe_link_preview_redirect(&current, "http://example.com"),
        Err(SandLinkPreviewError::RequiresHttps)
    ));
    assert!(matches!(
        resolve_safe_link_preview_redirect(&current, "https://127.0.0.1/private"),
        Err(SandLinkPreviewError::NonPublicIp)
    ));
}

#[test]
fn response_body_limit_truncates_only_when_explicitly_allowed() {
    let bytes = vec![b'x'; 17];
    let mut strict = Cursor::new(bytes.clone());
    assert!(matches!(
        read_limited_body(&mut strict, 16, false),
        Err(SandLinkPreviewError::BodyTooLarge)
    ));

    let mut truncated = Cursor::new(bytes);
    assert_eq!(
        read_limited_body(&mut truncated, 16, true).unwrap().len(),
        16
    );
}

#[test]
fn mapped_ipv6_addresses_preserve_ipv4_admission_decisions() {
    for (address, blocked) in [
        ("0.1.2.3", true),
        ("10.1.2.3", true),
        ("100.64.0.1", true),
        ("127.0.0.1", true),
        ("169.254.169.254", true),
        ("172.31.255.255", true),
        ("192.0.0.7", true),
        ("192.0.2.7", true),
        ("192.88.99.7", true),
        ("192.168.1.1", true),
        ("198.18.0.1", true),
        ("198.51.100.7", true),
        ("203.0.113.7", true),
        ("224.0.0.1", true),
        ("255.255.255.255", true),
        ("1.1.1.1", false),
        ("8.8.8.8", false),
        ("100.63.255.255", false),
        ("100.128.0.0", false),
        ("172.15.255.255", false),
        ("172.32.0.0", false),
        ("198.17.255.255", false),
        ("198.20.0.0", false),
    ] {
        let ipv4: Ipv4Addr = address.parse().expect("IPv4 fixture");
        let mapped = IpAddr::V6(ipv4.to_ipv6_mapped());
        assert_eq!(
            is_blocked_link_preview_ip(IpAddr::V4(ipv4)),
            blocked,
            "native IPv4 control for {address}"
        );
        assert_eq!(
            is_blocked_link_preview_ip(mapped),
            blocked,
            "mapped address {mapped} must retain the IPv4 decision"
        );
    }

    assert!(!is_blocked_link_preview_ip(
        "2001:4860:4860::8888".parse().unwrap()
    ));
}

const MAPPED_PRIVATE_LINK_URLS: &[&str] = &[
    "https://[::ffff:127.0.0.1]/private",
    "https://[::ffff:7f00:1]/private",
    "https://[::ffff:10.1.2.3]/private",
    "https://[::ffff:a01:203]/private",
    "https://[::ffff:169.254.169.254]/private",
    "https://[::ffff:a9fe:a9fe]/private",
];

#[test]
fn mapped_ip_url_literals_reject_private_dotted_and_hex_forms() {
    for raw_url in MAPPED_PRIVATE_LINK_URLS {
        url::Url::parse(raw_url).expect("negative fixture must be a valid URL");
        assert_eq!(
            parse_safe_link_preview_url(raw_url),
            Err(SandLinkPreviewError::NonPublicIp),
            "mapped private literal {raw_url} must fail IP admission"
        );
    }

    for raw_url in [
        "https://[::ffff:1.1.1.1]/public",
        "https://[::ffff:101:101]/public",
    ] {
        assert_eq!(
            parse_safe_link_preview_url(raw_url).expect("mapped public URL"),
            url::Url::parse(raw_url).unwrap()
        );
    }
}

#[test]
fn mapped_ip_redirects_revalidate_private_and_public_destinations() {
    let current = url::Url::parse("https://preview.example.test/a/b").unwrap();
    for destination in MAPPED_PRIVATE_LINK_URLS {
        assert_eq!(
            resolve_safe_link_preview_redirect(&current, destination),
            Err(SandLinkPreviewError::NonPublicIp),
            "redirect to {destination} must fail IP admission"
        );
    }

    assert_eq!(
        resolve_safe_link_preview_redirect(&current, "../next?q=public")
            .unwrap()
            .as_str(),
        "https://preview.example.test/next?q=public"
    );
    for destination in [
        "https://[::ffff:1.1.1.1]/public",
        "https://[::ffff:101:101]/public",
    ] {
        assert_eq!(
            resolve_safe_link_preview_redirect(&current, destination)
                .expect("mapped public redirect"),
            url::Url::parse(destination).unwrap()
        );
    }
}

#[test]
fn dns_resolution_rejects_mapped_private_answers_in_every_order() {
    let url = url::Url::parse("https://preview.example.test/page").unwrap();
    let public = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    for private in ["127.0.0.1", "10.1.2.3", "169.254.169.254"] {
        let ipv4: Ipv4Addr = private.parse().unwrap();
        let mapped = IpAddr::V6(ipv4.to_ipv6_mapped());
        for answers in [vec![public, mapped], vec![mapped, public], vec![mapped]] {
            let mut resolver_calls = 0;
            let result = get_safe_link_preview_connection_target_with(&url, |hostname| {
                resolver_calls += 1;
                assert_eq!(hostname, "preview.example.test");
                Ok(answers.clone())
            });
            assert_eq!(resolver_calls, 1);
            assert_eq!(
                result,
                Err(SandLinkPreviewError::NonPublicIp),
                "unsafe DNS answer set {answers:?} must not select a target"
            );
        }
    }
}

#[test]
fn public_mapped_dns_target_preserves_host_sni_and_https_port() {
    let url = url::Url::parse("https://PREVIEW.example.test:443/media").unwrap();
    let mapped: IpAddr = "::ffff:1.1.1.1".parse().unwrap();
    let mut resolver_calls = 0;
    let target = get_safe_link_preview_connection_target_with(&url, |hostname| {
        resolver_calls += 1;
        assert_eq!(hostname, "preview.example.test");
        Ok(vec![mapped])
    })
    .expect("mapped public DNS answer");

    assert_eq!(resolver_calls, 1);
    assert_eq!(target.connect_host, mapped.to_string());
    assert_eq!(target.socket_addr.ip(), mapped);
    assert_eq!(target.socket_addr.port(), 443);
    assert_eq!(target.host_header, "preview.example.test");
    assert_eq!(target.servername.as_deref(), Some("preview.example.test"));
}

#[test]
fn mapped_ip_literal_targets_never_resolve_dns() {
    for raw_url in MAPPED_PRIVATE_LINK_URLS {
        let url = url::Url::parse(raw_url).expect("mapped private URL fixture");
        let result = get_safe_link_preview_connection_target_with(&url, |_| {
            panic!("a private literal must fail before DNS resolution")
        });
        assert_eq!(result, Err(SandLinkPreviewError::NonPublicIp));
    }

    for raw_url in [
        "https://[::ffff:1.1.1.1]/public",
        "https://[::ffff:101:101]/public",
    ] {
        let url = url::Url::parse(raw_url).unwrap();
        let target = get_safe_link_preview_connection_target_with(&url, |_| {
            panic!("a public literal must connect without DNS resolution")
        })
        .expect("mapped public literal target");
        let mapped: IpAddr = "::ffff:1.1.1.1".parse().unwrap();
        assert_eq!(target.connect_host.parse::<IpAddr>().unwrap(), mapped);
        assert_eq!(target.socket_addr.ip(), mapped);
        assert_eq!(target.socket_addr.port(), 443);
        assert_eq!(target.host_header, url.host_str().unwrap());
        assert_eq!(target.servername, None);
    }
}
