use serde::{Deserialize, Serialize};

use crate::error::Result;

use super::Record;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum RecordOp {
    #[serde(alias = "create")]
    Create, // create a new record if not exists, do nothing if exists

    #[serde(alias = "delete")]
    Delete, // delete the existing record if exists, do nothing if not exists

    #[serde(alias = "purge")]
    Purge, // Remove all records with the same name, and create new one.
}

impl Default for RecordOp {
    fn default() -> Self {
        Self::Create
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RecordLabel {
    key: String,

    #[serde(alias = "value")]
    val: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Backend {
    provider: String,
    zone: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RecordMeta {
    #[serde(default)]
    comment: Option<String>,

    #[serde(default)]
    labels: Vec<RecordLabel>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RecordCfg {
    #[serde(flatten)]
    record: Record,

    #[serde(default)]
    op: RecordOp,

    backends: Vec<Backend>,

    #[serde(flatten)]
    meta: RecordMeta,
}

/// RecordCfgSet
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RecordCfgSet {
    records: Vec<RecordCfg>,
}

impl RecordCfgSet {
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let record_cfg_set: RecordCfgSet = serde_yaml::from_str(yaml)?;
        Ok(record_cfg_set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_cfg_deserialize_with_sync_to() {
        let yaml = r#"
type: A
name: case1.dns-syncer-test
proxied: true
content: 8.8.8.8
comment: 'DNS Syncer, google dns'
op: create
labels:
  - key: "proxy"
    val: "true"
backends:
  - provider: "cloudflare-1"
    zone: "example-au.org"
"#;

        let record_cfg: RecordCfg = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(record_cfg.op, RecordOp::Create);
        assert_eq!(
            record_cfg.meta.comment,
            Some("DNS Syncer, google dns".to_string())
        );
        assert_eq!(
            record_cfg.meta.labels,
            vec![RecordLabel {
                key: "proxy".to_string(),
                val: "true".to_string()
            }]
        );
        assert_eq!(
            record_cfg.backends,
            vec![Backend {
                provider: "cloudflare-1".to_string(),
                zone: "example-au.org".to_string(),
            }]
        );
    }
}
