[English](upbit.md) | [한국어](upbit.ko.md)

# Upbit

`UpbitAdapter`는 서로 분리된 Upbit 현물 거래소 네 곳을 지원합니다. 어댑터를 만들 때 지역 하나를 선택합니다.

## 연결

```rust
use maxt::{Client, adapters::{UpbitAdapter, UpbitRegion}};

let korea = Client::new(UpbitAdapter::new());
let singapore = Client::new(UpbitAdapter::with_region(UpbitRegion::Singapore));
```

| 지역 | 값 | REST 기본 URL | 공개 WebSocket |
| --- | --- | --- | --- |
| 한국, 기본값 | `UpbitRegion::Korea` | `https://api.upbit.com` | `wss://api.upbit.com/websocket/v1` |
| 싱가포르 | `UpbitRegion::Singapore` | `https://sg-api.upbit.com` | `wss://sg-api.upbit.com/websocket/v1` |
| 인도네시아 | `UpbitRegion::Indonesia` | `https://id-api.upbit.com` | `wss://id-api.upbit.com/websocket/v1` |
| 태국 | `UpbitRegion::Thailand` | `https://th-api.upbit.com` | `wss://th-api.upbit.com/websocket/v1` |

상장, 호가, 계정, 인증 정보는 지역별로 분리됩니다. 한 지역에서 발급한 키로 다른 지역을 인증할 수 없습니다. 선택한 지역은 `UpbitAdapter::region()`으로 확인합니다.

Upbit 페어 코드는 호가 자산(quote asset)을 앞에 써서 `KRW-BTC`가 됩니다. `maxt`에는 `Market::spot(Exchange::Upbit, "BTC", "KRW")`를 넘기고, Upbit 표기는 `MarketInfo::native_symbol`에서 읽습니다.

```text
cargo run --example public_rest -- upbit BTC KRW
```

## 기능 지원

| 표면 | 지원 범위 |
| --- | --- |
| 공개 REST | 인증 없이 `markets`, `trades`, `order_book`, `ticker`, `candles` |
| 공개 스트림(stream) | `subscribe` 또는 `subscribe_with`로 체결, 호가, 현재가, 캔들 |
| 계정과 주문 | `with_credentials` 설정 뒤 잔고, 미체결 주문, 주문, 계정 스트림 |
| 파생상품 | 미지원. Upbit은 현물만 상장합니다 |
| `markets(MarketKind::Perpetual)` | 빈 목록 |

## REST 상한과 캔들 간격

| 입력 | 허용 값 |
| --- | --- |
| `trades` 제한 | 1~500 |
| `order_book` 깊이 | 한 쪽당 1~30단계. 생략하면 Upbit 기본값인 30단계 |
| `candles` 페이지 | Upbit 응답당 200개. `maxt`는 최대 100페이지 또는 20,000개까지 순회 |
| `candles` 요청 | `limit > 0`, `from`과 `to`가 모두 있으면 `from < to` |
| 자산 코드 | 대문자 ASCII 알파벳과 숫자 |

| 표면 | Upbit 공식 지원 | `maxt` 노출 |
| --- | --- | --- |
| REST 캔들 | 1초; 1, 3, 5, 10, 15, 30, 60, 240분; 1일, 1주, 1개월, 1년 | 1초, 1분, 3분, 5분, 15분, 30분, 1시간, 4시간, 1일, 1주, 1개월 |
| WebSocket 캔들 | 1초; 1, 3, 5, 10, 15, 30, 60, 240분 | 1초, 1분, 3분, 5분, 15분, 30분, 1시간, 4시간 |

`Interval`에는 10분봉과 연봉이 없어서 Upbit이 공식 지원하는 두 간격을 노출하지 못합니다. REST 초봉 이력은 최근 3개월까지만 제공됩니다. `maxt`는 REST 캔들을 오래된 순서로 반환합니다.

## 스트림

| 피드(feed) | 동작 |
| --- | --- |
| `Feed::OrderBook` | 전체 스냅숏(snapshot), 한 쪽당 기본 30단계. `Subscription`으로 깊이를 줄일 수 없음 |
| 체결 | 체결마다 이벤트 하나. `Trade::id`는 Upbit의 `sequential_id` |
| 현재가 | 전체 스냅숏. 나중 이벤트가 이전 상태를 대체함 |
| 캔들 | 형성 중인 봉을 반복 전송하며 아래 마감 규칙을 따름 |

스트림 캔들은 다음과 같이 처리합니다.

- 이미 구간이 끝난 스냅숏은 즉시 `closed = true`로 발행합니다.
- 현재 구간의 스냅숏이나 갱신은 `closed = false`이며 보관 중인 형성 봉을 대체합니다.
- 더 나중 구간이 오면 보관한 구간을 최대 한 번 마감한 뒤 새 형성 봉을 발행합니다.
- 체결이 없어 캔들이 생성되지 않거나 후속 프레임이 오지 않으면 전환에 따른 마감 이벤트도 없습니다.

