# Hyperliquid

[English](hyperliquid.md) | [한국어](hyperliquid.ko.md)

## 거래소와 생성자

| 생성자 | 네트워크(network) |
| --- | --- |
| `HyperliquidAdapter::new()` | Mainnet |
| `HyperliquidAdapter::testnet()` | Testnet |

어댑터 하나가 Spot과 기본 무기한 선물 DEX를 제공합니다.

| 시장 | 메타데이터(metadata) | 지원 |
| --- | --- | --- |
| Spot | `spotMeta` | 지원; USDC 외 quote asset 포함 |
| 기본 무기한 선물 DEX | `dex`가 없는 `meta` | `MarketKind::Perpetual` |
| HIP-3 무기한 선물 DEX | `perpDexs`, `dex`가 있는 `meta` | 미지원 |
| 결과형 자산(outcome asset) | `outcomeMeta` | 미지원 |

어댑터는 첫 호출에서 `meta`, `spotMeta`를 캐시(cache)합니다. 상장 정보를 다시
읽으려면 새 어댑터를 생성하세요. Spot 메타데이터는
`spotMeta.universe[].name`을 사용합니다. 스트림 프레임(frame)은 이 이름 또는
`@{index}`를 사용할 수 있습니다. 반환된
`MarketInfo::native_symbol`을 사용하세요.

## REST

| 호출 | Hyperliquid 요청 | 계약 |
| --- | --- | --- |
| `markets(kind)` | `meta`, `spotMeta` | 위 지원 표의 시장 |
| `trades(market, limit)` | `recentTrades` | 거래소 개수 인자 없음; 거래소 페이지 `<= 10`; 최신순 |
| `order_book(market, depth)` | `l2Book` | `depth: 1..=20`; 각 측 `len() <= depth`; 로컬 절단 |
| `ticker(market)` | `metaAndAssetCtxs`, `spotMetaAndAssetCtxs` | 기준 가격 요약 |
| `funding_rates(request)` | `fundingHistory` | 공개; Perpetual 전용; 거래소 페이지 `<= 500` |

`trades`의 `limit` 계약:

| `limit` | 결과 |
| --- | --- |
| `None` | 거래소 페이지 전체 `<= 10` |
| `Some(limit)` | `limit: 1..=10`; 최신순 결과를 로컬에서 `limit`건으로 절단 |

| `HistoryRequest` 필드·상태 | 계약 |
| --- | --- |
| `from`, `to` | `from <= timestamp < to` |
| `startTime` | `ceil_ms(from)` |
| `endTime` | `ceil_ms(to) - 1` |
| `cursor` | 불투명 재개 지점(opaque resume point); `cursor != None` → `from` 무시 |
| `limit` | 로컬 페이지 크기 목표값; 최대값 아님 |
| `limit == 0` | `funding_rates(request)` → 네트워크 I/O 전 `Error::InvalidRequest`; `funding_payments(request)` → 인증 확인·네트워크 I/O 전 같은 오류 |
| 같은 밀리초(millisecond) 그룹이 `limit` 경계를 넘음 | 그룹을 다음 페이지로 미루면 `items.len() < limit`; 첫 그룹만으로 `limit`를 넘으면 `items.len() > limit` |
| 다음 요청 | `cursor = page.next`; `page.next == None`까지 반복 |

`Ticker` 매핑:

| `Ticker` 필드 | 출처 |
| --- | --- |
| `last_price` | `midPx.or(markPx)`; `midPx == None`이면 `markPx`; 최신 체결 가격 아님 |
| `last_trade_time` | `None` |
| `timestamp` | 로컬 조회 시각 |
| `change` | `last_price - prevDayPx` |
| `change_rate` | `(last_price - prevDayPx) / prevDayPx`; `prevDayPx == 0 → None` |

체결 가격과 시각은 `trades` 또는 `Feed::Trades`에서 조회하세요.

## 캔들

| 계약 | 값 |
| --- | --- |
| 지원하는 `Interval` | `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour2`, `Hour4`, `Hour8`, `Hour12`, `Day1`, `Day3`, `Week1`, `Month1` |
| `Sec1` | `Error::Unsupported` |
| 보존 범위 | 간격별 최신 5,000건 |
| `Month1` grid | 30일 고정 |
| `quote_volume` | `None` |

## 스트림

| feed | 구독 | 계약 |
| --- | --- | --- |
| `Feed::Trades` | `trades` | 거래소 timestamp를 포함한 체결 |
| `Feed::OrderBook` | `l2Book` | 전체 스냅샷(snapshot); 각 측 최대 20개 호가 단계; diff 없음 |
| `Feed::Ticker` | `activeAssetCtx` | REST ticker와 같은 매핑 |
| `Feed::Candles(interval)` | `candle` | 형성 중 캔들과 확정 캔들 |

