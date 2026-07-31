# 제공자 선택

[English](providers.md) | [한국어](providers.ko.md)

모든 내장 어댑터의 공개 시장 데이터는 인증 정보 없이 사용할 수 있습니다.

## 용도별 선택

| 용도 | 어댑터 |
| --- | --- |
| 원화 현물 | [Upbit](providers/upbit.ko.md), [Bithumb](providers/bithumb.ko.md) |
| 글로벌 현물 | [Binance Spot](providers/binance.ko.md) |
| `USDT`/`USDC` 증거금 무기한 선물 | [Binance USD-M](providers/binance.ko.md), [Hyperliquid](providers/hyperliquid.ko.md) |
| 지갑 서명 | [Hyperliquid](providers/hyperliquid.ko.md) |
| 내장 테스트넷 | `HyperliquidAdapter::testnet()` |

Binance 테스트넷 호스트는 노출하지 않습니다.

## 생성자와 인증

| 제공자 | 공개 생성자 | 비공개 기능 |
| --- | --- | --- |
| Upbit 한국 | `UpbitAdapter::new()` | `with_credentials(access_key, secret_key)` |
| Upbit 싱가포르·인도네시아·태국 | `UpbitAdapter::with_region(region)` | `with_credentials(access_key, secret_key)` |
| Bithumb | `BithumbAdapter::new()` | `with_credentials(access_key, secret_key)` |
| Binance Spot | `BinanceAdapter::spot()` | `with_credentials(api_key, secret_key)` |
| Binance USD-M | `BinanceAdapter::usd_m_futures()` | `with_credentials(api_key, secret_key)` |
| Hyperliquid 메인넷 | `HyperliquidAdapter::new()` | `with_wallet(address, private_key)` |
| Hyperliquid 테스트넷 | `HyperliquidAdapter::testnet()` | `with_wallet(address, private_key)` |

| 상태 | 계약 |
| --- | --- |
| 비공개 작업 매핑·인증 정보 없음 | `supports(feature) == false`; 호출 결과 `Error::Auth` |
| 구조적 미지원 | `supports(feature) == false`; 호출 결과 `Error::Unsupported` |
| 작업 매핑·인증 정보 있음 | `supports(feature) == true`; 호출 결과는 요청별 검증·거래소 응답에 따라 결정 |

## 설계 차이

| 제공자 | 계약 |
| --- | --- |
| Upbit | `MarketKind::Spot`; 지역별 호스트·상장·호가·인증 정보 분리 |
| Bithumb | `MarketKind::Spot`; `supports(Feature::CandleStream) == false`; `open_orders()`는 `/v1/orders` 한 페이지 |
| Binance Spot | `MarketKind::Spot`; USD-M과 별도 어댑터·상태 |
| Binance USD-M | `MarketKind::Perpetual`; `Subscription` 하나가 여러 WebSocket을 병합할 수 있음 |
| Hyperliquid | `Spot`, `Perpetual`; `positions_on(spot) == Ok(vec![])`; `Ticker::last_price = midPx.or(markPx)` |

캔들 간격, 요청 한도, 필드 출처, 주문 형식은 제공자별 레퍼런스를 따릅니다.

## 제공자별 레퍼런스

- [Upbit](providers/upbit.ko.md) ([English](providers/upbit.md))
- [Bithumb](providers/bithumb.ko.md) ([English](providers/bithumb.md))
- [Binance](providers/binance.ko.md) ([English](providers/binance.md))
- [Hyperliquid](providers/hyperliquid.ko.md) ([English](providers/hyperliquid.md))
