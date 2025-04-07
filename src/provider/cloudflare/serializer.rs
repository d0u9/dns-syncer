use serde::{Serialize, ser::SerializeStruct};

use crate::record::RecordContent;

impl Serialize for RecordContent {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RecordContent::A(a) => {
                let mut state = serializer.serialize_struct("RecordA", 2)?;
                state.serialize_field("type", &"A")?;
                state.serialize_field("content", &a.to_string())?;
                state.end()
            }
            RecordContent::AAAA(aaaa) => {
                let mut state = serializer.serialize_struct("RecordAAAA", 2)?;
                state.serialize_field("type", &"AAAA")?;
                state.serialize_field("content", &aaaa.to_string())?;
                state.end()
            }
            RecordContent::CNAME(cname) => {
                let mut state = serializer.serialize_struct("RecordCNAME", 2)?;
                state.serialize_field("type", &"CNAME")?;
                state.serialize_field("content", &cname.to_string())?;
                state.end()
            }
            RecordContent::Unassigned(unassigned) => {
                let mut state = serializer.serialize_struct("RecordUnassigned", 2)?;
                state.serialize_field("type", unassigned.as_str())?;
                state.end()
            }
            RecordContent::Unknown => serializer.serialize_unit(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::record::RecordType;
    use std::net::Ipv4Addr;

    #[test]
    fn test_serialize_record_content() {
        let record_content = RecordContent::A(Ipv4Addr::new(192, 168, 1, 1));
        let serialized = serde_json::to_string(&record_content).unwrap();
        println!("{}", serialized);

        let record_content = RecordContent::Unassigned(RecordType::A);
        let serialized = serde_json::to_string(&record_content).unwrap();
        println!("{}", serialized);
    }
}
