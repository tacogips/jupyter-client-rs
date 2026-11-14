mod types;
use super::error::JupyterApiError;
use futures::sink::SinkExt;
use futures_util::StreamExt;
use serde::Serialize;
use std::pin::Pin;
use tokio::time::{sleep, Duration, Sleep};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
pub use types::*;
use url;

type Result<T> = std::result::Result<T, JupyterApiError>;

pub struct KernelApiClient {
    url: String,
}

const DEFAULT_TIMEOUT_SEC: u64 = 120;
const DEFAULT_WAIT_SUCCESSION_RESULT_MILLI_SEC: u64 = 100;

pub enum WaitResultResponse {
    KernelResponse(KernelResponse),
    WaitSuccession(Sleep),
}

pub trait WaitResult {
    fn check(&mut self, message: KernelResponse) -> Option<WaitResultResponse>;
    fn latest_result(self) -> (Option<KernelResponse>, Option<Vec<KernelResponse>>);
}

struct WaitResultAndDisplayData {
    inner_latest_result: Option<KernelResponse>,
    inner_stream_result: Vec<KernelResponse>,
}

impl Default for WaitResultAndDisplayData {
    fn default() -> Self {
        Self {
            inner_latest_result: None,
            inner_stream_result: vec![],
        }
    }
}
impl WaitResult for WaitResultAndDisplayData {
    fn check(&mut self, message: KernelResponse) -> Option<WaitResultResponse> {
        match message.msg_type {
            MessageType::ExecuteResult => {
                self.inner_latest_result = Some(message);
                Some(WaitResultResponse::WaitSuccession(sleep(
                    Duration::from_millis(DEFAULT_WAIT_SUCCESSION_RESULT_MILLI_SEC),
                )))
            }

            MessageType::Stream => {
                self.inner_stream_result.push(message);
                Some(WaitResultResponse::WaitSuccession(sleep(
                    Duration::from_millis(DEFAULT_WAIT_SUCCESSION_RESULT_MILLI_SEC),
                )))
            }

            MessageType::ExecuteReply => {
                if self.inner_latest_result.is_none() {
                    self.inner_latest_result = Some(message);
                }
                Some(WaitResultResponse::WaitSuccession(sleep(
                    Duration::from_millis(DEFAULT_WAIT_SUCCESSION_RESULT_MILLI_SEC),
                )))
            }

            MessageType::DisplayData | MessageType::Error => {
                Some(WaitResultResponse::KernelResponse(message))
            }
            _ => None,
        }
    }

    fn latest_result(self) -> (Option<KernelResponse>, Option<Vec<KernelResponse>>) {
        let (latest, latest_multi) = (
            self.inner_latest_result,
            if self.inner_stream_result.is_empty() {
                None
            } else {
                Some(self.inner_stream_result)
            },
        );

        match (latest, latest_multi) {
            (None, Some(mut multi)) if multi.len() == 1 => (Some(multi.remove(0)), None),
            (Some(latest), Some(mut multi))
                if multi.len() == 1 && latest.msg_type == MessageType::ExecuteReply =>
            {
                (Some(multi.remove(0)), None)
            }
            (l, lm) => (l, lm),
        }
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
    ) -> Result<CompositeKernelResponses> {
        self.run_and_wait_message(request, WaitResultAndDisplayData::default(), timeout)
            .await
    }

    pub async fn run_and_wait_message<F, Req: Serialize>(
        &self,
        request: Req,
        mut wait_result: F,
        timeout: Option<Duration>,
    ) -> Result<CompositeKernelResponses>
    where
        F: WaitResult,
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
        let mut wait_succession_timeout = sleep(Duration::from_secs(u64::MAX));
        tokio::pin!(timeout);

        let mut pinned_wait_succession_timeout =
            unsafe { Pin::new_unchecked(&mut wait_succession_timeout) };

        loop {
            tokio::select! {
                 _ = &mut timeout =>{
                    log::debug!("timeout.");
                    return Err(JupyterApiError::KernelMessageTimeout)
                 },

                 _ = &mut pinned_wait_succession_timeout  =>{
                    match wait_result.latest_result() {
                        (None ,None)=> return Err(JupyterApiError::KernelResponseTimeout),
                        (Some(latest_result) ,None)=>{
                            return Ok(CompositeKernelResponses::SingleResponse(latest_result))
                        }
                        (None ,Some(stream))=>{
                            return Ok(CompositeKernelResponses::MultipleResponse(stream))
                        }

                        (Some(latest_result) ,Some(mut stream))=>{
                            stream.push(latest_result);
                            return Ok(CompositeKernelResponses::MultipleResponse(stream))
                        }
                    }
                 },

                 Some(receipt_message) = reader.next() =>{
                    log::debug!("receipt_message: {receipt_message:?}");
                    match receipt_message{
                        Ok(message) =>{
                            let resp: KernelResponse = match message{
                                Message::Pong(_body) =>{continue},
                                Message::Text(message) =>{
                                    serde_json::from_str(message.as_str())?
                                },
                                Message::Close(_) => return Err(JupyterApiError::KernelConnectionClosed),
                                Message::Ping(body) =>{
                                    writer.send(Message::Pong(body)).await.ok();
                                    continue
                                },
                                _ => {
                                    continue
                                }
                            };

                            if let Some(checked_result) = wait_result.check(resp){
                                match checked_result{
                                     WaitResultResponse::KernelResponse(response)=>return Ok(CompositeKernelResponses::SingleResponse(response)),
                                     WaitResultResponse::WaitSuccession(succession_timeout)=>{
                                        wait_succession_timeout = succession_timeout;
                                        pinned_wait_succession_timeout =
                                            unsafe { Pin::new_unchecked(&mut wait_succession_timeout) };
                                     }
                                }

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
