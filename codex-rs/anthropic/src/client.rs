//! Streaming client for the Anthropic Messages API.

use crate::ANTHROPIC_BETA_HEADER_VALUE;
use crate::ANTHROPIC_VERSION_HEADER_VALUE;
use crate::error::AnthropicApiError;
use crate::error::AnthropicError;
use crate::error::Result;
use crate::stream::AnthropicEvent;
use crate::stream::parse_sse_event;
use crate::types::MessagesRequest;
use eventsource_stream::Eventsource;
use futures::Stream;
use futures::StreamExt;
use http::HeaderMap;
use http::HeaderValue;
use http::Method;
use orbit_code_client::HttpTransport;
use orbit_code_client::Request;
use orbit_code_client::ReqwestTransport;
use orbit_code_client::TransportError;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;

const STREAM_CHANNEL_CAPACITY: usize = 128;

pub struct AnthropicClient {
    transport: Arc<dyn HttpTransport>,
    base_url: String,
    idle_timeout: Duration,
}

impl AnthropicClient {
    pub fn new(client: reqwest::Client, base_url: String, idle_timeout: Duration) -> Self {
        Self {
            transport: Arc::new(ReqwestTransport::new(client)),
            base_url,
            idle_timeout,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_transport(
        transport: Arc<dyn HttpTransport>,
        base_url: String,
        idle_timeout: Duration,
    ) -> Self {
        Self {
            transport,
            base_url,
            idle_timeout,
        }
    }

    pub async fn stream(
        &self,
        request: MessagesRequest,
        api_key: String,
        mut extra_headers: HeaderMap,
    ) -> Result<AnthropicStream> {
        extra_headers.insert("x-api-key", header_value(&api_key)?);
        extra_headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        extra_headers
            .entry("anthropic-version")
            .or_insert(HeaderValue::from_static(ANTHROPIC_VERSION_HEADER_VALUE));
        extra_headers
            .entry("anthropic-beta")
            .or_insert(HeaderValue::from_static(ANTHROPIC_BETA_HEADER_VALUE));

        let mut transport_request = Request::new(
            Method::POST,
            format!("{}/v1/messages", self.base_url.trim_end_matches('/')),
        )
        .with_json(&request);
        transport_request.headers = extra_headers;
        transport_request.timeout = Some(self.idle_timeout);

        let response = self
            .transport
            .stream(transport_request)
            .await
            .map_err(map_transport_error)?;

        let mut events = response.bytes.eventsource();
        let idle_timeout = self.idle_timeout;
        let (tx, rx) = mpsc::channel(STREAM_CHANNEL_CAPACITY);

        tokio::spawn(async move {
            loop {
                match timeout(idle_timeout, events.next()).await {
                    Ok(Some(Ok(event))) => {
                        match parse_sse_event(event.event.as_str(), event.data.as_str()) {
                            Ok(Some(parsed)) => {
                                let should_close = matches!(
                                    parsed,
                                    AnthropicEvent::MessageStop | AnthropicEvent::Error { .. }
                                );
                                if tx.send(Ok(parsed)).await.is_err() {
                                    return;
                                }
                                if should_close {
                                    return;
                                }
                            }
                            Ok(None) => {}
                            Err(err) => {
                                let _ = tx.send(Err(err)).await;
                                return;
                            }
                        }
                    }
                    Ok(Some(Err(err))) => {
                        let _ = tx
                            .send(Err(AnthropicError::StreamParse(err.to_string())))
                            .await;
                        return;
                    }
                    Ok(None) => {
                        let _ = tx
                            .send(Err(AnthropicError::StreamParse(
                                "stream closed before message_stop".to_string(),
                            )))
                            .await;
                        return;
                    }
                    Err(_) => {
                        let _ = tx
                            .send(Err(AnthropicError::Transport(TransportError::Timeout)))
                            .await;
                        return;
                    }
                }
            }
        });

        Ok(AnthropicStream { rx })
    }
}

fn header_value(value: &str) -> Result<HeaderValue> {
    HeaderValue::from_str(value)
        .map_err(|err| AnthropicError::StreamParse(format!("invalid header value: {err}")))
}

fn map_transport_error(error: TransportError) -> AnthropicError {
    match error {
        TransportError::Http {
            status,
            body,
            url: _,
            headers: _,
        } if status.as_u16() == 429 => AnthropicError::RateLimited,
        TransportError::Http {
            status,
            body,
            url: _,
            headers: _,
        } if status.as_u16() == 529 => AnthropicError::Overloaded,
        TransportError::Http {
            status,
            body,
            url: _,
            headers: _,
        } => parse_api_error(status.as_u16(), body),
        other => AnthropicError::Transport(other),
    }
}

fn parse_api_error(status: u16, body: Option<String>) -> AnthropicError {
    #[derive(serde::Deserialize)]
    struct ErrorEnvelope {
        error: ErrorBody,
    }

    #[derive(serde::Deserialize)]
    struct ErrorBody {
        r#type: String,
        message: String,
    }

    if let Some(body) = body {
        if let Ok(parsed) = serde_json::from_str::<ErrorEnvelope>(&body) {
            return AnthropicError::Api(Box::new(AnthropicApiError {
                status,
                error_type: parsed.error.r#type,
                message: parsed.error.message,
            }));
        }

        return AnthropicError::Api(Box::new(AnthropicApiError {
            status,
            error_type: "http_error".to_string(),
            message: body,
        }));
    }

    AnthropicError::Api(Box::new(AnthropicApiError {
        status,
        error_type: "http_error".to_string(),
        message: "request failed".to_string(),
    }))
}

pub struct AnthropicStream {
    rx: mpsc::Receiver<Result<AnthropicEvent>>,
}

impl Stream for AnthropicStream {
    type Item = Result<AnthropicEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::DeltaType;
    use crate::stream::Usage;
    use crate::types::Content;
    use crate::types::Message;
    use crate::types::Role;
    use async_trait::async_trait;
    use bytes::Bytes;
    use futures::TryStreamExt;
    use pretty_assertions::assert_eq;
    use std::sync::Mutex;

    fn test_request() -> MessagesRequest {
        MessagesRequest {
            model: "claude-sonnet-4-6".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: Content::Text("hello".to_string()),
            }],
            system: None,
            tools: None,
            tool_choice: None,
            thinking: None,
            max_tokens: 32_000,
            stream: true,
            temperature: None,
            top_p: None,
            top_k: None,
            metadata: None,
            output_config: None,
        }
    }

