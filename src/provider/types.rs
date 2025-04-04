use async_trait::async_trait;

use crate::error::Result;
use crate::record::PublicIp;
use crate::record::RecordCfgSet;

#[async_trait]
pub trait Provider {
    async fn sync(&self, records: RecordCfgSet, public_ip: PublicIp) -> Result<()>;
}
