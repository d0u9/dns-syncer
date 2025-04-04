use serde::Deserialize;

use crate::error::Result;

use super::RecordEntry;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RecordLabel {
    key: String,

    #[serde(alias = "value")]
    val: String,
}

impl RecordLabel {
    pub fn new(key: String, val: String) -> Self {
        Self { key, val }
    }

    pub fn tuple_ref(&self) -> (&str, &str) {
        (&self.key, &self.val)
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct RecordMeta {
    #[serde(default)]
    comment: Option<String>,

    #[serde(default)]
    labels: Vec<RecordLabel>,
}

impl RecordMeta {
    pub fn append_label(&mut self, label: RecordLabel) {
        self.labels.push(label);
    }

    pub fn split(self) -> (Option<String>, Vec<RecordLabel>) {
        (self.comment, self.labels)
    }
}

// Record is wrapper with metainfo aside to original RecordEntry
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Record {
    #[serde(flatten)]
    entry: RecordEntry,

    #[serde(flatten)]
    meta: RecordMeta,
}

impl Record {
    pub fn new(entry: RecordEntry) -> Self {
        Self {
            entry,
            meta: RecordMeta::default(),
        }
    }

    pub fn append_label(&mut self, label: RecordLabel) {
        self.meta.append_label(label);
    }

    pub fn set_comment(&mut self, comment: String) {
        self.meta.comment = Some(comment);
    }

    pub fn split(self) -> (RecordEntry, RecordMeta) {
        (self.entry, self.meta)
    }

    pub fn entry(&self) -> &RecordEntry {
        &self.entry
    }
}

// RecordWithOp is used by provider
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RecordWithOp {
    #[serde(flatten)]
    record: Record,

    #[serde(default)]
    op: RecordOp,
}

// RecordCfg is used when parsing from yaml
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Backend {
    provider: String,
    zone: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RecordCfg {
    #[serde(flatten)]
    record_with_op: RecordWithOp,

    backends: Vec<Backend>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum RecordOp {
    #[serde(alias = "create")]
    Create, // create a new record if not exists, do nothing if exists

    #[serde(alias = "purge")]
    Purge, // Remove all records with the same name, and create new one.
}

impl Default for RecordOp {
    fn default() -> Self {
        Self::Create
    }
}

/// RecordCfgSet
#[derive(Debug, Clone, PartialEq, Deserialize)]
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
        assert_eq!(record_cfg.record_with_op.op, RecordOp::Create);
        assert_eq!(
            record_cfg.record_with_op.record.meta.comment,
            Some("DNS Syncer, google dns".to_string())
        );
        assert_eq!(
            record_cfg.record_with_op.record.meta.labels,
            vec![RecordLabel {
                key: "proxy".to_string(),
                val: "true".to_string(),
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
