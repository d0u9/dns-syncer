use async_trait::async_trait;

use crate::error::Result;
use crate::record::PublicIp;
use crate::record::RecordCfgSet;
use crate::record::RecordWithOp;

#[async_trait]
pub trait Provider {
    async fn sync(&self, records: RecordCfgSet, public_ip: PublicIp) -> Result<()>;
}

pub struct RecordsPerZone {
    pub zone: String,
    pub records: Vec<RecordWithOp>,
}

pub struct RecordForProvier {
    records_per_zone: Vec<RecordsPerZone>,
}

impl RecordForProvier {
    pub fn new(records_per_zone: Vec<RecordsPerZone>) -> Self {
        Self { records_per_zone }
    }

    pub fn group_and_merge(self) -> Self {
        let mut by_key = std::collections::HashMap::new();

        for records_per_zone in self.records_per_zone {
            by_key
                .entry(records_per_zone.zone)
                .or_insert_with(Vec::new)
                .extend(records_per_zone.records);
        }

        let records_per_zone = by_key
            .into_iter()
            .map(|(zone, records)| RecordsPerZone { zone, records })
            .collect();

        Self { records_per_zone }
    }
}
