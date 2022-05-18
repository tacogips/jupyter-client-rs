mod error;
pub mod kernel;
pub mod types;

use error::*;

use reqwest::{header, Client};
use types::*;

type Result<T> = std::result::Result<T, JupyterApiError>;
pub enum Credential {
    Token(String),
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

pub struct JupyterClient {
    pub secure: bool,
    pub base_host: String,
    credential: Option<Credential>,
    req_client: Client,
}

impl Default for JupyterClient {
    fn default() -> JupyterClient {
        JupyterClient {
            secure: false,
            base_host: "localhost:8888".to_string(),
            credential: None,
            req_client: Client::new(),
        }
    }
}
impl JupyterClient {
    fn base_url(&self) -> String {
        let protocol = if self.secure { "https" } else { "http" };
        format!("{}://{}", protocol, self.base_host)
    }
    /// GET /api/contents
    pub async fn get_root_contents(&self) -> Result<Option<ContentList>> {
        let request_builder = with_auth_header! {
            self.credential,
            self.req_client.get(
                format!(
                "{base_url}/api/contents",
                base_url = self.base_url()
            ))
        };

        let resp = convert_error(request_builder.send().await?).await?;
        match resp {
            Some(found) => Ok(Some(found.json().await?)),
            None => Ok(None),
        }
    }

    /// GET /api/contents/{path}
    pub async fn get_contents(
        &self,
        path: &str,
        content_type: Option<ContentType>,
    ) -> Result<Option<Content>> {
        let mut request_builder = with_auth_header! {
            self.credential,
            self.req_client.get(
                format!(
                "{base_url}/api/contents/{path}",
                base_url = self.base_url()
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

    /// POST /api/contents/{path}
    pub async fn post_contents(&self, path: &str, content: Content) -> Result<Option<Content>> {
        let request_builder = with_auth_header! {
            self.credential,
            self.req_client.post(
                format!(
                "{base_url}/api/contents/{path}",
                base_url = self.base_url()
            ))
        }
        .json(&content);

        let resp = convert_error(request_builder.send().await?).await?;
        match resp {
            Some(found) => Ok(Some(found.json().await?)),
            None => Ok(None),
        }
    }

    /// PUT /api/contents/{path}
    pub async fn put_contents(
        &self,
        path: &str,
        content: ContentPutRequest,
    ) -> Result<Option<Content>> {
        let request_builder = with_auth_header! {
            self.credential,
            self.req_client.put(
                format!(
                "{base_url}/api/contents/{path}",
                base_url = self.base_url()
            ))
        }
        .json(&content);

        let resp = convert_error(request_builder.send().await?).await?;
        match resp {
            Some(found) => Ok(Some(found.json().await?)),
            None => Ok(None),
        }
    }

    /// POST /api/kernels
    pub async fn start_kernel(&self, request: KernelPostRequest) -> Result<()> {
        let request_builder = with_auth_header! {
            self.credential,
            self.req_client.post(format!(
                "{base_url}/api/kernels",
                base_url = self.base_url()
            ))
        }
        .json(&request);

        convert_error(request_builder.send().await?).await?;
        Ok(())
    }

    /// GET /api/kernels
    pub async fn get_kernels(&self) -> Result<Vec<Kernel>> {
        let request_builder = with_auth_header! {
            self.credential,
            self.req_client.get(format!(
                "{base_url}/api/kernels",
                base_url = self.base_url()
            ))
        };

        let resp = convert_error(request_builder.send().await?).await?;
        match resp {
            Some(found) => Ok(found.json().await?),
            None => Ok(vec![]),
        }
    }

    /// GET /api/sessions
    pub async fn get_sessions(&self) -> Result<Vec<Session>> {
        let request_builder = with_auth_header! {
            self.credential,
            self.req_client.get(format!(
                "{base_url}/api/sessions",
                base_url = self.base_url()
            ))
        };

        let resp = convert_error(request_builder.send().await?).await?;
        match resp {
            Some(found) => Ok(found.json().await?),
            None => Ok(vec![]),
        }
    }

    /// GET /api/sessions/{session_d}
    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        let request_builder = with_auth_header! {
            self.credential,
            self.req_client.get(format!(
                "{base_url}/api/sessions/{session_id}",
                base_url = self.base_url()
            ))
        };

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
                Err(JupyterApiError::BadRequest(text))
            }
            _ => {
                let text = response.text().await?;
                Err(JupyterApiError::InternalServerError(format!(
                    "{status}:{text}"
                )))
            }
        }
    }
}
