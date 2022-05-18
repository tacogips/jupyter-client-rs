use serde_json::error::Error as JsonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JupyterApiError {
    #[error("the error {0}")]
    Example(String),

    #[error("{0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("invalid message type: {0}")]
    InvalieMessageType(String),

    #[error("json error: {0}")]
    JsonError(#[from] JsonError),

    #[error("ws error: {0}")]
    WsError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("kernel message timeout ")]
    KernelMessageTimeout,

    #[error("kernel message error: {0}")]
    KernelMessageError(String),

    #[error("connection closed by kernel")]
    KernelConnectionClosed,

    #[error("internal server error: {0}")]
    InternalServerError(String),
}
