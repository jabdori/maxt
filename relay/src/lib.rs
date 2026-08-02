use std::{collections::HashSet, env, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{
        DefaultBodyLimit, State, WebSocketUpgrade,
        ws::{Message as BrowserMessage, WebSocket},
    },
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use reqwest::redirect::Policy;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    time::timeout,
};
use tokio_tungstenite::{
    connect_async_with_config,
    tungstenite::{
        Message as UpstreamMessage, client::IntoClientRequest, protocol::WebSocketConfig,
    },
};
use tower_http::cors::CorsLayer;
use url::{Origin, Url};

const ALLOWED_HEADERS: [&str; 3] = ["authorization", "x-mbx-apikey", "content-type"];

#[derive(Clone)]
pub struct Config {
    browser_origins: Vec<HeaderValue>,
    http_origins: HashSet<Origin>,
    ws_origins: HashSet<Origin>,
    max_request_bytes: usize,
    max_frame_bytes: usize,
    handshake_timeout: Duration,
    upstream_timeout: Duration,
    max_connections: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            browser_origins: browser_origins(required("RELAY_ALLOWED_ORIGINS")?)?,
            http_origins: upstream_origins(required("RELAY_ALLOWED_HTTP_ORIGINS")?, "https")?,
            ws_origins: upstream_origins(required("RELAY_ALLOWED_WS_ORIGINS")?, "wss")?,
            max_request_bytes: positive("RELAY_MAX_REQUEST_BYTES", 1_048_576)?,
            max_frame_bytes: positive("RELAY_MAX_FRAME_BYTES", 1_048_576)?,
            handshake_timeout: Duration::from_millis(
                positive("RELAY_HANDSHAKE_TIMEOUT_MS", 10_000)? as u64,
            ),
            upstream_timeout: Duration::from_millis(
                positive("RELAY_UPSTREAM_TIMEOUT_MS", 30_000)? as u64
            ),
            max_connections: positive("RELAY_MAX_CONNECTIONS", 100)?,
        })
    }
}

#[derive(Clone)]
struct AppState {
    browser_origins: Arc<HashSet<HeaderValue>>,
    http_origins: Arc<HashSet<Origin>>,
    ws_origins: Arc<HashSet<Origin>>,
    client: reqwest::Client,
    max_body: usize,
    max_frame: usize,
    handshake_timeout: Duration,
    upstream_timeout: Duration,
    slots: Arc<Semaphore>,
}

pub fn app(config: Config) -> Router {
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .timeout(config.upstream_timeout)
        .build()
        .expect("reqwest client configuration is valid");
    let cors = CorsLayer::new()
        .allow_origin(config.browser_origins.clone())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);
    let state = AppState {
        browser_origins: Arc::new(config.browser_origins.iter().cloned().collect()),
        http_origins: Arc::new(config.http_origins),
        ws_origins: Arc::new(config.ws_origins),
        client,
        max_body: config.max_request_bytes,
        max_frame: config.max_frame_bytes,
        handshake_timeout: config.handshake_timeout,
        upstream_timeout: config.upstream_timeout,
        slots: Arc::new(Semaphore::new(config.max_connections)),
    };

    Router::new()
        .route("/healthz", get(|| async { StatusCode::NO_CONTENT }))
        .route("/v1/http", post(http_relay))
        .route("/v1/ws", get(ws_relay))
        .layer(DefaultBodyLimit::max(config.max_request_bytes))
        .layer(cors)
        .with_state(state)
}

