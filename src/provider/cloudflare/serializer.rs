use serde::{Serialize, ser::SerializeStruct};

use crate::record::RecordA;
use crate::record::RecordAAAA;
use crate::record::RecordCNAME;
use crate::record::RecordEntry;
use crate::record::RecordTTL;

impl Serialize for RecordEntry {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RecordEntry::A(a) => a.serialize(serializer),
            RecordEntry::AAAA(aaaa) => aaaa.serialize(serializer),
            RecordEntry::CNAME(cname) => cname.serialize(serializer),
        }
    }
}

impl Serialize for RecordA {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("RecordA", 4)?;
        state.serialize_field("type", &"A")?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("content", &self.value)?;
        state.serialize_field("ttl", &self.ttl)?;
        state.end()
    }
}

impl Serialize for RecordAAAA {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("RecordAAAA", 4)?;
        state.serialize_field("type", &"AAAA")?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("content", &self.value)?;
        state.serialize_field("ttl", &self.ttl)?;
        state.end()
    }
}

impl Serialize for RecordCNAME {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("RecordCNAME", 4)?;
        state.serialize_field("type", &"CNAME")?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("content", &self.value)?;
        state.serialize_field("ttl", &self.ttl)?;
        state.end()
    }
}

impl Serialize for RecordTTL {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RecordTTL::Auto => serializer.serialize_u32(1),
            RecordTTL::Value(v) => serializer.serialize_u32(*v),
        }
    }
}
