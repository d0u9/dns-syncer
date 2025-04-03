use async_trait::async_trait;

use crate::error::{Error, Result};
use crate::wrapper::http;

use super::Fetcher;
use crate::record::{Record, RecordSet};

#[derive(Debug, Clone, Default)]
pub struct HttpFetcher;

impl HttpFetcher {
    pub async fn fetch(&self) -> Result<RecordSet> {
        let mut ret = RecordSet::default();

        ret.records.push(CloudflareFetcher::fetch_v4().await?);

        Ok(ret)
    }
}

#[async_trait]
impl Fetcher for HttpFetcher {
    async fn fetch(&self) -> Result<RecordSet> {
        self.fetch().await
    }
}

struct CloudflareFetcher;

impl CloudflareFetcher {
    pub async fn fetch_v4() -> Result<Record> {
        let url = "https://1.1.1.1/cdn-cgi/trace";
        let response = http::get(url).await?;
        if response.status != 200 {
            return Err(Error::HttpError(format!(
                "status not 200: {}",
                response.status
            )));
        }

        let ip = Self::parse_content_v4(&response.body).await?;
        let record = Record::new_v4(ip.parse()?);
        Ok(record)
    }

    pub async fn parse_content_v4(content: &str) -> Result<String> {
        let ip = content
            .lines()
            .find(|line| line.starts_with("ip="))
            .map_or(
                Err(Error::ParseError(String::from(
                    "cannot find ip in cloudflare response v4",
                ))),
                |line| {
                    let parts = line.splitn(2, '=').collect::<Vec<&str>>();
                    if parts.len() == 2 {
                        Ok(parts[1])
                    } else {
                        Err(Error::ParseError(String::from(
                            "ip keyword find in cloudflare response v4, but cannot get ip",
                        )))
                    }
                },
            )?;
        Ok(ip.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cloudflare_fetcher() {
        let ip = CloudflareFetcher::fetch_v4().await.unwrap();
        println!("{:?}", ip);
    }

    #[tokio::test]
    async fn test_cloudflare_fetcher_parse_content() {
        let content = r#"fl=490f68
h=1.1.1.1
ip=15.248.4.85
ts=1743642238.374
visit_scheme=https
uag=
colo=SYD
sliver=none
http=http/1.1
loc=AU
tls=TLSv1.2
sni=plaintext
warp=off
gateway=off
rbi=off
kex=P-256"#;
        let ip = CloudflareFetcher::parse_content_v4(content).await.unwrap();
        println!("{}", ip);
    }

    #[tokio::test]
    async fn test_http_fetcher() {
        let fetcher = HttpFetcher;
        let records = fetcher.fetch().await.unwrap();
        println!("{:?}", records);
    }
}