#[derive(Deserialize)]
struct HttpRequest {
    url: String,
    method: String,
    #[serde(default)]
    headers: Vec<(String, String)>,
    body: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct HttpResponse {
    status: u16,
    body: String,
}

async fn http_relay(
    State(state): State<AppState>,
    request_headers: HeaderMap,
    Json(input): Json<HttpRequest>,
) -> Result<Json<HttpResponse>, ApiError> {
    require_browser_origin(&request_headers, &state.browser_origins)?;
    let url = allowed_target(&input.url, &state.http_origins)?;
    let method = match input.method.as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        _ => return Err(ApiError::bad_request("unsupported method")),
    };
    let headers = relay_headers(input.headers)?;
    let _slot = slot(&state)?;
    let mut request = state.client.request(method, url).headers(headers);
    if let Some(body) = input.body {
        request = request.body(body);
    }
    let response = request
        .send()
        .await
        .map_err(|_| ApiError::bad_gateway("upstream request failed"))?;
    let status = response.status().as_u16();
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| ApiError::bad_gateway("upstream body failed"))?;
        if bytes.len().saturating_add(chunk.len()) > state.max_body {
            return Err(ApiError::bad_gateway("upstream body too large"));
        }
        bytes.extend_from_slice(&chunk);
    }
    let body = String::from_utf8(bytes)
        .map_err(|_| ApiError::bad_gateway("upstream body is not UTF-8"))?;
    Ok(Json(HttpResponse { status, body }))
}

#[derive(Deserialize)]
struct WsInit {
    url: String,
    #[serde(default)]
    headers: Vec<(String, String)>,
    #[serde(default)]
    subscribe: Vec<String>,
}

async fn ws_relay(
    State(state): State<AppState>,
    request_headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    require_browser_origin(&request_headers, &state.browser_origins)?;
    let permit = slot(&state)?;
    let max_frame = state.max_frame;
    Ok(upgrade
        .max_message_size(max_frame)
        .max_frame_size(max_frame)
        .on_upgrade(move |socket| socket_session(socket, state, permit))
        .into_response())
}

