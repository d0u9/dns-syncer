use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::error::Result;
use crate::provider::Provider;
use crate::record::ConfigRecord;
use crate::record::ProviderRecord;
use crate::record::RecordContent;
use crate::record::TTL;
use crate::wrapper::http;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "value")]
pub(super) enum Auth {
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
    async fn sync(&self) -> Result<()> {
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

pub(super) struct Cli {
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
pub(super) struct CfResponse {
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
pub(super) struct CfZone {
    pub id: String,
    pub name: String,
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
pub(super) struct CfRecord {
    pub id: String,
    pub name: String,
    pub comment: Option<String>,
    pub proxied: bool,
    pub ttl: u32,

    #[serde(flatten)]
    pub content: RecordContent,
}

impl From<ProviderRecord> for CfRecord {
    fn from(record: ProviderRecord) -> Self {
        Self {
            id: String::new(),
            name: record.name,
            comment: record.comment,
            content: record.content,
            ttl: match record.ttl {
                TTL::Auto => 1,
                TTL::Value(v) => v,
            },
            proxied: record
                .params
                .iter()
                .any(|p| p.name == "proxied" && p.value == "true"),
        }
    }
}

// Cloudflare record API
impl Cli {
    pub async fn records_list(&self, zone_id: &str) -> Result<Vec<CfRecord>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone_id
        );
        let resp = self.get(&url).await?;
        let resp: CfResponse = serde_json::from_str(&resp.into_body()?)?;
        let jsonbody = resp.into_json()?;
        let records: Vec<CfRecord> = serde_json::from_value(jsonbody)?;
        Ok(records)
    }

    pub async fn records_list_by_name(&self, zone_id: &str, name: &str) -> Result<Vec<CfRecord>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}",
            zone_id, name
        );
        let resp = self.get(&url).await?;
        let resp: CfResponse = serde_json::from_str(&resp.into_body()?)?;
        let jsonbody = resp.into_json()?;
        let records: Vec<CfRecord> = serde_json::from_value(jsonbody)?;
        Ok(records)
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
    pub async fn record_op_create(&self, zone_id: &str, record: ProviderRecord) -> Result<()> {
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

    pub async fn record_op_purge(&self, zone_id: &str, record: ProviderRecord) -> Result<()> {
        let rcd = self.records_list_by_name(zone_id, &record.name).await?;
        let deletes: Vec<BatchRecordDelete> = rcd
            .iter()
            .map(|r| BatchRecordDelete { id: r.id.clone() })
            .collect();
        let batch = BatchRecord {
            deletes: Some(deletes),
            patches: None,
            posts: Some(vec![record.into()]),
        };

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/batch",
            zone_id
        );
        let body = serde_json::to_string(&batch)?;
        let resp = self.post(&url, &body).await?;
        let json = resp.into_body().map_err(|e| {
            Error::HttpError(format!("force overwrite record failed: {}", e.to_string()))
        })?;

        let resp: CfResponse = serde_json::from_str(&json)?;

        resp.into_json().map_err(|e| {
            Error::HttpError(format!(
                "force overwrite record failed from cloudflare: {}",
                e.to_string()
            ))
        })?;
        Ok(())
    }
}
