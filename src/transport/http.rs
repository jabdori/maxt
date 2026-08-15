//! Buffered HTTP transport for adapter REST requests.

use std::time::Duration;

#[cfg(target_arch = "wasm32")]
use serde::Deserialize;
#[cfg(any(test, target_arch = "wasm32"))]
use serde::Serialize;

use crate::error::{Error, Result};

/// HTTP methods used by provider adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl HttpMethod {
    fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        }
    }
}

/// Client-independent REST request description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HttpRequest {
    pub(crate) method: HttpMethod,
    /// Path only, starting with `/`. The transport supplies the host.
    pub(crate) path: String,
    /// Already-encoded query string, without the leading `?`.
    pub(crate) query: String,
    /// Request body, for methods that carry one.
    pub(crate) body: Option<String>,
    /// Headers to send, including any authentication.
    pub(crate) headers: Vec<(String, String)>,
}

impl HttpRequest {
    pub(crate) fn get(path: impl Into<String>) -> Self {
        Self::new(HttpMethod::Get, path)
    }

    pub(crate) fn post(path: impl Into<String>) -> Self {
        Self::new(HttpMethod::Post, path)
    }

    pub(crate) fn delete(path: impl Into<String>) -> Self {
        Self::new(HttpMethod::Delete, path)
    }

    pub(crate) fn new(method: HttpMethod, path: impl Into<String>) -> Self {
        Self {
            method,
            path: path.into(),
            query: String::new(),
            body: None,
            headers: Vec::new(),
        }
    }

    pub(crate) fn query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    pub(crate) fn json_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self.header("content-type", "application/json")
    }

    pub(crate) fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Path and query joined, as it appears on the request line.
    pub(crate) fn target(&self) -> String {
        if self.query.is_empty() {
            self.path.clone()
        } else {
            format!("{}?{}", self.path, self.query)
        }
    }
}

/// Buffered HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HttpResponse {
    pub(crate) status: u16,
    pub(crate) body: String,
}

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Serialize)]
struct RelayHttpRequest<'a> {
    url: &'a str,
    method: &'static str,
    headers: &'a [(String, String)],
    body: Option<&'a str>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
struct RelayHttpResponse {
    status: u16,
    body: String,
}

impl HttpResponse {
    pub(crate) fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// Sends REST requests to one base URL.
#[derive(Debug, Clone)]
pub(crate) struct HttpTransport {
    client: reqwest::Client,
    base_url: String,
}

impl HttpTransport {
    /// A transport pointed at one host, for example `https://api.upbit.com`.
    pub(crate) fn new(base_url: impl Into<String>) -> Result<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        crate::transport::ensure_crypto_provider();

        let builder = reqwest::Client::builder();
        #[cfg(not(target_arch = "wasm32"))]
        let builder = builder
            .timeout(Duration::from_secs(30))
            .user_agent(concat!("maxt/", env!("CARGO_PKG_VERSION")));
        let client = builder
            .build()
            .map_err(|err| Error::transport(err.to_string()))?;

        Ok(Self {
            client,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        })
    }

    /// Sends a request and buffers the response body.
    ///
    /// Non-2xx statuses remain [`HttpResponse`] values for provider-specific
    /// error decoding by the adapter.
    pub(crate) async fn send(&self, request: &HttpRequest) -> Result<HttpResponse> {
        #[cfg(target_arch = "wasm32")]
        // ponytail: browser Fetch is thread-affine; split the async contract if threaded wasm support is needed.
        return send_wrapper::SendWrapper::new(self.send_inner(request)).await;

        #[cfg(not(target_arch = "wasm32"))]
        self.send_inner(request).await
    }

    async fn send_inner(&self, request: &HttpRequest) -> Result<HttpResponse> {
        let url = format!("{}{}", self.base_url, request.target());

        #[cfg(target_arch = "wasm32")]
        if let Some(relay) = crate::transport::browser_relay() {
            return self.send_via_relay(relay, request, &url).await;
        }
        #[cfg(target_arch = "wasm32")]
        if requires_relay(request) {
            return Err(Error::invalid_request(
                "relay_url",
                "browser requests with custom authentication headers require configure_browser_relay",
            ));
        }

        self.send_direct(request, &url).await
    }

