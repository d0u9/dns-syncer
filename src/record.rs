use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use serde::Deserialize;
use serde::Deserializer;

use crate::error::Error;
use crate::error::Result;

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

    pub fn ips(&self) -> (Option<Ipv4Addr>, Option<Ipv6Addr>) {
        (self.v4, self.v6)
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
pub enum RecordType {
    A,
    AAAA,
    CNAME,
    None,
}

impl RecordType {
    pub fn as_str(&self) -> &str {
        match self {
            RecordType::A => "A",
            RecordType::AAAA => "AAAA",
            RecordType::CNAME => "CNAME",
            RecordType::None => "None",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordContent {
    A(Ipv4Addr),
    AAAA(Ipv6Addr),
    CNAME(String),
    Unassigned(RecordType),
    Unknown,
}

impl RecordContent {
    pub fn is_unknown(&self) -> bool {
        matches!(self, RecordContent::Unknown)
    }

    pub fn is_unassigned(&self) -> bool {
        matches!(self, RecordContent::Unassigned(_))
    }
}

impl<'de> Deserialize<'de> for RecordContent {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
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
            (Some("a" | "A"), None) => Ok(RecordContent::Unassigned(RecordType::A)),
            (Some("a" | "A"), Some(content)) => {
                let v4: Ipv4Addr = content.parse().map_err(serde::de::Error::custom)?;
                Ok(RecordContent::A(v4))
            }
            (Some("aaaa" | "AAAA"), None) => Ok(RecordContent::Unassigned(RecordType::AAAA)),
            (Some("aaaa" | "AAAA"), Some(content)) => {
                let v6: Ipv6Addr = content.parse().map_err(serde::de::Error::custom)?;
                Ok(RecordContent::AAAA(v6))
            }
            (Some("cname" | "CNAME"), None) => Ok(RecordContent::Unassigned(RecordType::CNAME)),
            (Some("cname" | "CNAME"), Some(content)) => Ok(RecordContent::CNAME(content)),
            (Some(ty), _) => Err(serde::de::Error::custom(format!(
                "Unknown record type: {}",
                ty
            ))),
            (None, _) => Err(serde::de::Error::custom("have to give a type for record")),
        }
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

    pub fn new_v6(value: Ipv6Addr) -> Self {
        Self {
            value: RecordContent::AAAA(value),
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

impl From<FetcherRecordSet> for PublicIp {
    fn from(set: FetcherRecordSet) -> Self {
        let mut v4 = None;
        let mut v6 = None;

        for content in set.contents {
            match content.value {
                RecordContent::A(ip) => v4 = Some(ip),
                RecordContent::AAAA(ip) => v6 = Some(ip),
                _ => {}
            }
        }

        Self::new(v4, v6)
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
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
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

impl ProviderRecord {
    pub fn assign_public_ip_if_unassigned(
        &mut self,
        v4: Option<Ipv4Addr>,
        v6: Option<Ipv6Addr>,
    ) -> Result<()> {
        match (&self.content, (v4, v6)) {
            (RecordContent::Unassigned(RecordType::A), (Some(v4), _)) => {
                self.content = RecordContent::A(v4)
            }
            (RecordContent::Unassigned(RecordType::AAAA), (_, Some(v6))) => {
                self.content = RecordContent::AAAA(v6)
            }
            (RecordContent::Unassigned(ty), (_, _)) => {
                return Err(Error::Provider(format!(
                    "content is declared as {} but neither v4 nor v6 is provided",
                    ty.as_str()
                )));
            }
            _ => {
                return Err(Error::Provider(format!(
                    "content should be have a type like A or AAAA, but it is not. Maybe a bug?",
                )));
            }
        }

        Ok(())
    }
}
