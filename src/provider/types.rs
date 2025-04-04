use async_trait::async_trait;

use crate::error::Result;

#[async_trait]
pub trait Provider {
    async fn sync(&self) -> Result<()>;
}
