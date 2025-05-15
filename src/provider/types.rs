use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::Result;
use crate::types::ProviderRecord;
use crate::types::PublicIp;
use crate::types::ZoneName;

#[async_trait]
pub trait Provider {
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
