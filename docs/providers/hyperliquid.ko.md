# Hyperliquid

[English](hyperliquid.md) | [한국어](hyperliquid.ko.md)

## 거래 범위와 생성자

`HyperliquidAdapter` 하나가 현물과 기본 무기한 선물 DEX를 지원합니다.

| 생성자 | 네트워크 |
| --- | --- |
| `HyperliquidAdapter::new()` | Mainnet |
| `HyperliquidAdapter::testnet()` | Testnet |

| 마켓 | 메타데이터 | 지원 |
| --- | --- | --- |
| 현물 | `spotMeta` | 지원; USDC 외 quote asset 포함 |
| 기본 무기한 선물 DEX | `dex` 없는 `meta` | `MarketKind::Perpetual`로 지원 |
| HIP-3 무기한 선물 DEX | `perpDexs`, `dex`가 있는 `meta` | 미지원 |
| 결과형 자산 | `outcomeMeta` | 미지원 |

`meta`와 `spotMeta`는 첫 호출에서 캐시됩니다. 상장 정보를 다시 읽으려면 새
어댑터를 생성합니다. 현물 메타데이터는 `spotMeta.universe[].name`을 사용하고,
스트림 프레임은 이 이름 또는 `@{index}`를 사용할 수 있습니다. 반환된
`MarketInfo::native_symbol`을 사용합니다.

## 공개 REST

| 호출 | Hyperliquid 요청 | 계약 |
| --- | --- | --- |
| `markets(kind)` | `meta`, `spotMeta` | 표의 지원 범위 |
| `trades(market, limit)` | `recentTrades` | `limit in 1..=10`; 거래소 개수 인자 없음; `None -> 거래소 창 전체 (<= 10)`; `Some(limit)`는 로컬 절단; 최신순 |
| `order_book(market, depth)` | `l2Book` | `depth in 1..=20`; 로컬 절단 |
| `ticker(market)` | `metaAndAssetCtxs`, `spotMetaAndAssetCtxs` | 기준 가격 요약 |
| `funding_rates(request)` | `fundingHistory` | 공개·무기한 선물 전용; 원격 응답 최대 500건 |

| `HistoryRequest` 필드 | 거래소 요청 값 |
| --- | --- |
| 범위 | `from <= time < to` |
| `startTime` | `ceil_ms(from)` |
| `endTime` | `ceil_ms(to) - 1` |
| `cursor` | 설정 시 `from` 대체 |
| 페이지 종료 | `Page::next == None` |

| `Ticker` 필드 | 값 |
| --- | --- |
| `last_price` | `midPx ?? markPx` |
| `last_trade_time` | `None` |
| `timestamp` | 자산 컨텍스트를 읽은 시각 |
| `change` | `last_price - prevDayPx` |
| `change_rate` | `(last_price - prevDayPx) / prevDayPx`; `prevDayPx == 0 -> None` |

최근 체결 가격·시각은 `trades` 또는 `Feed::Trades`를 사용합니다.

## 캔들

| 계약 | 값 |
| --- | --- |
| 지원 `Interval` | `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour2`, `Hour4`, `Hour8`, `Hour12`, `Day1`, `Day3`, `Week1`, `Month1` |
| `Sec1` | `Error::Unsupported` |
| 보존 범위 | 간격별 최근 5,000개 |
| `Month1` 격자 | 30일 고정 |
| `quote_volume` | `None` |

## 공개 스트림

| 피드 | 구독 | 계약 |
| --- | --- | --- |
| `Feed::Trades` | `trades` | 거래소 시각의 체결 |
| `Feed::OrderBook` | `l2Book` | 각 측 최대 20단계의 전체 스냅샷 |
| `Feed::Ticker` | `activeAssetCtx` | REST ticker와 같은 기준 가격 |
| `Feed::Candles(interval)` | `candle` | 형성 봉·종결 봉 |

