use serde::Deserialize;

use super::record::RecordContent;
use super::record::ZoneName;

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

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigRecord {
    pub name: String,

    #[serde(flatten)]
    pub content: RecordContent,

    #[serde(default)]
    pub op: RecordOp,

    #[serde(default)]
    pub comment: Option<String>,
}

impl ConfigRecord {
    pub fn or_content(mut self, content: RecordContent) -> Self {
        if self.content.is_none() {
            self.content = content;
        }
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigRecordItem {
    #[serde(flatten)]
    pub record: ConfigRecord,

    pub backends: Vec<ConfigRecordBackend>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigRecordBackendParams {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigRecordBackend {
    pub provider: String,
    pub zones: Vec<ZoneName>,

    #[serde(default)]
    pub params: Vec<ConfigRecordBackendParams>,
}
