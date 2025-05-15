use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::error::{Error, Result};

#[derive(Debug, Clone)]
pub enum HeaderKey {
    Authorization,
    ContentType,
    Custom(String),
}

impl HeaderKey {
    fn as_str(&self) -> &str {
        match self {
            HeaderKey::Authorization => "Authorization",
            HeaderKey::ContentType => "Content-Type",
            HeaderKey::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

impl Response {
    pub fn into_body(self) -> Result<String> {
        if self.status == 200 {
            Ok(self.body)
        } else {
            Err(Error::HttpError(format!("status: {}", self.status)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    key: HeaderKey,
    value: String,
}

impl Header {
    pub fn new(key: HeaderKey, value: String) -> Self {
        Self { key, value }
    }
}

pub struct Client {
    cli: reqwest::Client,
    dft_headers: Vec<Header>,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        Self {
            cli: reqwest::Client::new(),
            dft_headers: vec![],
        }
    }

    pub fn set_default_headers(&mut self, headers: Vec<Header>) {
        self.dft_headers = headers;
    }

    pub async fn get(&self, url: &str, headers: Option<Vec<Header>>) -> Result<Response> {
        let mut builder = self.cli.get(url);
        builder = self.add_headers(builder, headers);

        let response = builder.send().await?;
        Ok(Response {
            status: response.status().into(),
            body: response.text().await?,
        })
    }

    pub async fn post(
        &self,
        url: &str,
        headers: Option<Vec<Header>>,
        body: String,
    ) -> Result<Response> {
        let mut builder = self.cli.post(url);
        builder = self.add_headers(builder, headers);

        let response = builder.body(body).send().await?;
        Ok(Response {
            status: response.status().into(),
            body: response.text().await?,
        })
    }

    fn add_headers(
        &self,
        mut builder: reqwest::RequestBuilder,
        headers: Option<Vec<Header>>,
    ) -> reqwest::RequestBuilder {
        let mut hdrs = self.dft_headers.clone();
        if let Some(headers) = headers {
            hdrs.extend(headers);
        }

        for header in hdrs {
            builder = builder.header(header.key.as_str(), header.value.as_str());
        }

        builder
    }
}

#[allow(dead_code)]
pub async fn get_body(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    if response.status().is_success() {
        Ok(response.text().await?)
    } else {
        Err(Error::HttpError(format!(
            "get returns non-200 status: {}",
            response.status()
        )))
    }
}

pub async fn get_body_v4(url: &str) -> Result<String> {
    do_get_body(url, Some(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))).await
}

pub async fn get_body_v6(url: &str) -> Result<String> {
    do_get_body(url, Some(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)))).await
}

pub async fn do_get_body(url: &str, addr: Option<std::net::IpAddr>) -> Result<String> {
    let response = reqwest::Client::builder()
        .local_address(addr)
        .build()?
        .get(url)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(response.text().await?)
    } else {
        Err(Error::HttpError(format!(
            "get returns non-200 status: {}",
            response.status()
        )))
    }
}
