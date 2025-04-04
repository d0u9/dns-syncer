use crate::error::Result;
use crate::record::RecordEntrySet;

use async_trait::async_trait;

#[async_trait]
pub trait Fetcher {
    async fn fetch(&self) -> Result<RecordEntrySet>;
}
