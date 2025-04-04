use std::net::{Ipv4Addr, Ipv6Addr};

use serde::Deserialize;

/// TTL value for a record
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordTTL {
    Auto,
    Value(u32),
}

impl Default for RecordTTL {
    fn default() -> Self {
        Self::Auto
    }
}

impl<'de> Deserialize<'de> for RecordTTL {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TTLVisitor;

        impl<'de> serde::de::Visitor<'de> for TTLVisitor {
            type Value = RecordTTL;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid TTL value")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == "Auto" || v == "auto" {
                    Ok(RecordTTL::Auto)
                } else {
                    Ok(RecordTTL::Value(v.parse().map_err(E::custom)?))
                }
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RecordTTL::Value(v as u32))
            }

            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RecordTTL::Value(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RecordTTL::Value(v as u32))
            }

            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RecordTTL::Value(v as u32))
            }
        }

        deserializer.deserialize_any(TTLVisitor)
    }
}

/// A record for an A record
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RecordA {
    pub name: String,

    #[serde(alias = "content")]
    pub value: Ipv4Addr,

    #[serde(default)]
    pub ttl: RecordTTL,
}

/// A record for an AAAA record
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RecordAAAA {
    pub name: String,

    #[serde(alias = "content")]
    pub value: Ipv6Addr,

    #[serde(default)]
    pub ttl: RecordTTL,
}

/// A record for a CNAME record
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RecordCNAME {
    pub name: String,

    #[serde(alias = "content")]
    pub value: String,

    #[serde(default)]
    pub ttl: RecordTTL,
}

/// DNS record
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum RecordEntry {
    A(RecordA),
    AAAA(RecordAAAA),
    CNAME(RecordCNAME),
}

impl RecordEntry {
    // TODO: deprecate this method
    pub fn new_v4_none_name(ip: Ipv4Addr) -> Self {
        Self::A(RecordA {
            name: "".to_string(),
            value: ip,
            ttl: RecordTTL::Auto,
        })
    }

    pub fn name(&self) -> &str {
        match self {
            Self::A(record) => &record.name,
            Self::AAAA(record) => &record.name,
            Self::CNAME(record) => &record.name,
        }
    }
}

/// A set of records
#[derive(Debug, Clone, PartialEq)]
pub struct RecordEntrySet {
    pub records: Vec<RecordEntry>,
}

impl Default for RecordEntrySet {
    fn default() -> Self {
        Self { records: vec![] }
    }
}

impl RecordEntrySet {
    pub fn new(records: Vec<RecordEntry>) -> Self {
        Self { records }
    }

    pub fn push(&mut self, record: RecordEntry) {
        self.records.push(record);
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn first_a(&self) -> Option<&RecordA> {
        self.records
            .iter()
            .find(|r| matches!(r, RecordEntry::A(_)))
            .map(|r| match r {
                RecordEntry::A(record) => record,
                _ => panic!("record is not a A record"),
            })
    }

    pub fn first_aaaa(&self) -> Option<&RecordAAAA> {
        self.records
            .iter()
            .find(|r| matches!(r, RecordEntry::AAAA(_)))
            .map(|r| match r {
                RecordEntry::AAAA(record) => record,
                _ => panic!("record is not a AAAA record"),
            })
    }

    pub fn first_cname(&self) -> Option<&RecordCNAME> {
        self.records
            .iter()
            .find(|r| matches!(r, RecordEntry::CNAME(_)))
            .map(|r| match r {
                RecordEntry::CNAME(record) => record,
                _ => panic!("record is not a CNAME record"),
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod test_deserialize {
        use super::*;
        use serde_yaml;
        use tokio::sync::AcquireError;

        #[test]
        fn test_record() {
            let yaml = r#"
type: A
name: example.com
value: 1.2.3.4
ttl: auto
        "#;

            let record: RecordEntry = serde_yaml::from_str(yaml).unwrap();
            if let RecordEntry::A(record) = record {
                assert_eq!(record.value, Ipv4Addr::new(1, 2, 3, 4));
                assert_eq!(record.ttl, RecordTTL::Auto);
            } else {
                panic!("record is not a A record");
            }
        }

        #[test]
        fn test_record_a_none_name() {
            let yaml = r#"
type: A
value: 1.2.3.4
ttl: auto
        "#;

            let err = serde_yaml::from_str::<RecordA>(yaml).unwrap_err();
            assert!(err.to_string().contains("missing field `name`"));
        }

        #[test]
        fn test_record_a() {
            let yaml = r#"
type: A
name: example.com
value: 1.2.3.4
ttl: auto
        "#;

            let record: RecordA = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(record.value, Ipv4Addr::new(1, 2, 3, 4));
            assert_eq!(record.ttl, RecordTTL::Auto);
        }

        #[test]
        fn test_record_a_ttl_value() {
            let yaml = r#"
type: A
name: example.com
value: 1.2.3.4
ttl: "3560"
        "#;

            let record: RecordA = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(record.value, Ipv4Addr::new(1, 2, 3, 4));
            assert_eq!(record.ttl, RecordTTL::Value(3560));
        }
    }
}
