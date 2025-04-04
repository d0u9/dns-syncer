use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::Result;
use crate::record::ProviderRecord;
use crate::record::PublicIp;
use crate::record::ZoneName;

#[async_trait]
pub trait Provider {
    async fn sync(&self, records: BackendRecords, public_ip: PublicIp) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct ZoneRecords {
    pub records: Vec<ProviderRecord>,
}

#[derive(Debug, Default)]
pub struct BackendRecords {
    pub zones: HashMap<ZoneName, ZoneRecords>,
}
