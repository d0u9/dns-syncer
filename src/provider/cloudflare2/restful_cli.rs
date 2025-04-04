use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::wrapper::http::{Client, Header, HeaderKey, Response};

use super::cloudflare::{Auth, CfApiKey, CfRecord, CfRecordOp, CfZone};

impl From<Auth> for Vec<Header> {
    fn from(auth: Auth) -> Self {
        match auth {
            Auth::ApiToken(api_token) => vec![Header::new(
                HeaderKey::Authorization,
                format!("Bearer {}", api_token),
            )],
            Auth::ApiKey(CfApiKey { email, key }) => vec![
                Header::new(HeaderKey::Custom("X-Auth-Email".to_string()), email),
                Header::new(HeaderKey::Custom("X-Auth-Key".to_string()), key),
            ],
        }
    }
}

pub(super) struct CfClient {
    auth: Auth,
    cli: Client,
}

impl CfClient {
    pub fn new(auth: Auth) -> Self {
        Self {
            auth,
            cli: Client::new(),
        }
    }
}

impl CfClient {
    async fn headers(&self) -> Result<Vec<Header>> {
        let mut headers: Vec<Header> = self.auth.clone().into();
        headers.push(Header::new(
            HeaderKey::ContentType,
            "application/json".to_string(),
        ));

        Ok(headers)
    }

    async fn get(&self, url: &str) -> Result<Response> {
        let headers = self.headers().await?;
        let resp = self.cli.get(url, Some(headers)).await?;
        Ok(resp)
    }

    async fn post(&self, url: &str, body: &str) -> Result<Response> {
        let headers = self.headers().await?;
        let resp = self.cli.post(url, Some(headers), body.to_string()).await?;
        Ok(resp)
    }

    async fn put(&self) -> Result<String> {
        Err(Error::NotImplemente)
    }

    async fn delete(&self) -> Result<String> {
        Err(Error::NotImplemente)
    }
}

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
                "cloudflare api unsuccess: {:?}",
                self.result
            )))
        }
    }
}

impl CfClient {
    pub async fn zones_list(&self) -> Result<Vec<CfZone>> {
        let url = "https://api.cloudflare.com/client/v4/zones";
        let resp = self.get(url).await?;
        let jstr = resp.into_body()?;
        let resp: CfResponse = serde_json::from_str(&jstr)?;
        let jzones = resp.into_json()?;

        let zones: Vec<CfZone> = serde_json::from_value(jzones)?;

        Ok(zones)
    }

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

impl CfClient {
    pub async fn record_list(
        &self,
        zoneid: &str,
        name: &str,
        rtype: &str,
    ) -> Result<Option<CfRecord>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?name={}&type={}",
            zoneid, name, rtype
        );
        let resp = self.get(&url).await?;
        let body = resp.into_body()?;
        let resp: CfResponse = serde_json::from_str(&body)?;
        let records: Vec<CfRecord> = serde_json::from_value(resp.into_json()?)?;

        match records.len() {
            0 => Ok(None),
            1 => Ok(Some(records.into_iter().nth(0).unwrap())),
            _ => Err(Error::ParseError(format!(
                "multiple records found: {}",
                name
            ))),
        }
    }

    pub async fn record_op_create(&self, zoneid: &str, record: &CfRecord) -> Result<()> {
        let url = format!("https://api.cloudflare.com/client/v4/zones/{}/dns_records", zoneid);
        let body = serde_json::to_string(&record)?;
        let resp = self.post(&url, &body).await?;
        let resp: CfResponse = serde_json::from_str(&resp.into_body().map_err(|e| 
            Error::HttpError(format!(
                "create record failed: {}",
                e.to_string()
            ))
        )?)?;
        resp.into_json().map_err(|e|
            Error::HttpError(format!(
                "create record failed from cloudflare: {}",
                e.to_string()
            ))
        )?;
        Ok(())
    }

    pub async fn record_op_force_overwrite(&self, zoneid: &str, record: &CfRecord) -> Result<()> {
        let rcd = self
            .record_list(zoneid, &record.name, &record.r#type)
            .await?;
        let deletes = rcd.map(|r| vec![BatchRecordDelete { id: r.id.unwrap() }]);

        let batch = BatchRecord {
            deletes: deletes,
            patches: None,
            posts: Some(vec![record.clone()]),
        };

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/batch",
            zoneid
        );
        let body = serde_json::to_string(&batch)?;
        let resp = self.post(&url, &body).await?;
        let json = resp.into_body().map_err(|e| Error::HttpError(format!(
            "force overwrite record failed: {}",
            e.to_string()
        )))?;

        let resp: CfResponse = serde_json::from_str(&json)?;

        resp.into_json().map_err(|e| Error::HttpError(format!(
            "force overwrite record failed from cloudflare: {}",
            e.to_string()
        )))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cf_record_op_create() {
        let token = std::env::var("CF_API_TOKEN").unwrap();
        let auth = Auth::ApiToken(token);
        let cli = CfClient::new(auth);

        let zone_name = std::env::var("CF_ZONE_NAME").unwrap();
        let zone = cli.zone_list(&zone_name).await.unwrap();

        let record = CfRecord {
            id: None,
            name: format!("{}.{}", "testcf", zone_name),
            r#type: "A".to_string(),
            content: Some("99.6.7.11".to_string()),
            comment: "test dns syncer create test".to_string(),
            op: CfRecordOp::Create,
            proxied: true,
            ttl: Some(3600),
        };

        cli.record_op_create(&zone.clone().unwrap().id, &record).await.unwrap();
    }

    #[tokio::test]
    async fn test_cf_client_sync() {
        let token = std::env::var("CF_API_TOKEN").unwrap();
        let auth = Auth::ApiToken(token);
        let cli = CfClient::new(auth);

        let zone_name = std::env::var("CF_ZONE_NAME").unwrap();
        let zone = cli.zone_list(&zone_name).await.unwrap();
        // println!("zone: {:?}", zone);

        let record = CfRecord {
            id: None,
            name: format!("{}.{}", "testcf", zone_name),
            r#type: "A".to_string(),
            content: Some("3.6.7.10".to_string()),
            comment: "test dns syncer".to_string(),
            op: CfRecordOp::Overwrite,
            proxied: false,
            ttl: None,
        };

        let listed_record = cli
            .record_list(&zone.clone().unwrap().id, &record.name, &record.r#type)
            .await
            .unwrap();
        // println!("listedrecords: {:?}", listed_record);

        let resp = cli
            .record_op_force_overwrite(&zone.clone().unwrap().id, &record)
            .await
            .unwrap();
        println!("resp: {:?}", resp);
    }

    #[test]
    fn test_env_set() {
        let token = std::env::var("CF_API_TOKEN").unwrap();
        println!("token: {}", token);
    }
}
