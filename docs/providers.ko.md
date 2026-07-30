[English](providers.md) | [한국어](providers.ko.md)

# 거래소 고르기

어댑터는 넷입니다. 이 페이지는 그중 하나를 고릅니다. 호출당 상한, 받아들이는
주문 형태, 공통 API에 자리가 없는 고유 기능은 각 거래소 페이지에 있습니다.

## 어떤 일에 어떤 어댑터

| 필요한 것 | 어댑터 |
| --- | --- |
| 원화 마켓 | [Upbit](providers/upbit.ko.md) 또는 [Bithumb](providers/bithumb.ko.md), 둘 다 현물 전용 |
| 글로벌 현물 | [Binance](providers/binance.ko.md), `BinanceAdapter::spot()` |
| 무기한 선물 | [Binance USD-M](providers/binance.ko.md), `BinanceAdapter::usd_m_futures()`, 또는 [Hyperliquid](providers/hyperliquid.ko.md) |
| API 키 대신 지갑 서명 | [Hyperliquid](providers/hyperliquid.ko.md) 하나뿐 |
| 테스트 네트워크 | [Hyperliquid](providers/hyperliquid.ko.md), `HyperliquidAdapter::testnet()`. 나머지 셋은 여기에 테스트 환경이 없습니다. |

공개 시세는 넷 모두 계정 없이 동작합니다. API의 파생상품 절반, 곧 포지션과 마진과
펀딩과 레버리지는 [공통 API](common-api.ko.md#파생상품-읽기-예제)에서 처음부터
끝까지 훑습니다. Upbit과 Bithumb에서는 그 호출이 하나같이
`Error::Unsupported`입니다.

## 생성자와 인증 정보

| 어댑터 | 만드는 법 | 인증 정보 |
| --- | --- | --- |
| Upbit | `UpbitAdapter::new()`, 싱가포르·인도네시아·태국은 `::with_region(..)` | `with_credentials(access_key, secret_key)` |
| Bithumb | `BithumbAdapter::new()` | `with_credentials(access_key, secret_key)` |
| Binance | `BinanceAdapter::spot()` 또는 `::usd_m_futures()` | `with_credentials(api_key, secret_key)` |
| Hyperliquid | `HyperliquidAdapter::new()`, 또는 `::testnet()` | `with_wallet(address, private_key)` |

`maxt`는 이 넷을 하나의 인증 타입으로 합치지 않습니다. 애초에 하나가 아니기
때문입니다. Upbit과 Bithumb은 키 한 쌍으로 서명합니다. Binance는 API 키를 보내고
시크릿으로 서명합니다. Hyperliquid는 각 요청을 로컬에서 지갑 키로 서명하고 키는
아예 보내지 않는데, 여기서는 승인된 API 지갑 키가 낫습니다. 같은 동작에
서명하면서 출금은 못 합니다.

인증 정보를 넣기 전까지 `Client::supports`는 모든 계좌 기능에 `false`로
답합니다.

## 설계를 바꾸는 차이

| 어댑터 | 무엇이 달라지는가 |
| --- | --- |
| Upbit, Bithumb | 상장된 파생상품이 없습니다. 포지션, 마진, 펀딩 비율, 펀딩 지급, 레버리지 설정, reduce-only 주문은 `Error::Unsupported`입니다. |
| Upbit | 별개의 거래소 넷입니다. 한국, 싱가포르, 인도네시아, 태국은 상장 목록도 호가창도 인증 정보도 각각입니다. 어댑터 하나는 정확히 한 지역과 통신하고 지역은 `UpbitAdapter::with_region`으로 정하며 한 지역에서 발급한 인증 정보는 다른 지역에서 통하지 않습니다. |
| Bithumb | 캔들 스트림이 없습니다. `Feed::Candles(_)`가 들어 있는 구독은 소켓을 열기 전에 통째로 `Error::Unsupported`로 실패합니다. 피드 목록에서 조용히 빠지지도 않고 체결로 캔들을 대신 만들어 주지도 않습니다. 캔들은 REST로 주기적으로 읽거나 `Feed::Trades`를 직접 집계하세요. |
| Binance | 어댑터가 둘입니다. 현물과 USD-M 선물은 호스트도 잔고도 상장 목록도 분리된 별개의 API이고 `BTCUSDT`는 양쪽에 서로 다른 가격으로 있습니다. 어느 거래 시장인지는 생성 시점에 정해지고 종류가 다른 마켓은 네트워크에 닿기 전에 거절됩니다. |
| Hyperliquid | 어댑터 하나가 현물과 무기한 선물을 함께 다루고 그 구분은 `Market::kind`가 지고 갑니다. 그래서 파생상품 기능은 어댑터에서 지원한다고 읽히면서 마켓별로 거절합니다. Hyperliquid *현물* 마켓의 펀딩, 포지션, reduce-only 주문은 `Error::Unsupported`입니다. |
| Hyperliquid | `trades`는 최근 10건까지만 읽습니다. `recentTrades`는 개수를 받지 않아 10건이 창의 전부이고, 그보다 넓은 공백은 REST로 되읽을 수 없습니다. 연속된 기록은 `Feed::Trades`에서 옵니다. |

## 이제는 다르지 않은 것

예전에는 거래소마다 달랐지만 지금은 어디서나 같은 것이 셋입니다. 자세한 내용은
[공통 API](common-api.ko.md)에 있습니다.

| 항목 | 어디서나 같은 것 |
| --- | --- |
| 캔들 | 계약 하나입니다. 결과는 오래된 순입니다. `CandleRequest::from`은 넷 모두에서 지켜지고 `limit`은 응답당 상한을 넘어서도 최대 100페이지까지 페이지를 넘겨 가며 지켜집니다. 어떤 어댑터도 `from`을 `Error::Unsupported`로 보고하지 않으며 커서를 직접 걸어가게 만들지도 않습니다. |
| 캔들 간격 | 열 개가 보장됩니다. `supports(Feature::Candles) == true`는 `Min1`, `Min3`, `Min5`, `Min15`, `Min30`, `Hour1`, `Hour4`, `Day1`, `Week1`, `Month1`이 REST에서 모두 동작한다는 뜻입니다. 그 밖은 거래소마다 다르고 스트림이 싣는 집합은 또 다릅니다. |
| 최근 체결 | 이를 제공하는 모든 어댑터에서 최신 순입니다. |

나머지는 거래소마다 다릅니다. 호출당 상한, 각 거래소가 기준선 너머로 더하는
간격과 그 거래소의 스트림이 싣는 또 다른 집합, 받아들이는 주문 형태, 어떤
타임스탬프가 거래소 자신의 것인지, 실시간 호가창 피드가 몇 단계까지 내려가는지.
고른 어댑터의 페이지는 코드를 쓰기 전에 읽으세요. 각 페이지가 자신의 공백을 먼저
밝힙니다.

어느 페이지도 대신 정리해 줄 수 없는 것이 하나 있습니다. `Client::supports`는
기능 단위로 답하고 인자 단위로는 답하지 않습니다. Upbit에서
`Feature::CandleStream`은 `true`인데 `Feed::Candles(Interval::Day1)`은 여전히
`Error::Unsupported`입니다. Upbit이 일봉을 스트림으로 발행하지 않기 때문입니다.
기능으로 분기하는 것과 별도로 호출 지점에서 에러를 처리하고 나머지는
[공통 API](common-api.ko.md#feature와-clientsupports)에서 보세요.

## 거래소별 페이지

- [Upbit](providers/upbit.ko.md) ([English](providers/upbit.md))
- [Bithumb](providers/bithumb.ko.md) ([English](providers/bithumb.md))
- [Binance](providers/binance.ko.md) ([English](providers/binance.md))
- [Hyperliquid](providers/hyperliquid.ko.md) ([English](providers/hyperliquid.md))
