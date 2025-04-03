use std::net::{Ipv4Addr, Ipv6Addr};

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordTTL {
    Auto,
    Static(u32),
}

impl Default for RecordTTL {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordMeta {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub name: Option<String>,
    pub value: RecordValue,
    pub ttl: RecordTTL,
    pub meta: Option<Vec<RecordMeta>>,
}

impl Record {
    pub fn new_v4(ip: Ipv4Addr) -> Self {
        Self {
            name: None,
            value: RecordValue::A(ip),
            ttl: RecordTTL::Auto,
            meta: None,
        }
    }
}

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

    pub fn first_a(&self) -> Option<&Record> {
        self.records
            .iter()
            .find(|r| matches!(r.value, RecordValue::A(_)))
    }

    pub fn first_aaaa(&self) -> Option<&Record> {
        self.records
            .iter()
            .find(|r| matches!(r.value, RecordValue::AAAA(_)))
    }

    pub fn first_cname(&self) -> Option<&Record> {
        self.records
            .iter()
            .find(|r| matches!(r.value, RecordValue::CNAME(_)))
    }
}