async fn socket_session(mut browser: WebSocket, state: AppState, _permit: OwnedSemaphorePermit) {
    let Some(Ok(BrowserMessage::Text(init))) = timeout(state.handshake_timeout, browser.recv())
        .await
        .ok()
        .flatten()
    else {
        ws_fail(&mut browser, "expected text init frame").await;
        return;
    };
    let Ok(init) = serde_json::from_str::<WsInit>(&init) else {
        ws_fail(&mut browser, "invalid init frame").await;
        return;
    };
    let Ok(url) = allowed_target(&init.url, &state.ws_origins) else {
        ws_fail(&mut browser, "upstream origin is not allowed").await;
        return;
    };
    let Ok(headers) = relay_headers(init.headers) else {
        ws_fail(&mut browser, "upstream header is not allowed").await;
        return;
    };
    let Ok(mut request) = url.as_str().into_client_request() else {
        ws_fail(&mut browser, "invalid upstream URL").await;
        return;
    };
    for (name, value) in &headers {
        request.headers_mut().append(name, value.clone());
    }
    let ws_config = WebSocketConfig::default()
        .max_message_size(Some(state.max_frame))
        .max_frame_size(Some(state.max_frame));
    let Ok(Ok((mut upstream, _))) = timeout(
        state.upstream_timeout,
        connect_async_with_config(request, Some(ws_config), false),
    )
    .await
    else {
        ws_fail(&mut browser, "upstream connection failed").await;
        return;
    };
    for frame in init.subscribe {
        if !matches!(
            timeout(
                state.upstream_timeout,
                upstream.send(UpstreamMessage::Text(frame.into()))
            )
            .await,
            Ok(Ok(()))
        ) {
            ws_fail(&mut browser, "upstream subscription failed").await;
            return;
        }
    }
    if browser
        .send(BrowserMessage::Text(r#"{"type":"ready"}"#.into()))
        .await
        .is_err()
    {
        return;
    }
    bridge(browser, upstream, state.upstream_timeout).await;
}

async fn ws_fail(browser: &mut WebSocket, detail: &'static str) {
    let frame = serde_json::json!({ "type": "error", "detail": detail }).to_string();
    let _ = browser.send(BrowserMessage::Text(frame.into())).await;
    let _ = browser.send(BrowserMessage::Close(None)).await;
}

async fn bridge<S>(
    browser: WebSocket,
    upstream: tokio_tungstenite::WebSocketStream<S>,
    close_timeout: Duration,
) where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let (mut browser_tx, mut browser_rx) = browser.split();
    let (mut upstream_tx, mut upstream_rx) = upstream.split();
    loop {
        tokio::select! {
            message = browser_rx.next() => match message {
                Some(Ok(BrowserMessage::Text(text))) => if !matches!(timeout(close_timeout, upstream_tx.send(UpstreamMessage::Text(text.to_string().into()))).await, Ok(Ok(()))) { break; },
                Some(Ok(BrowserMessage::Binary(data))) => if !matches!(timeout(close_timeout, upstream_tx.send(UpstreamMessage::Binary(data))).await, Ok(Ok(()))) { break; },
                Some(Ok(BrowserMessage::Close(frame))) => {
                    let upstream_frame = frame.map(|f| tokio_tungstenite::tungstenite::protocol::CloseFrame { code: f.code.into(), reason: f.reason.to_string().into() });
                    let _ = browser_tx.flush().await;
                    let _ = upstream_tx.send(UpstreamMessage::Close(upstream_frame)).await;
                    let _ = timeout(close_timeout, async {
                        while let Some(Ok(message)) = upstream_rx.next().await {
                            if matches!(message, UpstreamMessage::Close(_)) {
                                let _ = upstream_tx.flush().await;
                                break;
                            }
                        }
                    }).await;
                    break;
                }
                Some(Ok(_)) => {}
                Some(Err(_)) | None => break,
            },
            message = upstream_rx.next() => match message {
                Some(Ok(UpstreamMessage::Text(text))) => if !matches!(timeout(close_timeout, browser_tx.send(BrowserMessage::Text(text.to_string().into()))).await, Ok(Ok(()))) { break; },
                Some(Ok(UpstreamMessage::Binary(data))) => if !matches!(timeout(close_timeout, browser_tx.send(BrowserMessage::Binary(data))).await, Ok(Ok(()))) { break; },
                Some(Ok(UpstreamMessage::Close(frame))) => {
                    let browser_frame = frame.map(|f| axum::extract::ws::CloseFrame { code: f.code.into(), reason: f.reason.to_string().into() });
                    let _ = upstream_tx.flush().await;
                    let _ = browser_tx.send(BrowserMessage::Close(browser_frame)).await;
                    let _ = timeout(close_timeout, browser_rx.next()).await;
                    let _ = browser_tx.flush().await;
                    break;
                }
                Some(Ok(_)) => {}
                Some(Err(_)) | None => break,
            }
        }
    }
}

fn required(name: &str) -> Result<String, String> {
    env::var(name).map_err(|_| format!("{name} is required"))
}

fn positive(name: &str, default: usize) -> Result<usize, String> {
    let value = env::var(name).map_or(Ok(default), |value| {
        value
            .parse()
            .map_err(|_| format!("{name} must be a positive integer"))
    })?;
    (value > 0)
        .then_some(value)
        .ok_or_else(|| format!("{name} must be a positive integer"))
}

fn browser_origins(value: String) -> Result<Vec<HeaderValue>, String> {
    nonempty(
        split(&value)
            .map(|raw| {
                let url = origin_url(raw, &["http", "https"])?;
                if url.as_str().trim_end_matches('/') != raw.trim_end_matches('/') {
                    return Err("RELAY_ALLOWED_ORIGINS entries must be exact origins".into());
                }
                raw.parse()
                    .map_err(|_| "invalid RELAY_ALLOWED_ORIGINS entry".into())
            })
            .collect(),
        "RELAY_ALLOWED_ORIGINS",
    )
}

fn upstream_origins(value: String, scheme: &str) -> Result<HashSet<Origin>, String> {
    let origins: HashSet<_> = split(&value)
        .map(|raw| origin_url(raw, &[scheme]).map(|url| url.origin()))
        .collect::<Result<_, _>>()?;
    if origins.is_empty() {
        return Err(format!(
            "{} must contain at least one origin",
            if scheme == "https" {
                "RELAY_ALLOWED_HTTP_ORIGINS"
            } else {
                "RELAY_ALLOWED_WS_ORIGINS"
            }
        ));
    }
    Ok(origins)
}

fn nonempty<T>(value: Result<Vec<T>, String>, name: &str) -> Result<Vec<T>, String> {
    let value = value?;
    (!value.is_empty())
        .then_some(value)
        .ok_or_else(|| format!("{name} must contain at least one origin"))
}

fn split(value: &str) -> impl Iterator<Item = &str> {
    value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
}

fn origin_url(raw: &str, schemes: &[&str]) -> Result<Url, String> {
    let url = Url::parse(raw).map_err(|_| "invalid origin URL".to_string())?;
    if !schemes.contains(&url.scheme())
        || !url.has_host()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("allowlist entries must be exact origins".into());
    }
    Ok(url)
}

