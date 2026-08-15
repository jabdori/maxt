# 거래소 지원

[English](providers.md) | [한국어](providers.ko.md)

모든 내장 어댑터는 계정 설정 없이 공개 시장 데이터와 시장 스트림을 제공합니다.
공통 작업은 `Client`, 거래소 전용 작업은 `Client::adapter()`를 통해 호출합니다.

먼저 [작업 중심 예제](examples.ko.md)를 실행한 뒤, 거래소별 한도와 공식 레퍼런스는
아래 제공자 페이지에서 확인하세요.

## 지원 상태

- [x] Binance Spot
- [x] Binance USD-M 무기한 선물
- [x] Upbit Spot: 한국, 싱가포르, 인도네시아, 태국
- [x] Bithumb Spot
- [x] Hyperliquid Spot과 무기한 선물

Binance testnet 생성자는 제공하지 않습니다.

## 생성자

- Binance Spot: `BinanceAdapter::spot()`
- Binance USD-M: `BinanceAdapter::usd_m_futures()`
- Upbit 한국: `UpbitAdapter::new()`
- Upbit 지역: `UpbitAdapter::with_region(region)`
- Bithumb: `BithumbAdapter::new()`
- Hyperliquid mainnet: `HyperliquidAdapter::new()`
- Hyperliquid testnet: `HyperliquidAdapter::testnet()`

`Client::new(adapter)` 전에 계정 접근 설정을 추가합니다.

- Binance: `.with_credentials(api_key, secret_key)`
- Upbit, Bithumb: `.with_credentials(access_key, secret_key)`
- Hyperliquid 계좌 조회: `.with_query_address(address)`; 로컬 서명을 사용하지 않음
- Hyperliquid 서명 작업: `.with_signer(private_key)`
- Hyperliquid 편의 설정: `.with_wallet(address, private_key)`는 두 설정을 함께 적용

`supports(feature) == false`는 필요한 어댑터 설정이 없거나 작업이 구조적으로
미지원이라는 뜻입니다. 해당 호출은 각각 `Error::Auth`, `Error::Unsupported`를
반환합니다. 거래소 권한, 시장 선택, 요청 검증은 별도로 확인합니다.

## 경계

- Binance Spot과 USD-M은 어댑터와 상태가 분리됩니다.
- Binance USD-M `Subscription` 하나가 여러 WebSocket을 병합할 수 있습니다.
- Upbit 지역은 host, 상장, 호가, 인증 정보가 분리됩니다.
- Bithumb은 `Feature::CandleStream`을 지원하지 않으며 `open_orders()`는 거래소 한 페이지를 읽습니다.
- Hyperliquid `positions_on(spot)`은 `Ok(vec![])`를 반환합니다.

## 레퍼런스

기록된 작업의 매핑과 구현·검증 상태는 생성된
[endpoint 지원 레퍼런스](../bindings/common/generated/api.md)를 참고하세요. 생성자,
시장·지역 한도, 공식 링크는 아래 거래소별 페이지에서 확인할 수 있습니다.

- [Binance](providers/binance.ko.md) ([English](providers/binance.md))
- [Upbit](providers/upbit.ko.md) ([English](providers/upbit.md))
- [Bithumb](providers/bithumb.ko.md) ([English](providers/bithumb.md))
- [Hyperliquid](providers/hyperliquid.ko.md) ([English](providers/hyperliquid.md))
