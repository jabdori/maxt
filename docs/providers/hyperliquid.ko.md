[English](hyperliquid.md) | [한국어](hyperliquid.ko.md)

# Hyperliquid

`HyperliquidAdapter` 하나가 공개 현물과 기본 무기한 선물 탈중앙화 거래소(DEX)를 같은 API로 제공합니다. 공개 호출에는 지갑이 필요하지 않습니다.

## 생성자

| 생성자 | 용도 |
| --- | --- |
| `HyperliquidAdapter::new()` | 메인넷 공개 클라이언트 |
| `HyperliquidAdapter::testnet()` | 테스트넷 공개 클라이언트 |
| `.with_wallet(address, private_key)` | 계좌·거래 호출에 인증 정보를 추가합니다. 첫 비공개 호출에서 값을 검사합니다 |

## 마켓 범위

| 마켓 | 출처 | 지원 범위 |
| --- | --- | --- |
| 현물 | `spotMeta` | USDC가 아닌 호가 자산 페어까지 노출 |
| 기본 무기한 선물 DEX | `dex` 없는 `meta` | `MarketKind::Perpetual`로 노출 |
| HIP-3 무기한 선물 DEX | `perpDexs`, `dex`가 있는 `meta` | 미노출 |
| 결과형 자산(outcome assets) | `outcomeMeta` | 미노출 |

어댑터는 처음 사용할 때 `meta`와 `spotMeta`를 캐시합니다. 그 뒤 추가된 상장을 찾으려면 새 어댑터를 만드세요. 현물 거래소 프로토콜 심볼(wire symbol)은 대부분 `@{index}`입니다. 추측하지 말고 `markets(MarketKind::Spot)`으로 찾으세요.

## 공개 REST

| 호출 | Hyperliquid 요청 | 계약 |
| --- | --- | --- |
| `markets(kind)` | `meta`, `spotMeta` | 위 범위의 마켓 목록 |
| `trades(market, limit)` | `recentTrades` | 최신순. 지정한 `limit`은 `1..=10`. 비우면 엔드포인트의 최근 10건을 반환합니다 |
| `order_book(market, depth)` | `l2Book` | 지정한 `depth`는 `1..=20`. 전체 응답을 어댑터가 로컬에서 자릅니다 |
| `ticker(market)` | `metaAndAssetCtxs`, `spotMetaAndAssetCtxs` | 최근 체결가가 아닌 기준 가격 요약. 아래 설명을 보세요 |
| `candles(request)` | `candleSnapshot` | `1m`부터 `1M`까지 14개 간격. `Sec1`은 미지원 |
| `funding_rates(request)` | `fundingHistory` | 공개·무기한 선물 전용. 시간 범위 응답은 최대 500건 |

Hyperliquid는 간격마다 최근 5,000개 캔들만 제공합니다. 이는 페이지 크기가 아니라 보존 범위입니다. 큰 로컬 `limit`도 거래소가 더는 제공하지 않는 데이터를 만들지 못합니다.

Hyperliquid의 `1M` 캔들은 달력 월이 아니라 고정 30일 격자를 사용합니다.

`HistoryRequest`의 `from`은 포함(inclusive), `to`는 제외(exclusive)입니다. Hyperliquid의 `endTime`은 포함이고 밀리초 단위이므로, 어댑터는 `to`보다 엄격히 앞선 마지막 밀리초를 보냅니다. `Page::next`가 `None`일 때까지 따라가세요.

## Ticker 의미

| 필드 | Hyperliquid에서의 의미 |
| --- | --- |
| `last_price` | `midPx`, 없으면 `markPx`. 필드 이름과 달리 가장 최근 체결가는 아닙니다 |
| `last_trade_time` | `None`. 자산 컨텍스트(asset context)에는 체결 시각이 없습니다 |
| `timestamp` | `maxt`가 자산 컨텍스트를 읽은 시각. 컨텍스트 자체에는 시각이 없습니다 |
| `change`, `change_rate` | 위 기준 가격을 `prevDayPx`와 비교한 값 |

