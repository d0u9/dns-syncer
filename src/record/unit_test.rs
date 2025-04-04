use super::*;

use super::config::RecordOp;
use super::record::RecordContent;
use std::net::Ipv4Addr;

#[test]
fn test_record_cfg_deserialize_with_sync_to() {
    let yaml = r#"
type: A
name: case1.dns-syncer-test
proxied: true
content: 8.8.8.8
comment: 'DNS Syncer, google dns'
op: create
backends:
- provider: "cloudflare-1"
  params:
    - name: "proxied"
      value: "true"
  zones:
    - "example-au.org"
"#;

    let cfg_record: ConfigRecordItem = serde_yaml::from_str(yaml).unwrap();
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
    assert_eq!(cfg_record.backends.len(), 1);
    assert_eq!(cfg_record.backends[0].provider, "cloudflare-1");
    assert_eq!(cfg_record.backends[0].zones.len(), 1);
}
