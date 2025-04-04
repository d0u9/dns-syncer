use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use serde::Deserialize;
use serde::Deserializer;

////////////////////////////////////////////////////////////
// Public IP
////////////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq)]
pub struct PublicIp {
    v4: Option<Ipv4Addr>,
    v6: Option<Ipv6Addr>,
}

impl PublicIp {
    pub fn new(v4: Option<Ipv4Addr>, v6: Option<Ipv6Addr>) -> Self {
        Self { v4, v6 }
    }
}

////////////////////////////////////////////////////////////
// Record
////////////////////////////////////////////////////////////
pub type ZoneName = String;

#[derive(Debug, Clone, PartialEq)]
pub struct RecordLabel {
    key: String,
    val: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordContent {
    A(Ipv4Addr),
    AAAA(Ipv6Addr),
    CNAME(String),
    None,
}

impl<'de> Deserialize<'de> for RecordContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RecordContentHelper {
            #[serde(rename = "type")]
            ty: Option<String>,
            content: Option<String>,
        }

        let helper = RecordContentHelper::deserialize(deserializer)?;

        match (helper.ty.as_deref(), helper.content) {
            (Some("a" | "A"), Some(content)) => {
                let v4: Ipv4Addr = content.parse().map_err(serde::de::Error::custom)?;
                Ok(RecordContent::A(v4))
            }
            (Some("aaaa" | "AAAA"), Some(content)) => {
                let v6: Ipv6Addr = content.parse().map_err(serde::de::Error::custom)?;
                Ok(RecordContent::AAAA(v6))
            }
            (Some("cname" | "CNAME"), Some(content)) => Ok(RecordContent::CNAME(content)),
            _ => Ok(RecordContent::None),
        }
    }
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
    Value(u32),
    Auto,
}

impl Default for TTL {
    fn default() -> Self {
        Self::Auto
    }
}

impl<'de> Deserialize<'de> for TTL {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_yaml::Value::deserialize(deserializer)?;

        match value.as_u64() {
            Some(int_value) => Ok(TTL::Value(int_value as u32)),
            _ => Ok(TTL::Auto),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum RecordOp {
    #[serde(alias = "create")]
    Create,
    #[serde(alias = "purge")]
    Purge,
}

impl Default for RecordOp {
    fn default() -> Self {
        Self::Create
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
    pub op: RecordOp,
    pub ttl: TTL,

    pub params: Vec<ProviderParam>,
}
