use crate::error::Result;

pub struct Response {
    pub status: u16,
    pub body: String,
}

pub async fn get(url: &str) -> Result<Response> {
    let response = reqwest::get(url).await?;
    Ok(Response {
        status: response.status().into(),
        body: response.text().await?,
    })
}
