use async_trait::async_trait;

use crate::error::{Error, Result};
use crate::wrapper::http;

use super::Fetcher;
use crate::record::FetcherRecord;
use crate::record::FetcherRecordSet;

#[derive(Debug, Clone, Default)]
pub struct HttpFetcher;

impl HttpFetcher {
    pub async fn fetch(&self) -> Result<FetcherRecordSet> {
        let mut ret = FetcherRecordSet::default();
        ret.push(CloudflareFetcher::fetch_v4().await?);
        Ok(ret)
    }
}

#[async_trait]
impl Fetcher for HttpFetcher {
    async fn fetch(&self) -> Result<FetcherRecordSet> {
        self.fetch().await
    }
}

#[async_trait]
trait HttpFetcherBackend {
    fn v4_url<'a>() -> &'a str;
    fn v6_url<'a>() -> &'a str;
    fn parse_content<T: AsRef<str>>(content: T) -> Result<String>;

    async fn fetch_v4() -> Result<FetcherRecord> {
        let url = Self::v4_url();
        let body = http::get_body_v4(url).await?;
        let ip = Self::parse_content(body)?;
        let record = FetcherRecord::new_v4(ip.parse()?);
        Ok(record)
    }

    async fn fetch_v6() -> Result<FetcherRecord> {
        let url = Self::v6_url();
        let body = http::get_body_v6(url).await?;
        let ip = Self::parse_content(body)?;
        let record = FetcherRecord::new_v6(ip.parse()?);
        Ok(record)
    }
}

struct CloudflareFetcher;

impl HttpFetcherBackend for CloudflareFetcher {
    fn v4_url<'a>() -> &'a str {
        "https://1.1.1.1/cdn-cgi/trace"
    }

    fn v6_url<'a>() -> &'a str {
        "https://[2606:4700:4700::1111]/cdn-cgi/trace"
    }

    fn parse_content<T: AsRef<str>>(content: T) -> Result<String> {
        let content = content.as_ref();
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
mod cloudflare_fetcher_tests {
    use super::*;

    #[tokio::test]
    async fn test_fetcher_v4() {
        let ip = CloudflareFetcher::fetch_v4().await.unwrap();
        println!("{:?}", ip);
    }

    #[tokio::test]
    async fn test_fetcher_v6() {
        let ip = CloudflareFetcher::fetch_v6().await.unwrap();
        println!("{:?}", ip);
    }

    #[test]
    fn test_fetcher_parse_v4_content() {
        let content = r#"
fl=490f68
h=1.1.1.1
ip=155.156.157.158
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
        let ip = CloudflareFetcher::parse_content(content).unwrap();
        assert_eq!(ip, "155.156.157.158");
    }

    #[tokio::test]
    async fn test_fetcher_parse_v6_content() {
        let content = r#"
fl=465f162
h=[2606:3007:4007::1111]
ip=2604:5006:8:1d0::4b:d000
ts=1744159940.969
visit_scheme=https
uag=curl/7.88.1
colo=SJC
sliver=none
http=http/2
loc=US
tls=TLSv1.3
sni=off
warp=off
gateway=off
rbi=off
kex=X25519
    "#;
        let ip = CloudflareFetcher::parse_content(content).unwrap();
        assert_eq!(ip, "2604:5006:8:1d0::4b:d000");
    }
}

struct IpwFetcher;

impl HttpFetcherBackend for IpwFetcher {
    fn v4_url<'a>() -> &'a str {
        "http://4.ipw.cn"
    }

    fn v6_url<'a>() -> &'a str {
        "http://6.ipw.cn"
    }

    fn parse_content<T: AsRef<str>>(content: T) -> Result<String> {
        let content = content.as_ref();
        Ok(content.to_string())
    }
}

#[cfg(test)]
mod ipw_fetcher_tests {
    use super::*;

    #[tokio::test]
    async fn test_fetcher_v4() {
        let ip = IpwFetcher::fetch_v4().await.unwrap();
        println!("{:?}", ip);
    }

    #[tokio::test]
    async fn test_fetcher_v6() {
        let ip = IpwFetcher::fetch_v6().await.unwrap();
        println!("{:?}", ip);
    }
}
