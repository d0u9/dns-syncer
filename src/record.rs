use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use serde::Deserialize;

pub type ZoneName = String;

#[derive(Debug, Clone, PartialEq)]
pub struct RecordLabel {
    key: String,
    val: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum RecordContent {
    #[serde(alias = "a")]
    A(Ipv4Addr),
    #[serde(alias = "aaaa")]
    AAAA(Ipv6Addr),
    #[serde(alias = "cname")]
    CNAME(String),
    #[serde(alias = "none")]
    None,
}

impl RecordContent {
    pub fn is_none(&self) -> bool {
        matches!(self, RecordContent::None)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FetcherRecord {
    pub value: RecordContent,
    pub labels: Vec<RecordLabel>,
}

impl FetcherRecord {
    pub fn new(value: RecordContent) -> Self {
        Self {
            value,
            labels: vec![],
        }
    }

    pub fn new_v4(value: Ipv4Addr) -> Self {
        Self {
            value: RecordContent::A(value),
            labels: vec![],
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct FetcherRecordSet {
    contents: Vec<FetcherRecord>,
}

impl FetcherRecordSet {
    pub fn new() -> Self {
        Self { contents: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }

    pub fn push(&mut self, content: FetcherRecord) {
        self.contents.push(content);
    }
}

////////////////////////////////////////////////////////////
// Provider Record
////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq)]
pub enum TTL {
    Auto,
    Value(u32),
}

impl Default for TTL {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderParam {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProviderRecord {
    pub name: String,
    pub content: RecordContent,
    pub comment: Option<String>,

    pub ttl: TTL,
    pub params: Vec<ProviderParam>,
}
