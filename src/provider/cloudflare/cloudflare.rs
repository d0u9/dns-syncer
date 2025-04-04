use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::error::Result;
use crate::provider::Provider;
use crate::record::PublicIp;
use crate::record::Record;
use crate::record::RecordCfgSet;
use crate::record::RecordEntry;
use crate::record::RecordLabel;
use crate::wrapper::http;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "value")]
enum Auth {
    #[serde(alias = "api_token")]
    ApiToken(String),

    #[serde(alias = "api_key")]
    ApiKey { email: String, key: String },
}

struct Cloudflare {
    authentication: Auth,
}

impl Cloudflare {
    pub fn new(authentication: Auth) -> Self {
        Self { authentication }
    }
}

#[async_trait]
impl Provider for Cloudflare {
    async fn sync(&self, records: RecordCfgSet, public_ip: PublicIp) -> Result<()> {
        Ok(())
    }
}

///////////////////////////////////////////////////////////
// Client
///////////////////////////////////////////////////////////
impl Auth {
    fn http_headers(&self) -> Vec<http::Header> {
        match self {
            Auth::ApiToken(token) => vec![http::Header::new(
                http::HeaderKey::Authorization,
                format!("Bearer {}", token),
            )],
            Auth::ApiKey { email, key } => vec![
                http::Header::new(
                    http::HeaderKey::Custom("X-Auth-Email".to_string()),
                    email.to_owned(),
                ),
                http::Header::new(
                    http::HeaderKey::Custom("X-Auth-Key".to_string()),
                    key.to_owned(),
                ),
            ],
        }
    }
}

struct Cli {
    cli: http::Client,
}

impl Cli {
    pub fn new(auth: Auth) -> Self {
        let mut headers = auth.http_headers();
        headers.push(http::Header::new(
            http::HeaderKey::ContentType,
            "application/json".to_string(),
        ));

        let mut cli = http::Client::new();
        cli.set_default_headers(headers);

        Self { cli }
    }
}

// Basic http method wrappers
impl Cli {
    async fn get(&self, url: &str) -> Result<http::Response> {
        let resp = self.cli.get(url, None).await?;
        Ok(resp)
    }

    async fn post(&self, url: &str, body: &str) -> Result<http::Response> {
        let resp = self.cli.post(url, None, body.to_string()).await?;
        Ok(resp)
    }

    async fn put(&self) -> Result<String> {
        Err(Error::NotImplemente)
    }

    async fn delete(&self) -> Result<String> {
        Err(Error::NotImplemente)
    }
}

// Cloudflare API response
#[derive(Debug, Clone, Deserialize)]
struct CfResponse {
    success: bool,
    result: serde_json::Value,
}

impl CfResponse {
    fn into_json(self) -> Result<serde_json::Value> {
        if self.success {
            Ok(self.result)
        } else {
            Err(Error::ParseError(format!(
                "cloudflare api call failed: {:?}",
                self.result
            )))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct CfZone {
    id: String,
    name: String,
}

// Cloudflare zone API
impl Cli {
    pub async fn zone_list(&self, name: &str) -> Result<Option<CfZone>> {
        let url = format!("https://api.cloudflare.com/client/v4/zones?name={}", name);
        let resp = self.get(&url).await?;
        let resp: CfResponse = serde_json::from_str(&resp.into_body()?)?;
        let zones: Vec<CfZone> = serde_json::from_value(resp.into_json()?)?;

        match zones.len() {
            0 => Ok(None),
            1 => Ok(Some(zones.into_iter().nth(0).unwrap())),
            _ => Err(Error::ParseError(format!("multiple zones found: {}", name))),
        }
    }
}

// Cloudflare record
#[derive(Debug, Clone, Deserialize, Serialize)]
struct CfRecord {
    #[serde(flatten)]
    entry: RecordEntry,

    comment: Option<String>,
    proxied: bool,
}

impl From<Record> for CfRecord {
    fn from(record: Record) -> Self {
        let (entry, meta) = record.split();

        let (comment, labels) = meta.split();
        Self {
            entry,
            comment: comment,
            proxied: labels.iter().any(|l| l.tuple_ref() == ("proxied", "true")),
        }
    }
}

impl From<CfRecord> for Record {
    fn from(record: CfRecord) -> Self {
        let CfRecord {
            entry,
            comment,
            proxied,
        } = record;

        let label = RecordLabel::new("proxied".to_string(), proxied.to_string());
        let mut slf = Self::new(entry);
        if let Some(comment) = comment {
            slf.set_comment(comment);
        }
        slf.append_label(label);
        slf
    }
}

// Cloudflare record API
impl Cli {
    pub async fn records_list(&self, zone_id: &str) -> Result<Vec<Record>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );
        let resp = self.get(&url).await?;
        let resp: CfResponse = serde_json::from_str(&resp.into_body()?)?;
        let jsonbody = resp.into_json()?;
        let records: Vec<CfRecord> = serde_json::from_value(jsonbody)?;
        Ok(records.into_iter().map(|r| r.into()).collect())
    }

    pub async fn records_list_by_name(&self, zone_id: &str, name: &str) -> Result<Vec<Record>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}",
            zone_id, name
        );
        let resp = self.get(&url).await?;
        let resp: CfResponse = serde_json::from_str(&resp.into_body()?)?;
        let jsonbody = resp.into_json()?;
        let records: Vec<CfRecord> = serde_json::from_value(jsonbody)?;
        Ok(records.into_iter().map(|r| r.into()).collect())
    }
}

