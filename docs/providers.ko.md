[English](providers.md) | [한국어](providers.ko.md)

# 거래소 고르기

## 어떤 일에 어떤 어댑터

| 필요한 것 | 어댑터 |
| --- | --- |
| 원화 마켓 | [Upbit](providers/upbit.ko.md) 또는 [Bithumb](providers/bithumb.ko.md), 둘 다 현물 전용 |
| 글로벌 현물 | [Binance](providers/binance.ko.md), `BinanceAdapter::spot()` |
| 무기한 선물 | [Binance USD-M](providers/binance.ko.md), `BinanceAdapter::usd_m_futures()`, 또는 [Hyperliquid](providers/hyperliquid.ko.md) |
| API 키 대신 지갑 서명 | [Hyperliquid](providers/hyperliquid.ko.md) 하나뿐 |
| 테스트 네트워크 | [Hyperliquid](providers/hyperliquid.ko.md), `HyperliquidAdapter::testnet()`. 나머지 셋은 여기에 테스트 환경이 없습니다 |

공개 시세는 넷 다 인증 정보가 필요 없습니다.

## 생성자와 인증 정보

| 어댑터 | 만드는 법 | 인증 정보 |
| --- | --- | --- |
| Upbit | `UpbitAdapter::new()`, 싱가포르·인도네시아·태국은 `::with_region(..)` | `with_credentials(access_key, secret_key)` |
| Bithumb | `BithumbAdapter::new()` | `with_credentials(access_key, secret_key)` |
| Binance | `BinanceAdapter::spot()` 또는 `::usd_m_futures()` | `with_credentials(api_key, secret_key)` |
| Hyperliquid | `HyperliquidAdapter::new()`, 또는 `::testnet()` | `with_wallet(address, private_key)`, 계정 키보다 [승인된 API wallet 키](providers/hyperliquid.ko.md#인증-정보)를 쓰세요 |

인증 정보를 넣기 전까지 `Client::supports`는 모든 계좌 기능에 `false`로
답합니다.

## 설계를 바꾸는 차이

| 어댑터 | 차이 |
| --- | --- |
| Upbit | 상장된 파생상품이 없어서 포지션, 마진, 펀딩 비율, 펀딩 지급, 레버리지 설정, reduce-only 주문은 `Error::Unsupported`입니다. 게다가 [별개의 거래소 넷](providers/upbit.ko.md#지원-범위)이라 어댑터 하나가 그중 한 곳만 봅니다. 한국, 싱가포르, 인도네시아, 태국은 상장 목록도 호가창도 인증 정보도 각각입니다 |
| Bithumb | 상장된 파생상품이 없는 것은 Upbit과 같습니다. 여기에 [캔들 스트림도 없습니다](providers/bithumb.ko.md#스트림). `Feed::Candles(_)`가 든 구독은 소켓을 열기 전에 통째로 `Error::Unsupported`로 실패합니다 |
| Binance | 현물과 USD-M 선물, [거래 시장 설정이 둘](providers/binance.ko.md#거래-시장)이고 생성할 때 정해집니다. 호스트도 잔고도 상장 목록도 따로이고 `BTCUSDT`는 양쪽에 서로 다른 가격으로 있습니다 |
| Hyperliquid | 설정 하나가 현물과 무기한 선물을 함께 다루고 그 구분은 `Market::kind`가 집니다. 그래서 파생상품 기능은 어댑터 차원에서 지원한다고 나오면서 마켓별로 거절합니다. Hyperliquid *현물* 마켓의 펀딩, 포지션, reduce-only 주문은 `Error::Unsupported`입니다 |

## 이제는 다르지 않은 것

| 항목 | 네 거래소 공통 |
| --- | --- |
| 캔들 | 오래된 순입니다. `CandleRequest::from`은 넷 모두에서 지켜지고 `limit`은 응답당 상한을 넘어서도 최대 100페이지까지 넘겨 가며 지켜집니다 |
| 캔들 간격 | `supports(Feature::Candles) == true`는 `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1`이 REST에서 모두 동작한다는 뜻입니다. 그 밖은 거래소마다 다르고 스트림이 싣는 집합은 또 다릅니다 |
| 최근 체결 | 이를 제공하는 모든 어댑터에서 최신 순입니다 |

`Client::supports`가 답하는 단위는 기능이지 인자가 아니어서 `true`가 나와도
호출 지점에서 거절될 수 있습니다.
[공통 API](common-api.ko.md#true도-호출-지점에서-다시-확인해야-합니다)를 보세요.

## 거래소별 페이지

- [Upbit](providers/upbit.ko.md) ([English](providers/upbit.md))
- [Bithumb](providers/bithumb.ko.md) ([English](providers/bithumb.md))
- [Binance](providers/binance.ko.md) ([English](providers/binance.md))
- [Hyperliquid](providers/hyperliquid.ko.md) ([English](providers/hyperliquid.md))
