mod types;
use super::error::JupyterApiError;
use futures::sink::SinkExt;
use futures_util::StreamExt;
use serde::Serialize;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use types::*;
use url;

type Result<T> = std::result::Result<T, JupyterApiError>;

pub struct KernelApiClient {
    url: String,
}

const DEFAULT_TIMEOUT_SEC: u64 = 120;

pub fn find_request_result(message: KernelResponse) -> Option<KernelResponse> {
    match message.msg_type {
        MessageType::ExecuteReply | MessageType::ExecuteResult | MessageType::Error => {
            Some(message)
        }
        _ => None,
    }
}

impl KernelApiClient {
    pub fn new(url_without_protocol: &str, kernel_id: &str, secure: bool) -> Self {
        let protocol = if secure { "wss" } else { "ws" };
        let url = format!("{protocol}://{url_without_protocol}/api/kernels/{kernel_id}/channels");
        Self { url }
    }

    pub async fn run_code(
        &self,
        request: KernelCodeRequest,
        timeout: Option<Duration>,
    ) -> Result<KernelResponse> {
        self.run_and_wait_message(request, find_request_result, timeout)
            .await
    }

    pub async fn run_and_wait_message<FoundFn, Req: Serialize, T>(
        &self,
        request: Req,
        found_fn: FoundFn,
        timeout: Option<Duration>,
    ) -> Result<T>
    where
        FoundFn: Fn(KernelResponse) -> Option<T>,
    {
        let parsed_url = url::Url::parse(&self.url)?;
        let (connection, _resp) = connect_async(&parsed_url).await?;
        let (mut writer, mut reader) = connection.split();

        writer
            .send(Message::Ping("ping".as_bytes().to_vec()))
            .await?;

        writer
            .send(Message::Text(serde_json::to_string(&request)?))
            .await?;
        let timeout = sleep(timeout.unwrap_or(Duration::from_secs(DEFAULT_TIMEOUT_SEC)));
        tokio::pin!(timeout);
        loop {
            tokio::select! {
                 _ = &mut timeout =>{
                    log::debug!("timeout.");
                    return Err(JupyterApiError::KernelMessageTimeout)
                 },

                 Some(receipt_message) = reader.next() =>{
                    log::debug!("receipt_message: {receipt_message:?}");
                    match receipt_message{
                        Ok(message) =>{
                            let resp: KernelResponse = match message{
                                Message::Pong(_body) =>{continue},
                                Message::Text(message) =>{serde_json::from_str(message.as_str())?},
                                Message::Close(_) => return Err(JupyterApiError::KernelConnectionClosed),
                                Message::Ping(body) =>{
                                    writer.send(Message::Pong(body)).await.ok();
                                    continue
                                },
                                _ => {
                                    continue
                                }
                            };

                            if let Some(found) = found_fn(resp){
                                return Ok(found)
                            }
                        },
                        Err(e) =>{
                          return Err(JupyterApiError::KernelMessageError(format!("{e}")))
                        }
                    }
                 },
            }
        }
    }
}
