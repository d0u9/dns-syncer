use serde::Deserialize;
use serde_yaml;
use serde_yaml::Value as YamlValue;

use crate::error::{Error, Result};
use crate::fetcher::global;
use crate::record::{RecordSet, RecordValue};

use super::restful_cli::CfClient;

#[derive(Debug, Clone, Deserialize)]
pub(crate) enum CfRecordOp {
    #[serde(alias = "overwrite")]
    Overwrite, // overwrite the existing record

    #[serde(alias = "create")]
    Create, // create a new record if not exists, do nothing if exists

    #[serde(alias = "update")]
    Update, // update the existing record if exists, do nothing if not exists
}

impl Default for CfRecordOp {
    fn default() -> Self {
        Self::Overwrite
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(super) struct CfRecord {
    pub r#type: String,
    pub name: String,

    #[serde(default)]
    pub content: Option<String>,

    #[serde(default)]
    pub proxied: bool,

    #[serde(default)]
    pub comment: String,

    #[serde(default)]
    pub op: CfRecordOp,
}

#[derive(Debug, Clone, Deserialize)]
struct CfAuth {
    api_token: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CfZone {
    id: String,
    records: Vec<CfRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CloudflareCfg {
    authentication: CfAuth,
    zones: Vec<CfZone>,
}

impl TryFrom<YamlValue> for CloudflareCfg {
    type Error = Error;

    fn try_from(yaml: YamlValue) -> std::result::Result<Self, Self::Error> {
        let cfg: CloudflareCfg = serde_yaml::from_value(yaml)?;
        Ok(cfg)
    }
}

pub struct Cloudflare {
    auth: CfAuth,
    zones: Vec<CfZone>,
}

impl Cloudflare {
    pub fn from_cfg(cfg: CloudflareCfg) -> Result<Self> {
        let auth = cfg.authentication;
        let zones = cfg.zones;
        Ok(Self { auth, zones })
    }

    pub async fn sync(&self) -> Result<()> {
        let public_ip = global::fetch().await?;

        let cli = CfClient::new(self.auth.api_token.clone());

        for zone in &self.zones {
            Self::sync_zone(&cli, zone, &public_ip).await?;
        }

        Ok(())
    }

    async fn sync_zone(cli: &CfClient, zone: &CfZone, public_ip: &RecordSet) -> Result<()> {
        let public_v4 = public_ip
            .first_a()
            .map(|r| &r.value)
            .unwrap_or(&RecordValue::None);
        let public_v6 = public_ip
            .first_aaaa()
            .map(|r| &r.value)
            .unwrap_or(&RecordValue::None);

        let cf_records = Self::assign_ip_to_record(&zone.records, public_v4, public_v6)?;

        println!("cf_records: {:?}", cf_records);
        Ok(())
    }

    fn assign_ip_to_record(
        record_cfg: &[CfRecord],
        v4_val: &RecordValue,
        v6_val: &RecordValue,
    ) -> Result<Vec<CfRecord>> {
        let mut ret = Vec::new();

        for record in record_cfg {
            let mut cur_record = record.clone();

            match (record.r#type.as_str(), &record.content, v4_val, v6_val) {
                ("A", None, RecordValue::A(ip), _) => {
                    cur_record.content = Some(ip.to_string());
                }
                ("AAAA", None, _, RecordValue::AAAA(ip)) => {
                    cur_record.content = Some(ip.to_string());
                }
                _ => {}
            }

            ret.push(cur_record);
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_cf_sync() {
        let cf_cfg_yaml = get_cf_cfg_yaml();
        let cfg = CloudflareCfg::try_from(cf_cfg_yaml).unwrap();

        let cf = Cloudflare::from_cfg(cfg).unwrap();
        cf.sync().await.unwrap();
    }

    #[test]
    fn test_cf_cfg_parse() {
        let cf_cfg_yaml = get_cf_cfg_yaml();
        let cfg = CloudflareCfg::try_from(cf_cfg_yaml).unwrap();
        println!("{:?}", cfg);
    }

    fn get_cf_cfg_yaml() -> YamlValue {
        #[derive(Debug, Clone, Deserialize)]
        struct Y {
            providers: Vec<YamlValue>,
        }

        let crate_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let sample_cfg_file = PathBuf::from(crate_root).join("sample_config.yaml");
        let cfg_yaml = std::fs::read_to_string(sample_cfg_file).unwrap();

        let cfg: Y = serde_yaml::from_str(&cfg_yaml).unwrap();
        let cf_cfg_yaml = cfg
            .providers
            .into_iter()
            .find(|provider| provider["type"] == "cloudflare")
            .unwrap();

        cf_cfg_yaml
    }
}
