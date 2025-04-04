use async_trait::async_trait;
use serde::Deserialize;

use crate::error::Error;
use crate::error::Result;
use crate::provider::Provider;
use crate::record::{PublicIp, RecordCfgSet};
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
    auth: Auth,
}

impl Cli {
    pub fn new(auth: Auth) -> Self {
        Self {
            auth,
            cli: http::Client::new(),
        }
    }
}

// Basic http method wrappers
impl Cli {
    async fn get(&self, url: &str) -> Result<http::Response> {
        let headers = self.auth.http_headers();
        let resp = self.cli.get(url, Some(headers)).await?;
        Ok(resp)
    }

    async fn post(&self, url: &str, body: &str) -> Result<http::Response> {
        let headers = self.auth.http_headers();
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

// Cloudflare API response
#[derive(Debug, Clone, Deserialize)]
struct Response {
    success: bool,
    result: serde_json::Value,
}

impl Response {
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
struct Zone {
    id: String,
    name: String,
}

// Cloudflare zone API
impl Cli {
    pub async fn zone_list(&self, name: &str) -> Result<Option<Zone>> {
        let url = format!("https://api.cloudflare.com/client/v4/zones?name={}", name);
        let resp = self.get(&url).await?;
        let resp: Response = serde_json::from_str(&resp.into_body()?)?;
        let zones: Vec<Zone> = serde_json::from_value(resp.into_json()?)?;

        match zones.len() {
            0 => Ok(None),
            1 => Ok(Some(zones.into_iter().nth(0).unwrap())),
            _ => Err(Error::ParseError(format!("multiple zones found: {}", name))),
        }
    }
}

///////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
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
