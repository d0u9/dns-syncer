use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::Result;
use crate::record::ProviderRecord;
use crate::record::PublicIp;
use crate::record::ZoneName;

#[async_trait]
pub trait Provider {
    fn name(&self) -> &str;
    async fn sync(&self, records: BackendRecords, public_ip: PublicIp) -> Result<()>;
}

#[derive(Debug, Clone, Default)]
pub struct ZoneRecords {
    pub records: Vec<ProviderRecord>,
}

#[derive(Debug, Clone, Default)]
pub struct BackendRecords {
    pub zones: HashMap<ZoneName, ZoneRecords>,
}
