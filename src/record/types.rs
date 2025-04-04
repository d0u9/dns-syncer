use std::net::{Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum RecordValue {
    A(Ipv4Addr),
    AAAA(Ipv6Addr),
    CNAME(String),
    None,
}

impl Default for RecordValue {
    fn default() -> Self {
        Self::None
    }
}

/// TTL value for a record
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RecordA {
    pub name: Option<String>,

    #[serde(alias = "content")]
    pub value: Option<Ipv4Addr>,

    #[serde(default)]
    pub ttl: RecordTTL,
}

/// A record for an AAAA record
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RecordAAAA {
    pub name: Option<String>,

    #[serde(alias = "content")]
    pub value: Option<Ipv6Addr>,

    #[serde(default)]
    pub ttl: RecordTTL,
}

/// A record for a CNAME record
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RecordCNAME {
    pub name: Option<String>,

    #[serde(alias = "content")]
    pub value: Option<String>,

    #[serde(default)]
    pub ttl: RecordTTL,
}

/// DNS record
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Record {
    A(RecordA),
    AAAA(RecordAAAA),
    CNAME(RecordCNAME),
}

impl Record {
    pub fn new_v4_none_name(ip: Ipv4Addr) -> Self {
        Self::A(RecordA {
            name: None,
            value: Some(ip),
            ttl: RecordTTL::Auto,
        })
    }
}

/// A set of records
#[derive(Debug, Clone, PartialEq)]
pub struct RecordSet {
    pub records: Vec<Record>,
}

impl Default for RecordSet {
    fn default() -> Self {
        Self { records: vec![] }
    }
}

impl RecordSet {
    pub fn new(records: Vec<Record>) -> Self {
        Self { records }
    }

    pub fn push(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn first_a(&self) -> Option<&RecordA> {
        self.records
            .iter()
            .find(|r| matches!(r, Record::A(_)))
            .map(|r| match r {
                Record::A(record) => record,
                _ => panic!("record is not a A record"),
            })
    }

    pub fn first_aaaa(&self) -> Option<&RecordAAAA> {
        self.records
            .iter()
            .find(|r| matches!(r, Record::AAAA(_)))
            .map(|r| match r {
                Record::AAAA(record) => record,
                _ => panic!("record is not a AAAA record"),
            })
    }

    pub fn first_cname(&self) -> Option<&RecordCNAME> {
        self.records
            .iter()
            .find(|r| matches!(r, Record::CNAME(_)))
            .map(|r| match r {
                Record::CNAME(record) => record,
                _ => panic!("record is not a CNAME record"),
            })
    }
}

#[cfg(test)]
mod test_deserialize {
    use super::*;
    use serde_yaml;

    #[test]
    fn test_record() {
        let yaml = r#"
type: A
name: example.com
value: 1.2.3.4
ttl: auto
        "#;

        let record: Record = serde_yaml::from_str(yaml).unwrap();
        if let Record::A(record) = record {
            assert_eq!(record.value, Some(Ipv4Addr::new(1, 2, 3, 4)));
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

        let record: RecordA = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(record.value, Some(Ipv4Addr::new(1, 2, 3, 4)));
        assert_eq!(record.ttl, RecordTTL::Auto);
        assert_eq!(record.name, None);
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
        assert_eq!(record.value, Some(Ipv4Addr::new(1, 2, 3, 4)));
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
        assert_eq!(record.value, Some(Ipv4Addr::new(1, 2, 3, 4)));
        assert_eq!(record.ttl, RecordTTL::Value(3560));
    }
}