/// Cloudflare record API operations by op
#[derive(Debug, Clone, Serialize)]
struct BatchRecordDelete {
    id: String,
}

#[derive(Debug, Clone, Serialize)]
struct BatchRecord {
    deletes: Option<Vec<BatchRecordDelete>>,
    patches: Option<Vec<CfRecord>>,
    posts: Option<Vec<CfRecord>>,
}

impl Cli {
    pub async fn record_op_create(&self, zone_id: &str, record: Record) -> Result<()> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );
        let cf_record = CfRecord::from(record);
        let body = serde_json::to_string(&cf_record)?;
        println!("{}", body);
        let resp = self.post(&url, &body).await?;
        let resp: CfResponse =
            serde_json::from_str(&resp.into_body().map_err(|e| {
                Error::HttpError(format!("create record failed: {}", e.to_string()))
            })?)?;
        resp.into_json().map_err(|e| {
            Error::HttpError(format!(
                "create record failed from cloudflare: {}",
                e.to_string()
            ))
        })?;

        Ok(())
    }
}
///////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;
    use std::net::Ipv4Addr;

    mod test_record_op {
        use super::*;
        use crate::record::RecordA;
        use crate::record::RecordTTL;

        #[tokio::test]
        async fn test_cf_record_op_create() {
            let (zone_name, zone_id) = zone_name();
            let entry = RecordEntry::A(RecordA {
                name: Some(format!("testcf.{}", zone_name)),
                value: Some(Ipv4Addr::new(1, 2, 3, 5)),
                ttl: RecordTTL::Auto,
            });
            let record = Record::new(entry);

            let cli = init_cli();
            let resp = cli.record_op_create(&zone_id, record).await.unwrap();
            println!("{:?}", resp);
        }
    }

    mod test_record {
        use super::*;

        #[tokio::test]
        async fn test_cf_records_list_by_name() {
            let cli = init_cli();
            let (zone_name, zone_id) = zone_name();
            let name = format!("testcf.{}", zone_name);
            let records = cli.records_list_by_name(&zone_id, &name).await.unwrap();
            println!("{:?}", records);
        }

        #[tokio::test]
        async fn test_cf_records_list() {
            let cli = init_cli();
            let (_name, id) = zone_name();
            let records = cli.records_list(&id).await.unwrap();
            println!("{:?}", records);
        }
    }

    mod test_zone {
        use super::*;

        #[tokio::test]
        async fn test_cf_zone_list() {
            let cli = init_cli();
            let (name, id) = zone_name();
            let zone = cli.zone_list(&name).await.unwrap();
            if let Some(zone) = zone {
                assert_eq!(zone.id, id);
            } else {
                panic!("Zone not found");
            }
        }
    }

    fn init_cli() -> Cli {
        let token = std::env::var("CF_API_TOKEN").unwrap();
        let auth = Auth::ApiToken(token);
        Cli::new(auth)
    }

    fn zone_name() -> (String, String) {
        let name = std::env::var("CF_ZONE_NAME").unwrap();
        let id = std::env::var("CF_ZONE_ID").unwrap();
        (name, id)
    }
}

#[cfg(test)]
mod test_deserialize {
    use super::*;
    use serde_yaml;

    #[test]
    fn test_cf_record_deserialize() {
        let json = r#"{
        "comment": null,
        "content": "42.192.202.2",
        "created_on": "2022-06-08T02:19:45.956932Z",
        "id": "79de548c4af681c2af1a9e92be42d004",
        "meta": {},
        "modified_on": "2022-06-08T02:19:45.956932Z",
        "name": "cn.d0u9.top",
        "proxiable": true,
        "proxied": false,
        "settings": {},
        "tags": [],
        "ttl": 1,
        "type": "A"
    }"#;
        let record: CfRecord = serde_json::from_str(json).unwrap();
        println!("{:?}", record);
    }

    #[test]
    fn test_cf_auth_deserialize() {
        let yaml = r#"
type: api_token
value: "1234567890"
        "#;
        let auth: Auth = serde_yaml::from_str(yaml).unwrap();
        if let Auth::ApiToken(token) = auth {
            assert_eq!(token, "1234567890");
        } else {
            panic!("Expected ApiToken");
        }

        let yaml = r#"
type: api_key
value:
  email: "test@example.com"
  key: "1234567890"
        "#;
        let auth: Auth = serde_yaml::from_str(yaml).unwrap();
        if let Auth::ApiKey { email, key } = auth {
            assert_eq!(email, "test@example.com");
            assert_eq!(key, "1234567890");
        } else {
            panic!("Expected ApiKey");
        }
    }
}