    async fn send_direct(&self, request: &HttpRequest, url: &str) -> Result<HttpResponse> {
        let mut builder = self.client.request(
            request
                .method
                .as_str()
                .parse()
                .map_err(|_| Error::transport("unsupported HTTP method"))?,
            url,
        );

        for (name, value) in &request.headers {
            builder = builder.header(name, value);
        }
        if let Some(body) = &request.body {
            builder = builder.body(body.clone());
        }
        #[cfg(target_arch = "wasm32")]
        let builder = builder.timeout(Duration::from_secs(30));

        let response = builder
            .send()
            .await
            .map_err(|err| Error::transport(err.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|err| Error::transport(err.to_string()))?;

        Ok(HttpResponse { status, body })
    }

    #[cfg(target_arch = "wasm32")]
    async fn send_via_relay(
        &self,
        relay: &crate::transport::BrowserRelay,
        request: &HttpRequest,
        url: &str,
    ) -> Result<HttpResponse> {
        let envelope = serde_json::to_string(&RelayHttpRequest {
            url,
            method: request.method.as_str(),
            headers: &request.headers,
            body: request.body.as_deref(),
        })
        .map_err(|error| Error::transport(format!("could not encode relay request: {error}")))?;
        let response = self
            .client
            .post(&relay.http)
            .header("content-type", "application/json")
            .body(envelope)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|error| Error::transport(format!("browser relay request failed: {error}")))?;
        let relay_status = response.status();
        let body = response.text().await.map_err(|error| {
            Error::transport(format!("could not read browser relay response: {error}"))
        })?;
        if !relay_status.is_success() {
            return Err(Error::transport(format!(
                "browser relay returned {relay_status}: {body}"
            )));
        }

        let response: RelayHttpResponse = serde_json::from_str(&body).map_err(|error| {
            Error::transport(format!(
                "browser relay returned an invalid response: {error}"
            ))
        })?;
        Ok(HttpResponse {
            status: response.status,
            body: response.body,
        })
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
fn requires_relay(request: &HttpRequest) -> bool {
    request
        .headers
        .iter()
        .any(|(name, _)| !name.eq_ignore_ascii_case("content-type"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_without_a_query_has_a_bare_path_target() {
        let request = HttpRequest::get("/v1/ticker");

        assert_eq!(request.target(), "/v1/ticker");
    }

    #[test]
    fn a_query_is_joined_with_a_single_question_mark() {
        let request = HttpRequest::get("/v1/ticker").query("markets=KRW-BTC");

        assert_eq!(request.target(), "/v1/ticker?markets=KRW-BTC");
    }

    #[test]
    fn a_json_body_sets_its_own_content_type() {
        let request = HttpRequest::post("/v1/orders").json_body(r#"{"side":"bid"}"#);

        assert_eq!(request.body.as_deref(), Some(r#"{"side":"bid"}"#));
        assert!(
            request
                .headers
                .iter()
                .any(|(name, value)| name == "content-type" && value == "application/json")
        );
    }

    #[test]
    fn success_is_the_2xx_range_and_nothing_else() {
        for status in [200, 201, 204, 299] {
            assert!(
                HttpResponse {
                    status,
                    body: String::new()
                }
                .is_success(),
                "{status}"
            );
        }
        for status in [199, 300, 400, 429, 500] {
            assert!(
                !HttpResponse {
                    status,
                    body: String::new()
                }
                .is_success(),
                "{status}"
            );
        }
    }

    #[test]
    fn a_trailing_slash_on_the_host_does_not_double_up() {
        let transport = HttpTransport::new("https://api.upbit.com/").unwrap();

        assert_eq!(transport.base_url, "https://api.upbit.com");
    }

    #[test]
    fn authentication_headers_require_a_browser_relay_but_content_type_does_not() {
        assert!(!requires_relay(&HttpRequest::post("/info").json_body("{}")));
        assert!(requires_relay(
            &HttpRequest::get("/account").header("authorization", "Bearer token")
        ));
        assert!(requires_relay(
            &HttpRequest::get("/account").header("X-MBX-APIKEY", "key")
        ));
    }

    #[test]
    fn relay_http_envelope_preserves_the_exact_upstream_request() {
        let request = HttpRequest::post("/orders")
            .header("authorization", "Bearer token")
            .json_body(r#"{"side":"buy"}"#);
        let encoded = serde_json::to_value(RelayHttpRequest {
            url: "https://exchange.example/orders",
            method: request.method.as_str(),
            headers: &request.headers,
            body: request.body.as_deref(),
        })
        .unwrap();

        assert_eq!(encoded["url"], "https://exchange.example/orders");
        assert_eq!(encoded["method"], "POST");
        assert_eq!(encoded["headers"][0][0], "authorization");
        assert_eq!(encoded["headers"][0][1], "Bearer token");
        assert_eq!(encoded["body"], r#"{"side":"buy"}"#);
    }
}
