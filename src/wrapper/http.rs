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

    pub fn into_json(self) -> Result<serde_json::Value> {
        let body = self.into_body()?;
        let json: serde_json::Value = serde_json::from_str(&body)?;
        Ok(json)
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

// A simple wrapper for reqwest::get
pub async fn get(url: &str) -> Result<Response> {
    let response = reqwest::get(url).await?;
    Ok(Response {
        status: response.status().into(),
        body: response.text().await?,
    })
}
