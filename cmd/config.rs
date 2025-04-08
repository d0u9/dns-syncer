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

////////////////////////////////////////////////////////////
// Parameters
////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Deserialize)]
pub struct CfgParam {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CfgParamList(Vec<CfgParam>);

impl CfgParamList {
    pub fn iter(&self) -> impl Iterator<Item = &CfgParam> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl std::ops::Index<usize> for CfgParamList {
    type Output = CfgParam;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IntoIterator for CfgParamList {
    type Item = CfgParam;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a CfgParamList {
    type Item = &'a CfgParam;
    type IntoIter = std::slice::Iter<'a, CfgParam>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

////////////////////////////////////////////////////////////
// Record
////////////////////////////////////////////////////////////
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
    pub fn into_provider_record(self, params: &CfgParamList) -> ProviderRecord {
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
    pub fetchers: Vec<CfgRecordFetcher>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgRecordFetcher {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgRecordProvider {
    pub name: String,
    pub zones: Vec<ZoneName>,
    pub params: CfgParamList,
}

////////////////////////////////////////////////////////////
// Provider
////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Deserialize)]
pub struct CfgProviderAuthentication {
    pub method: String,
    pub params: CfgParamList,
}

impl CfgProviderAuthentication {
    pub fn get_value_ref(&self, key: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|p| p.name == key)
            .map(|p| p.value.as_str())
    }
}

impl TryFrom<CfgProviderAuthentication> for Auth {
    type Error = Error;

    fn try_from(cfg: CfgProviderAuthentication) -> Result<Self> {
        if cfg.method == "api_token" {
            let api_token = cfg
                .get_value_ref("api_token")
                .ok_or(Error::Provider(format!(
                    "cloudflare authencation method is declared as api_token, but api_token is not found",
                )))?;
            Ok(Auth::ApiToken(api_token.to_string()))
        } else if cfg.method == "api_key" {
            let email = cfg.get_value_ref("email");
            let key = cfg.get_value_ref("key");

            match (email, key) {
                (Some(email), Some(key)) => Ok(Auth::ApiKey {
                    email: email.to_string(),
                    key: key.to_string(),
                }),
                _ => Err(Error::Provider(
                    "cloudflare api_key auth requires both email and key".into(),
                )),
            }
        } else {
            Err(Error::Provider(format!(
                "{}: unsupported authentication method for cloudflare provider",
                cfg.method
            )))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgProvider {
    pub name: String,
    pub r#type: String,
    pub authentication: CfgProviderAuthentication,
}

////////////////////////////////////////////////////////////
// Fetcher
////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CfgFetcher {
    pub name: String,
    pub r#type: String,
    pub params: CfgParamList,
}

////////////////////////////////////////////////////////////
// Yaml parser
////////////////////////////////////////////////////////////
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Cfg {
    pub check_interval: u64,
    pub fetchers: Vec<CfgFetcher>,
    pub providers: Vec<CfgProvider>,
    pub records: Vec<CfgRecordItem>,
}

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
#[path = "config_test.rs"]
mod test;
