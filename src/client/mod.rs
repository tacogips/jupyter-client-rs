mod contents;
mod params;

use contents::*;
use params::*;
use reqwest::{header, Client};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ThisError>;

#[derive(Error, Debug)]
pub enum ThisError {
    #[error("the error {0}")]
    Example(String),

    #[error("{0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal server error: {0}")]
    InternalServerError(String),
}

pub enum Credential {
    Token(String),
}
pub struct JupyterClient {
    base_url: String,
    credential: Option<Credential>,
    req_client: Client,
}

macro_rules! with_auth_header {
    ($credential:expr, $request_builder:expr) => {{
        match $credential.as_ref() {
            Some(credential) => match credential {
                Credential::Token(token) => {
                    $request_builder.header(header::AUTHORIZATION, format!("token {token}"))
                }
            },
            None => $request_builder,
        }
    }};
}

impl JupyterClient {
    /// GET /api/contents/{path}
    pub async fn get_contents(
        &self,
        path: &str,
        content_type: Option<ContentType>,
    ) -> Result<Option<Contents>> {
        let mut request_builder = with_auth_header! {
            self.credential,
            self.req_client.get(format!(
                "{base_url}/api/contents/{path}",
                base_url = self.base_url
            ))
        };

        if let Some(content_type) = content_type {
            request_builder = request_builder.query(&[("type", content_type.as_str())]);
        }
        let resp = convert_error(request_builder.send().await?).await?;
        match resp {
            Some(found) => Ok(Some(found.json().await?)),
            None => Ok(None),
        }
    }
}

pub async fn convert_error(response: reqwest::Response) -> Result<Option<reqwest::Response>> {
    if response.status().is_success() {
        Ok(Some(response))
    } else {
        let status = response.status().into();
        match status {
            404 => {
                log::debug!("keycloak returned 404 error");
                Ok(None)
            }
            400 => {
                let text = response.text().await?;
                Err(ThisError::BadRequest(text))
            }
            _ => {
                let text = response.text().await?;
                Err(ThisError::InternalServerError(format!("{status}:{text}")))
            }
        }
    }
}
