use std::net::{Ipv4Addr, Ipv6Addr};

use async_trait::async_trait;

use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordType {
    A,
    CNAME,
    MX,
    NS,
    PTR,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordValue {
    V4(Ipv4Addr),
    V6(Ipv6Addr),
    Str(String),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordTTL {
    Auto,
    Static(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordMeta {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    r#type: RecordType,
    name: Option<String>,
    value: RecordValue,
    ttl: RecordTTL,
    meta: Option<Vec<RecordMeta>>,
}

impl Record {
    pub fn new_v4(ip: Ipv4Addr) -> Self {
        Self {
            r#type: RecordType::A,
            name: None,
            value: RecordValue::V4(ip),
            ttl: RecordTTL::Auto,
            meta: None,
        }
    }
}

#[async_trait]
pub trait Fetcher {
    async fn fetch(&self) -> Result<Vec<Record>>;
}
