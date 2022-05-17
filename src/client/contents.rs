use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contents {
    pub name: String,
    pub path: String,
    pub last_modified: String,
    pub created: String,
    pub content: Content,
    pub format: String,
    pub mimetype: Value,
    pub size: i64,
    pub writable: bool,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Content {
    pub cells: Vec<Cell>,
    pub metadata: Metadata,
    pub nbformat: i64,
    pub nbformat_minor: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub cell_type: String,
    pub execution_count: Value,
    pub id: String,
    pub metadata: Metadata,
    pub outputs: Vec<Value>,
    pub source: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub trusted: Option<bool>,
    pub kernelspec: Option<Kernelspec>,
    pub language_info: Option<LanguageInfo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Kernelspec {
    pub display_name: String,
    pub language: String,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub codemirror_mode: String,
    pub file_extension: String,
    pub mimetype: String,
    pub name: String,
    pub pygment_lexer: String,
    pub version: String,
}