fn allowed_target(raw: &str, allowed: &HashSet<Origin>) -> Result<Url, ApiError> {
    let url = Url::parse(raw).map_err(|_| ApiError::bad_request("invalid upstream URL"))?;
    if !url.has_host()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !allowed.contains(&url.origin())
    {
        return Err(ApiError::forbidden("upstream origin is not allowed"));
    }
    Ok(url)
}

fn relay_headers(headers: Vec<(String, String)>) -> Result<HeaderMap, ApiError> {
    let mut output = HeaderMap::new();
    for (name, value) in headers {
        let name =
            HeaderName::try_from(name).map_err(|_| ApiError::bad_request("invalid header name"))?;
        if !ALLOWED_HEADERS
            .iter()
            .any(|allowed| name.as_str() == *allowed)
        {
            return Err(ApiError::forbidden("header is not allowed"));
        }
        let value = HeaderValue::try_from(value)
            .map_err(|_| ApiError::bad_request("invalid header value"))?;
        output.append(name, value);
    }
    Ok(output)
}

fn require_browser_origin(
    headers: &HeaderMap,
    allowed: &HashSet<HeaderValue>,
) -> Result<(), ApiError> {
    match headers.get(header::ORIGIN) {
        Some(origin) if allowed.contains(origin) => Ok(()),
        _ => Err(ApiError::forbidden("browser origin is not allowed")),
    }
}

fn slot(state: &AppState) -> Result<OwnedSemaphorePermit, ApiError> {
    state
        .slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| ApiError(StatusCode::SERVICE_UNAVAILABLE, "relay capacity reached"))
}