    #[derive(Clone)]
    struct TestTransport {
        requests: Arc<Mutex<Vec<Request>>>,
        stream_body: String,
    }

    #[async_trait]
    impl HttpTransport for TestTransport {
        async fn execute(
            &self,
            _req: Request,
        ) -> std::result::Result<orbit_code_client::Response, TransportError> {
            unreachable!("execute is not used in these tests")
        }

        async fn stream(
            &self,
            req: Request,
        ) -> std::result::Result<orbit_code_client::StreamResponse, TransportError> {
            self.requests.lock().expect("lock").push(req);
            let body = self.stream_body.clone();
            Ok(orbit_code_client::StreamResponse {
                status: http::StatusCode::OK,
                headers: HeaderMap::new(),
                bytes: Box::pin(futures::stream::iter(vec![Ok(Bytes::from(body))])),
            })
        }
    }

    #[tokio::test]
    async fn sends_api_key_and_omits_authorization_and_temperature() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let transport = Arc::new(TestTransport {
            requests: Arc::clone(&requests),
            stream_body: concat!(
                "event: message_start\n",
                "data: {\"message\":{\"id\":\"msg_1\",\"model\":\"claude-sonnet-4-6\",\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n\n",
                "event: message_stop\n",
                "data: {}\n\n"
            )
            .to_string(),
        });

        let client = AnthropicClient::with_transport(
            transport,
            "https://api.anthropic.com".to_string(),
            Duration::from_secs(5),
        );
        let mut stream = client
            .stream(test_request(), "sk-ant-test".to_string(), HeaderMap::new())
            .await
            .expect("stream");

        let first = stream
            .try_next()
            .await
            .expect("stream event")
            .expect("message start");
        assert_eq!(
            first,
            AnthropicEvent::MessageStart {
                message_id: "msg_1".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                usage: Usage {
                    input_tokens: 1,
                    output_tokens: 0,
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: None,
                },
            }
        );

        let request = requests
            .lock()
            .expect("lock")
            .first()
            .cloned()
            .expect("request");
        assert_eq!(
            request
                .headers
                .get("x-api-key")
                .and_then(|value| value.to_str().ok()),
            Some("sk-ant-test")
        );
        assert!(request.headers.get("authorization").is_none());

        let payload = request.body.expect("request payload");
        assert!(payload.get("temperature").is_none());
    }

    #[tokio::test]
    async fn propagates_text_deltas_from_stream() {
        let transport = Arc::new(TestTransport {
            requests: Arc::new(Mutex::new(Vec::new())),
            stream_body: concat!(
                "event: content_block_delta\n",
                "data: {\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"hello\"}}\n\n",
                "event: message_stop\n",
                "data: {}\n\n"
            )
            .to_string(),
        });

        let client = AnthropicClient::with_transport(
            transport,
            "https://api.anthropic.com".to_string(),
            Duration::from_secs(5),
        );
        let mut stream = client
            .stream(test_request(), "sk-ant-test".to_string(), HeaderMap::new())
            .await
            .expect("stream");

        let first = stream
            .try_next()
            .await
            .expect("stream event")
            .expect("delta");
        assert_eq!(
            first,
            AnthropicEvent::ContentBlockDelta {
                index: 0,
                delta: DeltaType::TextDelta {
                    text: "hello".to_string(),
                },
            }
        );
    }
}
