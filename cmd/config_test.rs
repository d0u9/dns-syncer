use super::*;
use std::net::Ipv4Addr;

use dns_syncer::provider::Auth;
use dns_syncer::record::RecordType;

#[test]
fn test_record_deserialize_with_content() {
    let yaml = r#"
type: A
name: case1.dns-syncer-test
proxied: true
content: 8.8.8.8
comment: 'DNS Syncer, google dns'
ttl: 300
op: create
providers:
- name: "cloudflare-1"
  params:
    - name: "proxied"
      value: "true"
  zones:
    - "example-au.org"
fetchers:
  - name: "http_fetcher-1"
"#;

    let cfg_record: CfgRecordItem = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg_record.record.name, "case1.dns-syncer-test");
    assert_eq!(
        cfg_record.record.content,
        RecordContent::A(Ipv4Addr::new(8, 8, 8, 8))
    );
    assert_eq!(
        cfg_record.record.comment,
        Some("DNS Syncer, google dns".to_string())
    );
    assert_eq!(cfg_record.record.op, RecordOp::Create);
    assert_eq!(cfg_record.providers.len(), 1);
    assert_eq!(cfg_record.providers[0].name, "cloudflare-1");
    assert_eq!(cfg_record.providers[0].zones.len(), 1);

    assert_eq!(cfg_record.record.ttl, TTL::Value(300));
    assert_eq!(cfg_record.fetchers.len(), 1);
    assert_eq!(cfg_record.fetchers[0].name, "http_fetcher-1");
}

#[test]
fn test_record_cfg_deserialize_without_content() {
    let yaml = r#"
type: A
name: case1.dns-syncer-test
proxied: true
comment: 'DNS Syncer, google dns'
op: create
providers:
- name: "cloudflare-1"
  params:
    - name: "proxied"
      value: "true"
  zones:
    - "example-au.org"
fetchers:
  - name: "http_fetcher-1"
"#;

    let cfg_record: CfgRecordItem = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg_record.record.name, "case1.dns-syncer-test");
    assert_eq!(
        cfg_record.record.content,
        RecordContent::Unassigned(RecordType::A)
    );
    assert_eq!(
        cfg_record.record.comment,
        Some("DNS Syncer, google dns".to_string())
    );
    assert_eq!(cfg_record.record.op, RecordOp::Create);
    assert_eq!(cfg_record.providers.len(), 1);
    assert_eq!(cfg_record.providers[0].name, "cloudflare-1");
    assert_eq!(cfg_record.providers[0].zones.len(), 1);
    assert_eq!(cfg_record.record.ttl, TTL::Auto);
    assert_eq!(cfg_record.fetchers.len(), 1);
    assert_eq!(cfg_record.fetchers[0].name, "http_fetcher-1");
    assert_eq!(cfg_record.providers[0].params.len(), 1);
    assert_eq!(cfg_record.providers[0].params[0].name, "proxied");
    assert_eq!(cfg_record.providers[0].params[0].value, "true");
}

#[test]
fn test_providers_cloudflare_api_token_deserialize() {
    let yaml = r#"
name: cloudflare-1
type: cloudflare
authentication:
  method: api_token
  params:
  - name: api_token
    value: TestToken
    "#;

    let cfg_provider: CfgProvider = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg_provider.name, "cloudflare-1");
    assert_eq!(cfg_provider.r#type, "cloudflare");
    assert_eq!(cfg_provider.authentication.method, "api_token");
    assert_eq!(cfg_provider.authentication.params.len(), 1);
    assert_eq!(cfg_provider.authentication.params[0].name, "api_token");
    assert_eq!(cfg_provider.authentication.params[0].value, "TestToken");

    let auth: Auth = cfg_provider.authentication.try_into().unwrap();
    assert!(matches!(auth, Auth::ApiToken(token) if token == "TestToken"));
}

#[test]
fn test_providers_cloudflare_api_key_deserialize() {
    let yaml = r#"
name: cloudflare-1
type: cloudflare
authentication:
  method: api_key
  params:
  - name: email
    value: test@example.com
  - name: key
    value: 1234567890
    "#;

    let cfg_provider: CfgProvider = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg_provider.name, "cloudflare-1");
    assert_eq!(cfg_provider.r#type, "cloudflare");
    assert_eq!(cfg_provider.authentication.method, "api_key");
    assert_eq!(cfg_provider.authentication.params.len(), 2);
    assert_eq!(cfg_provider.authentication.params[0].name, "email");
    assert_eq!(
        cfg_provider.authentication.params[0].value,
        "test@example.com"
    );
    assert_eq!(cfg_provider.authentication.params[1].name, "key");

    let auth: Auth = cfg_provider.authentication.try_into().unwrap();
    assert!(
        matches!(auth, Auth::ApiKey { email, key } if email == "test@example.com" && key == "1234567890")
    );
}

#[test]
fn test_fetchers_deserialize() {
    let yaml = r#"
name: http_fetcher-1
type: http_fetcher
params:
- name: enabled
  value: "1.1.1.1,ipinfo.io"
    "#;

    let cfg_fetcher: CfgFetcher = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(cfg_fetcher.name, "http_fetcher-1");
    assert_eq!(cfg_fetcher.r#type, "http_fetcher");
    assert_eq!(cfg_fetcher.params.len(), 1);
    assert_eq!(cfg_fetcher.params[0].name, "enabled");
    assert_eq!(cfg_fetcher.params[0].value, "1.1.1.1,ipinfo.io");
}