struct ApiError(StatusCode, &'static str);

impl ApiError {
    fn bad_request(message: &'static str) -> Self {
        Self(StatusCode::BAD_REQUEST, message)
    }
    fn forbidden(message: &'static str) -> Self {
        Self(StatusCode::FORBIDDEN, message)
    }
    fn bad_gateway(message: &'static str) -> Self {
        Self(StatusCode::BAD_GATEWAY, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({ "error": self.1 }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, extract::ws::WebSocketUpgrade, http::Request, response::Redirect};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite::{Error as WsError, Message};
    use tower::ServiceExt;

    fn test_config(http: &str, ws: &str) -> Config {
        Config {
            browser_origins: vec!["https://app.example".parse().unwrap()],
            http_origins: [Url::parse(http).unwrap().origin()].into(),
            ws_origins: [Url::parse(ws).unwrap().origin()].into(),
            max_request_bytes: 4096,
            max_frame_bytes: 4096,
            handshake_timeout: Duration::from_secs(2),
            upstream_timeout: Duration::from_secs(2),
            max_connections: 4,
        }
    }

    async fn listen(app: Router) -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        address
    }

    #[test]
    fn rejects_ssrf_and_unlisted_headers() {
        let allowed = [Url::parse("https://api.example").unwrap().origin()].into();
        assert!(allowed_target("http://127.0.0.1/secret", &allowed).is_err());
        assert!(allowed_target("https://api.example.evil/", &allowed).is_err());
        assert!(allowed_target("https://user@api.example/", &allowed).is_err());
        assert!(relay_headers(vec![("cookie".into(), "secret".into())]).is_err());
        assert!(relay_headers(vec![("authorization".into(), "Bearer value".into())]).is_ok());
    }

    #[tokio::test]
    async fn forwards_http_without_following_redirects_and_checks_origin() {
        let calls = Arc::new(AtomicUsize::new(0));
        let seen = calls.clone();
        let upstream = Router::new()
            .route(
                "/echo",
                post(move |headers: HeaderMap, body: String| {
                    let seen = seen.clone();
                    async move {
                        seen.fetch_add(1, Ordering::SeqCst);
                        assert_eq!(headers[header::AUTHORIZATION], "Bearer value");
                        (StatusCode::CREATED, body)
                    }
                }),
            )
            .route("/redirect", post(|| async { Redirect::temporary("/echo") }));
        let upstream = listen(upstream).await;
        let relay = app(test_config(
            &format!("http://{upstream}"),
            "ws://127.0.0.1:9",
        ));

        let request = |origin: &str, path: &str| {
            Request::builder()
                .method("POST")
                .uri("/v1/http")
                .header(header::ORIGIN, origin)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "url": format!("http://{upstream}/{path}"), "method": "POST",
                        "headers": [["authorization", "Bearer value"]], "body": "hello"
                    })
                    .to_string(),
                ))
                .unwrap()
        };

        let denied = relay
            .clone()
            .oneshot(request("https://evil.example", "echo"))
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
        assert_eq!(calls.load(Ordering::SeqCst), 0);

        let response = relay
            .clone()
            .oneshot(request("https://app.example", "echo"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let response: HttpResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!((response.status, response.body.as_str()), (201, "hello"));

        let redirect = relay
            .oneshot(request("https://app.example", "redirect"))
            .await
            .unwrap();
        let bytes = axum::body::to_bytes(redirect.into_body(), 4096)
            .await
            .unwrap();
        let response: HttpResponse = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(response.status, 307);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn forwards_websocket_subscriptions_and_frames() {
        async fn upstream(upgrade: WebSocketUpgrade, headers: HeaderMap) -> Response {
            assert_eq!(headers["x-mbx-apikey"], "key");
            upgrade
                .on_upgrade(|mut socket| async move {
                    assert_eq!(
                        socket.recv().await.unwrap().unwrap(),
                        BrowserMessage::Text("one".into())
                    );
                    assert_eq!(
                        socket.recv().await.unwrap().unwrap(),
                        BrowserMessage::Text("two".into())
                    );
                    socket
                        .send(BrowserMessage::Text("subscribed".into()))
                        .await
                        .unwrap();
                    while let Some(Ok(message)) = socket.recv().await {
                        let close = matches!(message, BrowserMessage::Close(_));
                        if close {
                            break;
                        }
                        socket.send(message).await.unwrap();
                    }
                })
                .into_response()
        }
        let upstream = listen(Router::new().route("/socket", get(upstream))).await;
        let relay = listen(app(test_config(
            "http://127.0.0.1:9",
            &format!("ws://{upstream}"),
        )))
        .await;
        let mut request = format!("ws://{relay}/v1/ws").into_client_request().unwrap();
        request
            .headers_mut()
            .insert(header::ORIGIN, "https://app.example".parse().unwrap());
        let (mut socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();
        socket
            .send(Message::Text(
                serde_json::json!({
                    "url": format!("ws://{upstream}/socket"),
                    "headers": [["x-mbx-apikey", "key"]], "subscribe": ["one", "two"]
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();
        assert_eq!(
            socket.next().await.unwrap().unwrap(),
            Message::Text(r#"{"type":"ready"}"#.into())
        );
        assert_eq!(
            socket.next().await.unwrap().unwrap(),
            Message::Text("subscribed".into())
        );
        socket.send(Message::Text("hello".into())).await.unwrap();
        assert_eq!(
            socket.next().await.unwrap().unwrap(),
            Message::Text("hello".into())
        );
        socket
            .send(Message::Binary(vec![1, 2, 3].into()))
            .await
            .unwrap();
        assert_eq!(
            socket.next().await.unwrap().unwrap(),
            Message::Binary(vec![1, 2, 3].into())
        );
        socket.send(Message::Close(None)).await.unwrap();
        assert!(matches!(
            socket.next().await.unwrap().unwrap(),
            Message::Close(_)
        ));

        let mut denied_request = format!("ws://{relay}/v1/ws").into_client_request().unwrap();
        denied_request
            .headers_mut()
            .insert(header::ORIGIN, "https://app.example".parse().unwrap());
        let (mut denied, _) = tokio_tungstenite::connect_async(denied_request)
            .await
            .unwrap();
        denied
            .send(Message::Text(
                r#"{"url":"wss://evil.example/socket","headers":[],"subscribe":[]}"#.into(),
            ))
            .await
            .unwrap();
        assert_eq!(
            denied.next().await.unwrap().unwrap(),
            Message::Text(r#"{"detail":"upstream origin is not allowed","type":"error"}"#.into())
        );
    }

    #[tokio::test]
    async fn stalled_peer_releases_connection_capacity() {
        async fn flood(upgrade: WebSocketUpgrade) -> Response {
            upgrade
                .on_upgrade(|mut socket| async move {
                    let frame = BrowserMessage::Binary(vec![0; 1_048_576].into());
                    while socket.send(frame.clone()).await.is_ok() {}
                })
                .into_response()
        }

        let upstream = listen(Router::new().route("/flood", get(flood))).await;
        let mut config = test_config("http://127.0.0.1:9", &format!("ws://{upstream}"));
        config.max_frame_bytes = 1_048_576;
        config.upstream_timeout = Duration::from_millis(50);
        config.max_connections = 1;
        let relay = listen(app(config)).await;

        let request = || {
            let mut request = format!("ws://{relay}/v1/ws").into_client_request().unwrap();
            request
                .headers_mut()
                .insert(header::ORIGIN, "https://app.example".parse().unwrap());
            request
        };
        let stream = tokio::net::TcpStream::connect(relay).await.unwrap();
        socket2::SockRef::from(&stream)
            .set_recv_buffer_size(1024)
            .unwrap();
        let (mut stalled, _) = tokio_tungstenite::client_async(request(), stream)
            .await
            .unwrap();

        let error = tokio_tungstenite::connect_async(request())
            .await
            .unwrap_err();
        assert!(
            matches!(error, WsError::Http(response) if response.status() == StatusCode::SERVICE_UNAVAILABLE)
        );

        stalled
            .send(Message::Text(
                serde_json::json!({
                    "url": format!("ws://{upstream}/flood"),
                    "headers": [], "subscribe": []
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();

        let replacement = timeout(Duration::from_secs(3), async {
            loop {
                match tokio_tungstenite::connect_async(request()).await {
                    Ok((socket, _)) => break socket,
                    Err(WsError::Http(response))
                        if response.status() == StatusCode::SERVICE_UNAVAILABLE =>
                    {
                        tokio::time::sleep(Duration::from_millis(20)).await;
                    }
                    Err(error) => panic!("replacement connection failed: {error}"),
                }
            }
        })
        .await
        .expect("stalled session did not return its permit");
        drop(replacement);
    }
}
