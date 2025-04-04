use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::Result;
use crate::record::ProviderRecord;
use crate::record::ZoneName;

#[async_trait]
pub trait Provider {
    async fn sync(&self) -> Result<()>;
}

pub struct BackendRecords {
    zones: HashMap<ZoneName, ProviderRecord>,
}
