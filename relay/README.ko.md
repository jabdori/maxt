# maxt 릴레이

[English](README.md) | [한국어](README.ko.md)

인증된 Browser WebAssembly 호출을 위한 무상태(stateless) HTTP/WebSocket
릴레이입니다. 공개 브라우저 호출은 릴레이 없이 직접 연결할 수 있습니다.
Browser WASM을 `relayUrl`로 초기화하면 HTTP 요청은 릴레이를 사용합니다.
WebSocket은 거래소가 handshake header를 요구할 때만 릴레이를 사용합니다.

## 실행

```sh
export RELAY_ALLOWED_ORIGINS=https://app.example
export RELAY_ALLOWED_HTTP_ORIGINS=https://api.binance.com,https://fapi.binance.com
export RELAY_ALLOWED_WS_ORIGINS=wss://stream.binance.com:9443,wss://fstream.binance.com
cargo run --manifest-path relay/Cargo.toml
```

필수 환경변수:

- `RELAY_ALLOWED_ORIGINS`: 쉼표로 구분한 브라우저 `http`/`https` origin.
- `RELAY_ALLOWED_HTTP_ORIGINS`: 쉼표로 구분한 upstream `https` origin.
- `RELAY_ALLOWED_WS_ORIGINS`: 쉼표로 구분한 upstream `wss` origin.

Origin은 정확히 일치해야 합니다. Path, query, fragment, 인증 정보는 허용 목록
항목이 될 수 없습니다.

선택 환경변수:

- `RELAY_BIND=0.0.0.0:8080`
- `RELAY_MAX_REQUEST_BYTES=1048576`
- `RELAY_MAX_FRAME_BYTES=1048576`
- `RELAY_HANDSHAKE_TIMEOUT_MS=10000`
- `RELAY_UPSTREAM_TIMEOUT_MS=30000`
- `RELAY_MAX_CONNECTIONS=100`

제한값은 양의 정수여야 합니다.

컨테이너 빌드와 실행:

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
- `GET /v1/ws`: 첫 텍스트 프레임 `{url,headers,subscribe}`

Upstream 연결과 구독 프레임 전송에 성공하면 WebSocket 릴레이가
`{"type":"ready"}`를 보냅니다. 설정 실패 시
`{"type":"error","detail":"..."}`를 보내고 소켓을 닫습니다.

HTTP method: `GET`, `POST`, `PUT`, `DELETE`. 전달 header:
`authorization`, `x-mbx-apikey`, `content-type`. Redirect는 따르지 않습니다.

## 배포 경계

릴레이 자체는 평문 HTTP를 제공합니다. TLS ingress 뒤에 두고 HTTPS origin으로
노출하세요. `/v1/http`, `/v1/ws` 앞에 edge 인증과 속도 제한을 적용하세요.
릴레이 자체의 브라우저 접근 제어는 `Origin` 허용 목록뿐입니다.

모든 허용 목록은 최소화하세요. URL 검증은 검증 이후 DNS rebinding, 악성 upstream
응답, 운영자의 허용 목록 오류를 막지 못합니다.

프로세스는 세션을 유지하거나 트래픽을 저장하지 않습니다. Payload, header, 대상
URL도 기록하지 않습니다. 다만 요청을 처리하는 동안 인증 header와 서명된
payload는 릴레이 메모리에 존재합니다. 원본 API secret과 private key는 브라우저
JavaScript/WASM 메모리에 남으며 XSS, 확장 프로그램, source map, log, 손상된
runtime에 노출될 수 있습니다.

## 브라우저 설정

```ts
await initialize({
  relayUrl: "https://relay.example",
  allowInsecureBrowserCredentials: true,
});
```

`relayUrl`은 TLS ingress origin이며 `/v1/http` 또는 `/v1/ws`가 아닙니다.