정확한 최근 체결 가격과 시각이 필요하면 `trades` 또는 `Feed::Trades`를 사용하세요.

## 공개 스트림

| 피드 | 구독 | 동작 |
| --- | --- | --- |
| `Feed::Trades` | `trades` | 거래소 시각이 있는 체결 |
| `Feed::OrderBook` | `l2Book` | 한쪽당 20단계 전체 스냅샷. 차분이 아닙니다 |
| `Feed::Ticker` | `activeAssetCtx` | REST ticker와 같은 기준 가격 의미 |
| `Feed::Candles(interval)` | `candle` | 미완성 업데이트와, 다음 구간이 열릴 때 로컬에서 확정 표시한 캔들 1개 |
| 연결 유지 | `{"method":"ping"}` | 15초마다 전송. Hyperliquid는 60초 동안 유휴인 연결을 닫습니다 |

공식 `l2Book` 집계 옵션은 `nSigFigs`와 `mantissa`입니다. 공통 API는 둘을 노출하지 않으며 전송하지도 않습니다.

## Hyperliquid 전용 호출

| 메서드 | 용도 |
| --- | --- |
| `asset_context(&market)` | 공개 mark·mid·oracle 가격, 현재 펀딩, 미결제약정, 주문 정밀도 |
| `non_funding_ledger(from, to, cursor, limit)` | 지갑이 필요한 입금·출금·이체·청산 기록. 펀딩 지급은 포함하지 않습니다 |

## 정밀도와 최소 주문 명목가치

| 규칙 | 값 |
| --- | --- |
| 수량 소수 자릿수 | 자산의 `szDecimals` |
| 무기한 선물 가격 소수 자릿수 | `6 - szDecimals` |
| 현물 가격 소수 자릿수 | `8 - szDecimals` |
| 유효숫자(significant digits) | 정수가 아닌 가격은 최대 5자리 |
| 최소 주문 명목가치 | Hyperliquid가 결정하며 `maxt`는 미리 검사하지 않습니다 |

`asset_context`가 소수 자릿수 한도를 제공합니다. 주문 가격에는 소수 자릿수 한도와 유효숫자 규칙이 모두 적용됩니다.

## 요청 한도

| 범위 | 공식 한도 |
| --- | --- |
| REST | IP당 분당 총 요청 가중치 1,200. 엔드포인트마다 가중치가 다릅니다 |
| WebSocket 연결 | 동시 10개, 분당 신규 연결 30개 |
| WebSocket 구독 | 1,000개 |
| WebSocket 발신 메시지 | 모든 연결을 합쳐 분당 2,000개 |

`maxt`는 요청 속도를 제한하지 않습니다. 가중치 예산을 관리하고 `Error::is_rate_limited()`가 참이면 백오프하세요.

## 지갑 보안

- Hyperliquid는 API 키가 아니라 지갑 주소와 개인 키를 사용합니다.
- 계좌 개인 키보다 승인된 API wallet 키를 권장합니다. 거래는 가능하지만 출금은 할 수 없습니다.
- 서명은 로컬에서 처리합니다. 개인 키는 전송하지 않고 `Debug` 출력에서도 가립니다.
- 공개 REST와 스트림은 `.with_wallet(...)` 없이 동작합니다.

## 검증 범위

2026-07-31에 메인넷의 대표 현물·기본 무기한 선물 마켓으로 공개 REST와 `Trades`, `OrderBook`, `Ticker`, `Candles` 스트림 스모크 테스트(smoke test)를 통과했습니다. 비공개 실시간 호출은 검증하지 않았습니다.

## 예제

```text
cargo run --example public_rest -- hyperliquid HYPE USDC
cargo run --example public_stream -- hyperliquid HYPE USDC
```

## 공식 문서

- [API 개요](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api)
- [Info endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [무기한 선물 info](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals)
- [현물 info](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot)
- [Asset IDs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/asset-ids)
- [WebSocket 구독](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)
- [Timeouts and heartbeats](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/timeouts-and-heartbeats)
- [요청 한도](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)

---

[공통 API](../common-api.ko.md) · [거래소 선택](../providers.ko.md)
