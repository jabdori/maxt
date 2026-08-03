# maxt relay

[English](README.md) | [한국어](README.ko.md)

A stateless HTTP/WebSocket relay for credentialed Browser WebAssembly calls.
Public browser calls can connect directly without it. When Browser WASM is
initialized with `relayUrl`, HTTP requests use the relay. WebSocket connections
use it only when an exchange requires handshake headers.

## Run

```sh
export RELAY_ALLOWED_ORIGINS=https://app.example
export RELAY_ALLOWED_HTTP_ORIGINS=https://api.binance.com,https://fapi.binance.com
export RELAY_ALLOWED_WS_ORIGINS=wss://stream.binance.com:9443,wss://fstream.binance.com
cargo run --manifest-path relay/Cargo.toml
```

Required variables:

- `RELAY_ALLOWED_ORIGINS`: comma-separated browser `http`/`https` origins.
- `RELAY_ALLOWED_HTTP_ORIGINS`: comma-separated upstream `https` origins.
- `RELAY_ALLOWED_WS_ORIGINS`: comma-separated upstream `wss` origins.

Origins must match exactly. Paths, queries, fragments, and credentials are not
allowlist entries.

Optional variables:

- `RELAY_BIND=0.0.0.0:8080`
- `RELAY_MAX_REQUEST_BYTES=1048576`
- `RELAY_MAX_FRAME_BYTES=1048576`
- `RELAY_HANDSHAKE_TIMEOUT_MS=10000`
- `RELAY_UPSTREAM_TIMEOUT_MS=30000`
- `RELAY_MAX_CONNECTIONS=100`

Limits must be positive integers.

Container build:

```sh
docker build -t maxt-relay relay/
docker run --rm -p 8080:8080 \
  -e RELAY_ALLOWED_ORIGINS=https://app.example \
  -e RELAY_ALLOWED_HTTP_ORIGINS=https://api.binance.com \
  -e RELAY_ALLOWED_WS_ORIGINS=wss://stream.binance.com:9443 \
  maxt-relay
```

## API

- `GET /healthz`
- `POST /v1/http`: `{url,method,headers:[[name,value]],body:null|string}`
- `GET /v1/ws`: first text frame `{url,headers,subscribe}`

The WebSocket relay sends `{"type":"ready"}` after the upstream connection
and subscription frames succeed. It sends `{"type":"error","detail":"..."}`
and closes the socket on setup failure.

HTTP methods: `GET`, `POST`, `PUT`, `DELETE`. Forwarded headers:
`authorization`, `x-mbx-apikey`, `content-type`. Redirects are not followed.

## Deployment boundary

The relay serves plain HTTP. Put it behind a same-site TLS ingress and expose
an HTTPS origin. Apply edge authentication and rate limits before `/v1/http` and
`/v1/ws`; the relay's only browser access control is the `Origin` allowlist.
The allowlist is not user authentication.

Keep every allowlist minimal. URL validation does not prevent DNS rebinding
after validation, malicious upstream responses, or operator allowlist errors.

The process keeps no sessions and does not persist traffic. It does not log
payloads, headers, or target URLs. Authentication headers and signed payloads
still exist in relay memory while requests are processed. Raw API secrets and
private keys remain in browser JavaScript/WASM memory, where XSS, extensions,
source maps, logs, or a compromised runtime can expose them.

## Browser configuration

```ts
await initialize({
  relayUrl: "https://relay.example",
  allowInsecureBrowserCredentials: true,
});
```

`relayUrl` is the TLS ingress origin, not `/v1/http` or `/v1/ws`.
