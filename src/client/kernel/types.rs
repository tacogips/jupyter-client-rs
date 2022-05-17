use super::super::types::Data;
use super::super::JupyterApiError;
use serde::{Deserialize, Serialize};
use serde_json::{error::Error as JsonError, Map as JMap, Value};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    KernelInfoRequest,
    ExecuteRequest,
    ExecuteInput,
    ExecuteReply,
    ExecuteResult,
    Error,
    Status,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KernelRequestHeader {
    username: Option<String>,
    #[serde(rename = "session")]
    session_id: Option<String>,
    #[serde(rename = "msg_id")]
    message_id: Option<String>,
    #[serde(rename = "msg_type")]
    message_type: MessageType,
    date: Option<String>,
    version: String,
}

impl Default for KernelRequestHeader {
    fn default() -> Self {
        KernelRequestHeader {
            username: Some("jupyter-api-client-rs".to_string()),
            session_id: Some(Uuid::new_v4().to_string()),
            message_id: Some(Uuid::new_v4().to_string()),
            message_type: MessageType::ExecuteRequest,
            date: None, //TODO(tacogisp) set
            version: "5.0".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KernelCodeContent {
    code: String,
    silent: bool,
}
impl From<String> for KernelCodeContent {
    fn from(code: String) -> Self {
        Self {
            code,
            silent: false,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KernelCodeRequest {
    header: KernelRequestHeader,
    parent_header: KernelRequestHeader,
    metadata: Value,
    content: KernelCodeContent,
}

impl From<String> for KernelCodeRequest {
    fn from(code: String) -> Self {
        let header = KernelRequestHeader::default();
        Self {
            parent_header: header.clone(),
            header,
            metadata: Value::Object(JMap::new()),
            content: KernelCodeContent::from(code),
        }
    }
}

impl From<&str> for KernelCodeRequest {
    fn from(code: &str) -> Self {
        code.to_string().into()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KernelResponse {
    pub header: Header,
    pub msg_id: String,
    pub msg_type: MessageType,
    pub parent_header: ParentHeader,
    pub metadata: Metadata,
    pub content: Option<Value>,
    pub buffers: Vec<Value>,
    pub channel: String,
}

impl KernelResponse {
    pub fn as_content(&self) -> Result<Option<Content>, JupyterApiError> {
        match &self.msg_type {
            MessageType::ExecuteInput => {
                let r = self.as_execute_input_content()?;
                Ok(r.map(|v| Content::ExecuteInputContent(v)))
            }
            MessageType::ExecuteReply => {
                let r = self.as_execute_reply_content()?;
                Ok(r.map(|v| Content::ExecuteReplyContent(v)))
            }
            MessageType::ExecuteResult => {
                let r = self.as_execute_result_content()?;
                Ok(r.map(|v| Content::ExecuteResultContent(v)))
            }
            MessageType::Error => {
                let r = self.as_error_content()?;
                Ok(r.map(|v| Content::ErrorContent(v)))
            }
            MessageType::Status => {
                let r = self.as_status_content()?;
                Ok(r.map(|v| Content::StatusContent(v)))
            }
            typ => Err(JupyterApiError::InvalieMessageType(format!("{:?}", typ))),
        }
    }

    pub fn as_execute_reply_content(&self) -> Result<Option<ExecuteReplyContent>, JsonError> {
        match self.content.clone() {
            Some(content) => serde_json::from_value(content),
            None => Ok(None),
        }
    }

    pub fn as_execute_input_content(&self) -> Result<Option<ExecuteInputContent>, JsonError> {
        match self.content.clone() {
            Some(content) => serde_json::from_value(content),
            None => Ok(None),
        }
    }

    pub fn as_error_content(&self) -> Result<Option<ErrorContent>, JsonError> {
        match self.content.clone() {
            Some(content) => serde_json::from_value(content),
            None => Ok(None),
        }
    }

    pub fn as_status_content(&self) -> Result<Option<StatusContent>, JsonError> {
        match self.content.clone() {
            Some(content) => serde_json::from_value(content),
            None => Ok(None),
        }
    }

    pub fn as_execute_result_content(&self) -> Result<Option<ExecuteResultContent>, JsonError> {
        match self.content.clone() {
            Some(content) => serde_json::from_value(content),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    pub username: String,
    pub session: String,
    pub msg_type: MessageType,
    pub version: String,
    pub msg_id: String,
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParentHeader {
    pub username: String,
    pub session: String,
    pub msg_type: MessageType,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    //TODO(tacogips) impl
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecuteInputContent {
    pub status: String,
    pub execution_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecuteReplyContent {
    pub status: String,
    pub execution_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecuteResultContent {
    pub execution_count: i64,
    pub data: Data,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusContent {
    pub execution_state: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorContent {
    pub ename: String,
    pub evalue: String,
    pub traceback: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Content {
    ExecuteResultContent(ExecuteResultContent),
    StatusContent(StatusContent),
    ErrorContent(ErrorContent),
    ExecuteReplyContent(ExecuteReplyContent),
    ExecuteInputContent(ExecuteInputContent),
}
