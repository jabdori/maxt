# 제공자 선택

[English](providers.md) | [한국어](providers.ko.md)

함께 제공되는 모든 어댑터는 공개 시장 데이터를 읽을 때 인증 정보가 필요하지
않습니다.

## 용도별 선택

| 필요한 기능 | 어댑터 |
| --- | --- |
| 원화 현물 마켓 | [Upbit](providers/upbit.ko.md) 또는 [Bithumb](providers/bithumb.ko.md) |
| 글로벌 현물 마켓 | [Binance Spot](providers/binance.ko.md) |
| USD 연동 자산 증거금 무기한 선물 | [Binance USD-M](providers/binance.ko.md) 또는 [Hyperliquid](providers/hyperliquid.ko.md) |
| API 키 대신 지갑(wallet) 서명 | [Hyperliquid](providers/hyperliquid.ko.md) |
| 크레이트가 직접 연결하는 테스트 네트워크 | `HyperliquidAdapter::testnet()`; Binance 테스트넷 호스트는 이 크레이트에서 노출하지 않습니다 |

## 생성자와 인증 정보

| 제공자 | 공개 기능 생성자 | 비공개 기능 활성화 |
| --- | --- | --- |
| Upbit | 한국은 `UpbitAdapter::new()`, 싱가포르·인도네시아·태국은 `with_region(...)` | `with_credentials(access_key, secret_key)` |
| Bithumb | `BithumbAdapter::new()` | `with_credentials(access_key, secret_key)` |
| Binance Spot | `BinanceAdapter::spot()` | `with_credentials(api_key, secret_key)` |
| Binance USD-M | `BinanceAdapter::usd_m_futures()` | `with_credentials(api_key, secret_key)` |
| Hyperliquid 메인넷/테스트넷 | `HyperliquidAdapter::new()` 또는 `::testnet()` | `with_wallet(address, private_key)` |

인증 정보가 없으면 `Client::supports`는 비공개 기능에 `false`를 반환하고, 해당
호출은 `Error::Auth`를 반환합니다. 구조적으로 제공할 수 없는 기능도 `false`를
반환하지만, 이때 호출 결과는 `Error::Unsupported`입니다. [기능 확인](common-api.ko.md#기능-확인)을
참고하세요.

## 설계에 영향을 주는 차이

| 제공자 | 설계 경계 |
| --- | --- |
| Upbit | 현물만 지원합니다. 네 지역은 호스트, 상장 목록, 호가창, 인증 정보가 서로 분리되어 있으며 한국이 기본값입니다. |
| Bithumb | 현물만 지원하고 캔들 스트림은 없습니다. 현재 `open_orders` 구현은 한 페이지만 읽어 최대 100개 주문을 반환합니다. |
| Binance | Spot과 USD-M은 별도 어댑터 구성입니다. 마켓 종류(`MarketKind`)가 구성과 맞지 않으면 잘못된 요청입니다. 하나의 논리적 USD-M 구독이 여러 소켓을 사용한 뒤 이벤트를 합칠 수 있습니다. |
| Hyperliquid | 어댑터 하나가 현물과 무기한 선물 마켓을 함께 처리합니다. 펀딩, 마진 설정, 포지션 감소 전용 주문은 현물 인자를 거절하지만 `positions_on(spot)`은 빈 목록을 반환합니다. `Ticker::last_price`는 최근 체결가가 아니라 `midPx`이며, 값이 없으면 `markPx`를 사용합니다. |

제공자와 관계없이 REST 체결은 최신순, 캔들은 오래된 순, 호가창의 양쪽은 최우선
호가부터 정렬됩니다. 숫자는 `maxt::Decimal`을 사용하고, 제공자가 공개하지 않은
필드는 `None`으로 남습니다. 캔들 범위, 스트림 재연결, 재시도 안전성, 비공개 호출의
경계는 [공통 API 레퍼런스](common-api.ko.md)를 참고하세요.

제공자 전용 일괄 처리, 마켓 경보, 네이티브 컨텍스트, 원장 호출은 구체적인 어댑터에
남아 있습니다. `Client::adapter()`로 접근할 수 있으며 이식 가능한 공통 API에는
추가하지 않습니다.

## 실시간 검증 범위

2026-07-31에 Upbit 한국 `BTC/KRW`, Bithumb `BTC/KRW`, Binance Spot
`BTC/USDT`, Binance USD-M `BTC/USDT` 무기한 선물, Hyperliquid 메인넷
`BTC/USDC` 무기한 선물을 대상으로 공개 REST와 지원되는 공개 스트림을 실시간
검증했습니다. 이 검사는 인증 정보를 사용하지 않았습니다. 비공개 계좌와 거래 경로는
실시간으로 검증하지 않았습니다.

## 제공자별 레퍼런스

- [Upbit](providers/upbit.ko.md) ([English](providers/upbit.md))
- [Bithumb](providers/bithumb.ko.md) ([English](providers/bithumb.md))
- [Binance](providers/binance.ko.md) ([English](providers/binance.md))
- [Hyperliquid](providers/hyperliquid.ko.md) ([English](providers/hyperliquid.md))
