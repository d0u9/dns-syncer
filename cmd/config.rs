use serde::Deserialize;

use super::record::RecordContent;
use super::record::ZoneName;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum RecordOp {
    #[serde(alias = "create")]
    Create,
    #[serde(alias = "purge")]
    Purge,
}

impl Default for RecordOp {
    fn default() -> Self {
        Self::Create
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigRecord {
    pub name: String,

    #[serde(flatten)]
    pub content: RecordContent,

    #[serde(default)]
    pub op: RecordOp,

    #[serde(default)]
    pub comment: Option<String>,
}

impl ConfigRecord {
    pub fn or_content(mut self, content: RecordContent) -> Self {
        if self.content.is_none() {
            self.content = content;
        }
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigRecordItem {
    #[serde(flatten)]
    pub record: ConfigRecord,

    pub backends: Vec<ConfigRecordBackend>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigRecordBackendParams {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigRecordBackend {
    pub provider: String,
    pub zones: Vec<ZoneName>,

    #[serde(default)]
    pub params: Vec<ConfigRecordBackendParams>,
}

#[cfg(test)]
mod unit_test {
    use super::*;
    use super::super::*;
    
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
}
