use std::time::Duration;
use std::time::Instant;

use async_trait::async_trait;

use crate::error::{Error, Result};
use crate::wrapper::http;

use super::Fetcher;
use crate::types::FetcherRecord;
use crate::types::FetcherRecordSet;
use crate::types::Param;
use crate::types::RecordLabel;

#[derive(Clone)]
enum FetcherBackend {
    Cloudflare,
    Ipw,
}

#[derive(Clone)]
pub struct HttpFetcher {
    backends: Vec<FetcherBackend>,
    last_fetch_time: Instant,
    cache_alive_time: Duration,
    cache: Option<FetcherRecordSet>,
}

impl HttpFetcher {
    pub fn new() -> Self {
        Self {
            backends: vec![FetcherBackend::Cloudflare, FetcherBackend::Ipw],
            cache_alive_time: Duration::from_secs(30),
            cache: None,
            last_fetch_time: Instant::now(),
        }
    }

    pub fn new_with_args(args: Vec<Param>) -> Self {
        if args.is_empty() {
            return Self::new();
        }

        let mut enabled_backends: Vec<&str> = vec![];
        let mut cache_alive_time: Duration = Duration::default();

        args.iter().rev().for_each(|param| {
            if param.name == "enabled" {
                enabled_backends = param.value.split(',').collect::<Vec<&str>>();
            } else if param.name == "cache_alive_time" {
                cache_alive_time = Duration::from_secs(param.value.parse::<u64>().unwrap());
            }
        });

        let backends = Self::backends_from_types(enabled_backends);
        Self {
            backends,
            cache_alive_time,
            cache: None,
            last_fetch_time: Instant::now(),
        }
    }

    fn default_backends() -> Vec<&'static str> {
        vec!["cloudflare", "ipw"]
    }

    fn backends_from_types(backend_types: Vec<&str>) -> Vec<FetcherBackend> {
        let ret: Vec<FetcherBackend> = backend_types
            .iter()
            .map(|backend_type| match *backend_type {
                "cloudflare" => FetcherBackend::Cloudflare,
                "ipw" => FetcherBackend::Ipw,
                _ => panic!("unknown http fetcher backend type: {}", backend_type),
            })
            .collect();

        if ret.is_empty() {
            log::warn!(
                "no enabled backends, use default backends: {:?}",
                Self::default_backends()
            );
            vec![FetcherBackend::Cloudflare, FetcherBackend::Ipw]
        } else {
            ret
        }
    }

    async fn do_fetch_from_backends(&self) -> Result<FetcherRecordSet> {
        let mut ret = FetcherRecordSet::default();
        for backend in self.backends.iter() {
            match backend {
                FetcherBackend::Cloudflare => {
                    ret.push(CloudflareFetcher::fetch_v4().await?);
                    ret.push(CloudflareFetcher::fetch_v6().await?);
                }
                FetcherBackend::Ipw => {
                    ret.push(IpwFetcher::fetch_v4().await?);
                    ret.push(IpwFetcher::fetch_v6().await?);
                }
            }
        }
        Ok(ret)
    }

    async fn do_fetch(&mut self) -> Result<FetcherRecordSet> {
        if self.cache.is_none() || self.last_fetch_time.elapsed() > self.cache_alive_time {
            let records = self.do_fetch_from_backends().await?;
            self.cache = Some(records);
            self.last_fetch_time = Instant::now();
        }
        Ok(self.cache.clone().unwrap())
    }
}

#[async_trait]
impl Fetcher for HttpFetcher {
    async fn fetch(&mut self) -> Result<FetcherRecordSet> {
        self.do_fetch().await
    }
}

#[async_trait]
trait HttpFetcherBackend {
    fn v4_url<'a>() -> &'a str;
    fn v6_url<'a>() -> &'a str;
    fn parse_content<T: AsRef<str>>(content: T) -> Result<(String, Vec<RecordLabel>)>;

    async fn fetch_v4() -> Result<FetcherRecord> {
        let url = Self::v4_url();
        let body = http::get_body_v4(url).await?;
        let (ip, labels) = Self::parse_content(body)?;
        let record = FetcherRecord::new_v4_with_labels(ip.parse()?, labels);
        Ok(record)
    }

    async fn fetch_v6() -> Result<FetcherRecord> {
        let url = Self::v6_url();
        let body = http::get_body_v6(url).await?;
        let (ip, labels) = Self::parse_content(body)?;
        let record = FetcherRecord::new_v6_with_labels(ip.parse()?, labels);
        Ok(record)
    }
}

#[cfg(test)]
mod http_fetcher_tests {
    use super::*;

    #[tokio::test]
    async fn test_fetcher() {
        let mut fetcher = HttpFetcher::new();
        let records = fetcher.fetch().await.unwrap();
        dbg!(&records);
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

    fn parse_content<T: AsRef<str>>(content: T) -> Result<(String, Vec<RecordLabel>)> {
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
        let labels = vec![RecordLabel::new(
            String::from("backend"),
            String::from("cloudflare"),
        )];
        Ok((ip.to_string(), labels))
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
        let (ip, labels) = CloudflareFetcher::parse_content(content).unwrap();
        assert_eq!(ip, "155.156.157.158");
        assert_eq!(labels.len(), 1);
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
        let (ip, labels) = CloudflareFetcher::parse_content(content).unwrap();
        assert_eq!(ip, "2604:5006:8:1d0::4b:d000");
        assert_eq!(labels.len(), 1);
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

    fn parse_content<T: AsRef<str>>(content: T) -> Result<(String, Vec<RecordLabel>)> {
        let content = content.as_ref();
        let labels = vec![RecordLabel::new(
            String::from("backend"),
            String::from("ipw"),
        )];
        Ok((content.to_string(), labels))
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