| 수신 캔들의 `open_time` | 결과 |
| --- | --- |
| 현재 캔들보다 큼 | 현재 캔들 `closed = true` 발행 → 수신 캔들 `closed = false` 발행 |
| 현재 캔들과 같음 | 현재 캔들 교체; `closed = false` 발행 |
| 현재 캔들보다 작음 | 프레임 폐기 |
| 재연결 | 현재 캔들 폐기; 연결 경계를 넘는 확정 이벤트 없음 |

어댑터는 15초마다 `{"method":"ping"}`을 전송합니다. `l2Book.nSigFigs`,
`l2Book.mantissa`는 노출하지 않습니다.

## 비공개 API와 거래소 전용 API

`.with_wallet(address, private_key)`로 지갑을 설정합니다. 지갑 값은 첫 비공개
호출에서 검증합니다. `Client::supports`는 지갑 설정 여부만 확인합니다. 개인 키
(private key)는 로컬 서명에만 사용하며 `Debug` 출력에서 숨깁니다.

| 시장 | 비공개 기능 |
| --- | --- |
| Spot | 잔고, 미체결 주문, 주문 생성·취소, 계좌 스트림 |
| Perpetual | Spot 기능, 포지션, 증거금 요약·설정, funding 지급 이력, reduce-only 주문 |

`positions()`는 모든 미결 무기한 선물 포지션(position)을 반환합니다.
`positions_on(spot) == Ok(vec![])`입니다.

| 주문 입력 | 계약 |
| --- | --- |
| `order_type` | `Limit`; `Market → Error::Unsupported` |
| `size` | `Size::Base`; `size > 0` |
| `price` | `price > 0`; 최대 소수 자릿수는 Perpetual `6 - szDecimals`, Spot `8 - szDecimals`; 비정수 가격 `significant_figures <= 5`; 정수 가격은 유효숫자 제한 없음 |
| `time_in_force` | `GTC`, `IOC`, `PostOnly`; `FOK → Error::Unsupported` |
| `reduce_only` | `MarketKind::Perpetual`만 지원 |
| 최소 명목가치(minimum notional) | 거래소 검증; `maxt` 사전 검증 없음 |

| `set_margin` 입력 | 계약 |
| --- | --- |
| 필드 | `leverage.is_some() && margin_mode.is_some()` |
| `leverage` | 양의 정수; `leverage <= asset.max_leverage` |
| `margin_mode == Cross` | `asset.only_isolated == true → Error::InvalidRequest` |

다음 메서드는 `Client::adapter()`가 반환한 어댑터에서 호출합니다.

| 메서드 | 계약 |
| --- | --- |
| `asset_context(&market)` | mid, mark, oracle 가격, funding, open interest, 주문 정밀도 |
| `non_funding_ledger(from, to, cursor, limit)` | 입금, 출금, 이체, 청산; funding 제외; 지갑 필요; 거래소 페이지 `<= 500` |

| `non_funding_ledger` 필드·상태 | 계약 |
| --- | --- |
| `from`, `to` | 거래소 밀리초(millisecond) 범위 `from_ms <= time <= to_ms`; 양쪽 경계 포함 |
| `cursor` | 불투명 재개 지점; `cursor != None` → `from` 무시 |
| `limit` | 로컬 페이지 크기 목표값; 최대값 아님 |
| `limit == Some(0)` | `non_funding_ledger(...)` → 인증 확인·네트워크 I/O 전 `Error::InvalidRequest` |
| 같은 밀리초 그룹이 `limit` 경계를 넘음 | 그룹을 다음 페이지로 미루면 `items.len() < limit`; 첫 그룹만으로 `limit`를 넘으면 `items.len() > limit` |
| 다음 요청 | `cursor = page.next`; `page.next == None`까지 반복 |

알 수 없는 원장(ledger) `type` 문자열은 일반 값으로 합치지 않습니다.
`HyperliquidLedgerKind::Other(provider_name)`에 원문을 보존합니다.

## 한도와 공식 링크

| 범위 | 한도 |
| --- | --- |
| REST | IP당 분당 합산 요청 가중치(request weight) 1,200 |
| WebSocket 연결 | 동시 10개; 신규 분당 30개 |
| WebSocket 구독 | 1,000개 |
| WebSocket 발신 메시지 | 모든 연결 합산 분당 2,000개 |

`l2Book` weight는 2, 일반 `info` 요청 weight는 20입니다. `candleSnapshot`은 반환
항목 60건마다 weight를 추가합니다. `maxt`는 요청 속도를 제한하지 않습니다.

HIP-3, outcome asset, `Sec1`, 시장가 주문, `FOK`, `l2Book.nSigFigs`,
`l2Book.mantissa`는 노출하지 않습니다.

- [API 개요](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api)
- [Info endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [Perpetual 정보](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals)
- [Spot 정보](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot)
- [WebSocket 구독](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)
- [요청 한도](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)

[공통 API](../common-api.ko.md) · [거래소 지원](../providers.ko.md)