| 캔들 전이 | 이벤트 |
| --- | --- |
| `new.open_time > held.open_time` | `held(closed = true)`, 이후 `new(closed = false)` |
| `new.open_time == held.open_time` | `held` 교체; `new(closed = false)` 발행 |
| `new.open_time < held.open_time` | 프레임 폐기 |
| 재연결 | `held` 폐기; 연결 경계를 넘는 종결 이벤트 없음 |

어댑터는 15초마다 `{"method":"ping"}`을 전송합니다. `l2Book.nSigFigs`와
`l2Book.mantissa`는 노출하지 않습니다.

## 인증 후 기능과 Hyperliquid 전용 메서드

`with_wallet(address, private_key)`를 사용합니다. 지갑 값은 첫 비공개 호출에서
검증합니다. `Client::supports`는 지갑 설정 여부만 확인합니다. 비밀 키는 로컬
서명에만 사용하며 `Debug` 출력에서 숨깁니다.

| 시장 | 인증 후 기능 |
| --- | --- |
| 현물 | 잔고, 미체결 주문, 주문 생성·취소, 계정 스트림 |
| 무기한 선물 | 현물 기능 + 포지션, 증거금 요약·설정, 펀딩 지급 이력, reduce-only 주문 |

`positions()`는 모든 미결 무기한 선물 포지션을 반환하고,
`positions_on(spot) == Ok(vec![])`입니다.

| 주문 입력 | 계약 |
| --- | --- |
| `order_type` | `Limit`; `Market -> Error::Unsupported` |
| `size` | `Size::Base`; `size > 0` |
| `price` | `price > 0`; 최대 소수 자릿수: 무기한 선물 `6 - szDecimals`, 현물 `8 - szDecimals`; 정수가 아닌 가격은 전체 유효숫자 `<= 5`; 정수 가격은 유효숫자 제한 제외 |
| `time_in_force` | `GTC`, `IOC`, `PostOnly`; `FOK -> Error::Unsupported` |
| `reduce_only` | `MarketKind::Perpetual`만 지원 |
| 최소 주문 명목가치 | 거래소 검증; `maxt` 사전 검증 없음 |

| `set_margin` 입력 | 계약 |
| --- | --- |
| 필드 | `leverage.is_some() && margin_mode.is_some()` |
| `leverage` | 양의 정수; `leverage <= asset.max_leverage` |
| `margin_mode == Cross` | `asset.only_isolated == true`이면 `Error::InvalidRequest` |

| 메서드 | 계약 |
| --- | --- |
| `asset_context(&market)` | mark·mid·oracle 가격, 현재 funding, open interest, 주문 정밀도 |
| `non_funding_ledger(from, to, cursor, limit)` | 입금·출금·이체·청산; 펀딩 제외; 지갑 필요; 거래소 페이지 `<= 500` |

| `non_funding_ledger` 필드 | 계약 |
| --- | --- |
| 거래소 범위 | `from_ms <= time <= to_ms` |
| `cursor` | 설정 시 `from` 대체 |
| `limit` | 로컬 목표값; 같은 millisecond 그룹은 초과 가능 |

## 한도·미지원·공식 링크

| 범위 | 한도 |
| --- | --- |
| REST | IP당 분당 합산 weight 1,200 |
| WebSocket 연결 | 동시 10개; 신규 분당 30개 |
| WebSocket 구독 | 1,000개 |
| WebSocket 발신 메시지 | 모든 연결 합산 분당 2,000개 |

`l2Book` weight는 2, 일반 `info` 요청 weight는 20입니다. `candleSnapshot`은
응답 60개당 weight를 추가합니다. `maxt`는 요청 속도를 제한하지 않습니다.

HIP-3·결과형 자산·`Sec1`·시장가 주문·`FOK`·`l2Book.nSigFigs`·`l2Book.mantissa`는 미지원입니다.

- [API 개요](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api)
- [Info endpoint](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [무기한 선물 정보](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals)
- [현물 정보](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot)
- [WebSocket 구독](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)
- [요청 한도](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)

[공통 API](../common-api.ko.md) · [거래소 선택](../providers.ko.md)
