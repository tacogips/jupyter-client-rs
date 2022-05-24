use super::kernel::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KernelPostRequest {
    pub name: String,
    pub path: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentList {
    pub name: String,
    pub path: String,
    pub last_modified: String,
    pub created: String,
    pub content: Vec<Content>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Content {
    pub name: String,
    pub path: String,
    pub last_modified: String,
    pub created: String,
    pub content: Option<ContentBody>,
    pub format: Option<String>,
    pub mimetype: Option<String>,
    pub size: Option<i32>,
    pub writable: bool,
    #[serde(rename = "type")]
    pub type_field: ContentType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentPutRequest {
    #[serde(rename = "type")]
    pub file_type: FileType,
    pub content: ContentBody,
}

impl From<Vec<String>> for ContentPutRequest {
    fn from(codes: Vec<String>) -> Self {
        Self {
            file_type: FileType::Notebook,
            content: ContentBody::from(codes),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentBody {
    pub cells: Option<Vec<Cell>>,
    pub metadata: Metadata,
    pub nbformat: i64,
    pub nbformat_minor: i64,
}

impl From<Vec<String>> for ContentBody {
    fn from(codes: Vec<String>) -> Self {
        Self {
            cells: Some(codes.into_iter().map(|each| each.into()).collect()),
            metadata: Metadata::default(),

            nbformat: 4,
            nbformat_minor: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    pub cell_type: CellType,
    pub execution_count: Option<Value>,
    pub id: Option<String>,
    pub metadata: Metadata,
    pub outputs: Vec<Output>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellType {
    #[serde(rename = "code")]
    Code,

    #[serde(rename = "markdown")]
    Markdown,
}

impl Cell {
    pub fn code(code: String) -> Self {
        Self {
            cell_type: CellType::Code,
            execution_count: None,
            id: None,
            metadata: Metadata::default(),
            outputs: vec![],
            source: code,
        }
    }

    pub fn markdown(text: String) -> Self {
        Self {
            cell_type: CellType::Code,
            execution_count: None,
            id: None,
            metadata: Metadata::default(),
            outputs: vec![],
            source: text,
        }
    }
}

impl From<String> for Cell {
    fn from(code: String) -> Self {
        Cell::code(code)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub data: Data,
    pub execution_count: i64,
    pub metadata: Metadata,
    pub output_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Data {
    #[serde(rename = "text/plain")]
    pub text_plain: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub trusted: Option<bool>,
    pub kernelspec: Kernelspec,
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

/// Type of content
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ContentType {
    #[serde(rename = "directory")]
    Directory,
    #[serde(rename = "file")]
    File,
    #[serde(rename = "notebook")]
    Notebook,
}

impl Default for ContentType {
    fn default() -> ContentType {
        Self::Directory
    }
}

impl ContentType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Directory => "directory",
            Self::File => "file",
            Self::Notebook => "notebook",
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub path: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub kernel: Option<Kernel>,
    pub notebook: Notebook,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Kernel {
    pub id: String,
    pub name: String,
    pub last_activity: String,
    pub execution_state: String,
    pub connections: u32,
}
impl Kernel {
    pub fn kernel_client(&self, base_host: &str, secure: bool) -> KernelApiClient {
        KernelApiClient::new(base_host, self.id.as_ref(), secure)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notebook {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    Notebook,
    File,
    Directory,
}
