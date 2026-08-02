# 거래소 지원표

[English](providers.md) | [한국어](providers.ko.md)

모든 내장 어댑터는 인증 정보 없이 공개 시장 데이터를 제공합니다.

## 거래소 선택

| 요구사항 | 어댑터 |
| --- | --- |
| 원화 현물 | `UpbitAdapter` 또는 `BithumbAdapter` |
| 글로벌 현물 | `BinanceAdapter::spot()` |
| USD 증거금 무기한 선물 | `BinanceAdapter::usd_m_futures()` 또는 `HyperliquidAdapter` |
| 지갑 서명(wallet signing) | `HyperliquidAdapter` |
| 내장 테스트넷(testnet) | `HyperliquidAdapter::testnet()` |

Binance 테스트넷 host는 노출하지 않습니다.

## 생성자와 인증 정보

| 거래소 | 공개 생성자 | 비공개 설정 |
| --- | --- | --- |
| Upbit 한국 | `UpbitAdapter::new()` | `.with_credentials(access_key, secret_key)` |
| Upbit 싱가포르, 인도네시아, 태국 | `UpbitAdapter::with_region(region)` | `.with_credentials(access_key, secret_key)` |
| Bithumb | `BithumbAdapter::new()` | `.with_credentials(access_key, secret_key)` |
| Binance Spot | `BinanceAdapter::spot()` | `.with_credentials(api_key, secret_key)` |
| Binance USD-M | `BinanceAdapter::usd_m_futures()` | `.with_credentials(api_key, secret_key)` |
| Hyperliquid mainnet | `HyperliquidAdapter::new()` | `.with_wallet(address, private_key)` |
| Hyperliquid testnet | `HyperliquidAdapter::testnet()` | `.with_wallet(address, private_key)` |

| 상태 | 계약 |
| --- | --- |
| 비공개 작업 매핑됨; 인증 정보 없음 | `supports(feature) == false`; 호출 결과 `Error::Auth` |
| 구조적으로 미지원 | `supports(feature) == false`; 호출 결과 `Error::Unsupported` |
| 작업 매핑됨; 인증 정보 설정됨 | `supports(feature) == true`; 요청 검증과 거래소 응답이 호출 결과 결정 |

## 지원 경계

| 거래소 | 시장 | 주요 경계 |
| --- | --- | --- |
| Upbit | `MarketKind::Spot` | 지역별 host, 상장, 호가, 인증 정보 분리 |
| Bithumb | `MarketKind::Spot` | `supports(Feature::CandleStream) == false`; `open_orders()`는 `/v1/orders` 1페이지 |
| Binance Spot | `MarketKind::Spot` | USD-M과 별도 어댑터·상태 |
| Binance USD-M | `MarketKind::Perpetual` | `Subscription` 하나가 여러 WebSocket을 병합할 수 있음 |
| Hyperliquid | `Spot`, `Perpetual` | `positions_on(spot) == Ok(vec![])`; `Ticker::last_price = midPx.or(markPx)` |

공통 작업은 `Client`에서 호출합니다. 거래소 전용 메서드는
`Client::adapter()`가 반환한 어댑터에서 호출합니다.

## 거래소별 레퍼런스

- [Upbit](providers/upbit.ko.md) ([English](providers/upbit.md))
- [Bithumb](providers/bithumb.ko.md) ([English](providers/bithumb.md))
- [Binance](providers/binance.ko.md) ([English](providers/binance.md))
- [Hyperliquid](providers/hyperliquid.ko.md) ([English](providers/hyperliquid.md))
