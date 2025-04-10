use crate::error::Result;
use crate::types::FetcherRecordSet;

use async_trait::async_trait;

#[async_trait]
pub trait Fetcher {
    async fn fetch(&mut self) -> Result<FetcherRecordSet>;
}
