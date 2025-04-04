use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::Deserialize;

use dns_syncer::error::Result;
use dns_syncer::record::RecordContent;
use dns_syncer::record::RecordOp;
use dns_syncer::record::TTL;
use dns_syncer::record::ZoneName;

#[derive(Debug, Clone, Deserialize)]
pub struct CfgRecord {
    pub name: String,

    #[serde(flatten)]
    pub content: RecordContent,

    #[serde(default)]
    pub op: RecordOp,

    #[serde(default)]
    pub comment: Option<String>,

    #[serde(default)]
    pub ttl: TTL,
}

impl CfgRecord {
    pub fn or_content(mut self, content: RecordContent) -> Self {
        if self.content.is_none() {
            self.content = content;
        }
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgRecordItem {
    #[serde(flatten)]
    pub record: CfgRecord,

    pub backends: Vec<CfgRecordBackend>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgRecordBackendParams {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgRecordBackend {
    pub provider: String,
    pub zones: Vec<ZoneName>,

    #[serde(default)]
    pub params: Vec<CfgRecordBackendParams>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgProviderAuthenticationParams {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgProviderAuthentication {
    pub method: String,
    pub params: Vec<CfgProviderAuthenticationParams>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgProvider {
    pub name: String,
    pub r#type: String,
    pub authentication: CfgProviderAuthentication,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgFetcher {
    pub name: String,
    pub r#type: String,
    pub alive: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Cfg {
    pub check_interval: u64,
    pub providers: Vec<CfgProvider>,
    pub fetchers: Vec<CfgFetcher>,
    pub records: Vec<CfgRecordItem>,
}

////////////////////////////////////////////////////////////
// Yaml parser
////////////////////////////////////////////////////////////
pub struct Parser;

impl Parser {
    pub fn parse_yaml<P: AsRef<Path>>(path: P) -> Result<Cfg> {
        let reader = Self::file_reader(path)?;
        let config: Cfg = serde_yaml::from_reader(reader)?;
        Ok(config)
    }

    fn file_reader<P: AsRef<Path>>(path: P) -> Result<BufReader<File>> {
        let f = std::fs::File::open(path)?;
        Ok(BufReader::new(f))
    }
}

////////////////////////////////////////////////////////////
// Unit test
////////////////////////////////////////////////////////////
#[cfg(test)]
mod unit_test {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_record_cfg_deserialize_with_sync_to() {
        let yaml = r#"
type: A
name: case1.dns-syncer-test
proxied: true
content: 8.8.8.8
comment: 'DNS Syncer, google dns'
ttl: 300
op: create
backends:
- provider: "cloudflare-1"
  params:
    - name: "proxied"
      value: "true"
  zones:
    - "example-au.org"
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
        assert_eq!(cfg_record.backends.len(), 1);
        assert_eq!(cfg_record.backends[0].provider, "cloudflare-1");
        assert_eq!(cfg_record.backends[0].zones.len(), 1);
        assert_eq!(cfg_record.record.ttl, TTL::Value(300));
    }

    #[test]
    fn test_record_cfg_deserialize_without_content() {
        let yaml = r#"
type: A
name: case1.dns-syncer-test
proxied: true
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

        let cfg_record: CfgRecordItem = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg_record.record.name, "case1.dns-syncer-test");
        assert_eq!(cfg_record.record.content, RecordContent::None);
        assert_eq!(
            cfg_record.record.comment,
            Some("DNS Syncer, google dns".to_string())
        );
        assert_eq!(cfg_record.record.op, RecordOp::Create);
        assert_eq!(cfg_record.backends.len(), 1);
        assert_eq!(cfg_record.backends[0].provider, "cloudflare-1");
        assert_eq!(cfg_record.backends[0].zones.len(), 1);
        assert_eq!(cfg_record.record.ttl, TTL::Auto);
    }
}