REST 캔들의 마감 여부는 월 경계를 포함한 실제 구간 종료 시각과 클라이언트의 현재 시각으로 판단합니다.

## 요청 수 제한

| 그룹 | 한도 | 단위 |
| --- | --- | --- |
| `market` | 초당 10회 | IP |
| `candle` | 초당 10회 | IP |
| `trade` | 초당 10회 | IP |
| `ticker` | 초당 10회 | IP |
| `orderbook` | 초당 10회 | IP |
| WebSocket 연결 | 초당 5회 | 미인증은 IP, 인증은 계정 |
| WebSocket 요청 메시지 | 초당 5회, 분당 100회 | 연결 |

공개 REST 그룹 다섯 개는 서로 독립된 한도를 가집니다. `maxt`는 속도를 제한하지 않으므로 `Error::is_rate_limited()`와 Upbit 응답의 `Remaining-Req` 헤더를 사용하세요.

## Upbit 전용 메서드

한 요청으로 여러 페어를 읽으려면 `Client::adapter()`를 통해 호출합니다.

| 메서드 | 결과 | 요청 제한 그룹 |
| --- | --- | --- |
| `tickers(&[Market])` | 반환된 페어마다 현재가 하나 | `ticker` |
| `order_books(&[Market], Option<u32>)` | 반환된 페어마다 호가 하나, 한 쪽당 최대 30단계 | `orderbook` |
| `market_events()` | 페어마다 유의 여부와 제공되는 주의 기준 | `market` |

Upbit은 묶음 요청의 최대 페어 수를 공개하지 않습니다. 실제 상한은 서버가 받아들이는 요청 URL 길이에 따라 정해집니다.

### 유의 종목과 주의 종목

| 지정 | 공통 상태 | 상세 정보 출처 |
| --- | --- | --- |
| 투자 유의 종목 | `MarketStatus::Unknown` | `MarketInfo::status`, `market_events()` |
| 투자 주의 기준 | 상태는 `MarketStatus::Active` 유지 | `market_events()` |

한국은 유의 여부와 주의 기준을 함께 담은 `market_event`를 반환합니다. 싱가포르, 인도네시아, 태국은 예전 필드인 `market_warning`만 반환하므로 `UpbitMarketEvent::cautions`가 비어 있습니다. `MarketStatus::Active`는 주문 가능 보장이 아닙니다. 주문 가능 여부에 의존하기 전에 최신 페어 및 주문 정책을 확인하세요.

## 인증 정보

```rust,no_run
use maxt::{Client, adapters::{UpbitAdapter, UpbitRegion}};

let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
let secret_key = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
let adapter = UpbitAdapter::with_region(UpbitRegion::Korea)
    .with_credentials(access_key, secret_key);
let client = Client::new(adapter);
```

같은 지역에서 발급한 키 쌍을 사용하고 필요한 권한만 부여하세요. [`private_account`](../../examples/private_account.rs)와 [`private_stream`](../../examples/private_stream.rs)은 Upbit 전용 읽기 전용 예제입니다.

## 검증 범위

- 2026-07-31 한국에서 대표 공개 REST 및 WebSocket 스모크 테스트(smoke test)를 실행했고 위 예제 명령이 성공했습니다.
- 싱가포르, 인도네시아, 태국은 상장 목록, 현재가, 호가, 체결, 분봉 공개 REST를 표적 확인했습니다.
- 비공개 실제 호출은 검증하지 않았습니다. 비공개 동작은 오프라인 테스트와 공식 문서만 확인했습니다.

## 공식 문서

| 주제 | 링크 |
| --- | --- |
| 지역과 엔드포인트 | [Global 개요](https://global-docs.upbit.com/reference/api-overview) |
| 공개 REST | [페어](https://docs.upbit.com/kr/reference/list-trading-pairs) · [체결](https://docs.upbit.com/kr/reference/list-pair-trades) · [현재가](https://docs.upbit.com/kr/reference/list-tickers) · [호가](https://docs.upbit.com/kr/reference/list-orderbooks) · [캔들](https://docs.upbit.com/kr/reference/list-candles-minutes) |
| WebSocket | [안내](https://docs.upbit.com/kr/reference/websocket-guide) · [체결](https://docs.upbit.com/kr/reference/websocket-trade) · [현재가](https://docs.upbit.com/kr/reference/websocket-ticker) · [호가](https://docs.upbit.com/kr/reference/websocket-orderbook) · [캔들](https://docs.upbit.com/kr/reference/websocket-candle) |
| 한도와 인증 | [요청 수 제한](https://docs.upbit.com/kr/reference/rate-limits) · [인증](https://docs.upbit.com/kr/reference/auth) |

[공통 API](../common-api.ko.md) · [거래소 선택](../providers.ko.md)
