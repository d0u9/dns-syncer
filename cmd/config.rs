use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::Deserialize;

use dns_syncer::error::Error;
use dns_syncer::error::Result;
use dns_syncer::provider::Auth;
use dns_syncer::record::ProviderParam;
use dns_syncer::record::ProviderRecord;
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
    pub comment: Option<String>,

    #[serde(default)]
    pub op: RecordOp,

    #[serde(default)]
    pub ttl: TTL,
}

impl CfgRecord {
    pub fn into_provider_record(self, params: &Vec<CfgRecordParam>) -> ProviderRecord {
        ProviderRecord {
            name: self.name,
            content: self.content,
            comment: self.comment,
            op: self.op,
            ttl: self.ttl,
            params: params
                .into_iter()
                .map(|p| ProviderParam {
                    name: p.name.clone(),
                    value: p.value.clone(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgRecordItem {
    #[serde(flatten)]
    pub record: CfgRecord,

    pub providers: Vec<CfgRecordProvider>,

    #[serde(default)]
    pub _fetchers: Vec<CfgRecordFetcher>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgRecordParam {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgRecordFetcher {
    pub _name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgRecordProvider {
    pub name: String,
    pub zones: Vec<ZoneName>,

    #[serde(default)]
    pub params: Vec<CfgRecordParam>,
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

impl TryFrom<CfgProviderAuthentication> for Auth {
    type Error = Error;

    fn try_from(cfg: CfgProviderAuthentication) -> Result<Self> {
        match cfg.method.as_str() {
            "api_token" => Ok(Auth::ApiToken(
                cfg.params
                    .iter()
                    .find(|p| p.key == "api_token")
                    .ok_or(Error::Provider(format!(
                        "{}: api_token not found",
                        cfg.method
                    )))?
                    .value
                    .clone(),
            )),
            _ => Err(Error::Provider(format!(
                "{}: unsupported authentication method",
                cfg.method
            ))),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgProvider {
    pub name: String,
    pub r#type: String,
    pub authentication: CfgProviderAuthentication,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CfgFetcher {
    pub name: String,
    pub r#type: String,
    pub alive: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Cfg {
    pub check_interval: u64,
    pub fetchers: Vec<CfgFetcher>,
    pub providers: Vec<CfgProvider>,
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
        assert_eq!(cfg_record._fetchers.len(), 1);
        assert_eq!(cfg_record._fetchers[0]._name, "http_fetcher-1");
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
        assert_eq!(cfg_record.record.content, RecordContent::Unknown);
        assert_eq!(
            cfg_record.record.comment,
            Some("DNS Syncer, google dns".to_string())
        );
        assert_eq!(cfg_record.record.op, RecordOp::Create);
        assert_eq!(cfg_record.providers.len(), 1);
        assert_eq!(cfg_record.providers[0].name, "cloudflare-1");
        assert_eq!(cfg_record.providers[0].zones.len(), 1);
        assert_eq!(cfg_record.record.ttl, TTL::Auto);
        assert_eq!(cfg_record._fetchers.len(), 1);
        assert_eq!(cfg_record._fetchers[0]._name, "http_fetcher-1");
    }
}
