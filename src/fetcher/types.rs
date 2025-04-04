use crate::error::Result;
use crate::record::FetcherRecordSet;

use async_trait::async_trait;

#[async_trait]
pub trait Fetcher {
    async fn fetch(&self) -> Result<FetcherRecordSet>;
}
