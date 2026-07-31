[English](binance.md) | [한국어](binance.ko.md)

# Binance

현물과 USD 마진 무기한 선물. `BinanceAdapter` 하나는 생성할 때 고른 거래 시장 한
곳과 통신합니다.

```rust
use maxt::{Client, adapters::BinanceAdapter};

let spot = Client::new(BinanceAdapter::spot());
let perp = Client::new(BinanceAdapter::usd_m_futures());
```

## 거래 시장

`BinanceAdapter::default()`는 현물입니다. `BTCUSDT`는 양쪽에 다 있고 가격은 서로
다릅니다. 종류가 맞지 않는 `Market`은 네트워크로 나가기 전에
`Error::InvalidRequest`가 됩니다. `supports`는 마켓이 아니라 어댑터를 두고
답하므로 이 실수를 잡지 못합니다.

| 항목 | 현물 | USD-M 선물 |
| --- | --- | --- |
| `Market` 생성자 | `Market::spot` | `Market::perpetual` |
| 포지션·마진·펀딩·reduce-only 주문 | `Error::Unsupported` | 지원 |
| 캔들 간격 | 열다섯 개 전부. [기준선](../common-api.ko.md#간격) 열 개에 `Sec1`, `Hour2`, `Hour8`, `Hour12`, `Day3`를 더한 것 | 열네 개. 같은 목록에서 `Sec1`을 뺀 나머지 |
| 응답당 캔들 수 | 1,000개 | 1,500개 |
| REST 호가창 깊이 | 5, 10, 20, 50, 100, 500, 1,000, 5,000 | 5,000을 뺀 나머지 |
| 실시간 체결 | `@trade`, 체결 하나하나 | `@trade`, 체결 하나하나. 여기에 [소진된 체결 식별자](#스트림)마다 가격이 0인 프레임이 섞여 오는데 `maxt`가 버립니다 |
| 호가창 스냅숏 시각 | 읽은 시각 | Binance가 찍은 시각 |
| 시장가 주문 크기 단위 | `Size::Base` 또는 `Size::Quote` | `Size::Base` |
| post-only | `LIMIT_MAKER` 주문 타입 | `GTX` time in force |
| 가중치 예산 | IP당 분당 6,000 | IP당 분당 2,400 |

간격 매핑 하나가 REST와 `Feed::Candles`를 함께 맡으므로 어떤 간격이든 양쪽에서
쓰이거나 양쪽 다 쓰이지 않습니다. 두 시장의 차이는 `Sec1` 하나뿐입니다.
USD-M에서 `Sec1`은 `Feature::Candles`를 지목하는 `Error::Unsupported`입니다.

아래의 `limit`·구간 검사가 간격 조회보다 먼저 돌기 때문에, USD-M에 `Sec1`을
`limit` 0이나 뒤집힌 구간, 페이지 상한을 넘는 범위와 함께 요청하면
`Unsupported`가 아니라 해당 필드를 지목하는 `Error::InvalidRequest`가 돌아옵니다.
둘 다 매칭하거나, 간격으로 분기하기 전에 요청을 먼저 검증하세요. 공통 API가
[`supports`가 답한 `true`도 호출 지점에서 다시 확인해야 하는 이유](../common-api.ko.md#true도-호출-지점에서-다시-확인해야-합니다)를
설명하며 드는 실사례가 바로 이것입니다.

## 상한

요청을 만들기 전에 검사합니다. 범위를 벗어나면 해당 필드를 지목하는
`Error::InvalidRequest`입니다.

| 호출 | 허용 범위 |
| --- | --- |
| `trades` | `limit` 1~1,000 |
| `order_book` | 위 표에 있는 깊이. 목록에 없는 깊이는 반올림하지 않고 거절합니다 |
| `candles` | `limit`에 제한이 없고 최대 100번까지 호출하며 페이지를 대신 넘깁니다. 상한은 각 시장이 호출당 주는 양으로 100페이지입니다. `limit` 0은 `limit`을, 현물에서 캔들 100,000개나 USD-M에서 150,000개를 넘는 구간은 `from`을 지목하며, 둘 다 첫 호출 전에 걸러집니다 |
| `funding_rates`, `funding_payments` | `limit` 1~1,000, 기본값 100 |
| `limit` 없는 `HistoryRequest` | 100건. 이력의 끝이 아니라 페이지 하나입니다. `Page::next`가 `None`이 될 때까지 따라가세요 |

## 주문 정밀도와 최소 주문 크기

심볼마다 `exchangeInfo` 필터에 실립니다.

| 필터 | 담는 값 |
| --- | --- |
| `PRICE_FILTER` | 호가 단위 |
| `LOT_SIZE` | 수량 단위 |
| `NOTIONAL` | 받아들이는 가격 곱하기 수량의 최솟값 |

`maxt`는 주문을 이 값들과 대조하지도, 값에 맞춰 반올림하지도 않습니다. 그래서
호가 단위에서 벗어난 가격은 여기서 나오는 `Error::InvalidRequest`가 아니라
Binance의 거절로 돌아옵니다. 현물에서는
[`spot_symbol_filters`](#binance-전용-호출)가 대신 읽어 줍니다. USD-M 쪽 값은
`maxt` 어디에도 없으니
[`exchangeInfo`](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Exchange-Information)를
직접 읽으세요.

## 스트림

| 대상 | 동작 |
| --- | --- |
| `Feed::OrderBook` | `depth20@100ms`. 20단계, 100ms, 두 시장 공통, 변경 불가 |
| 호가창 이벤트 | 차분이 아니라 전체 스냅숏. 사본을 덮어쓰세요 |
| 더 깊은 호가창 | REST의 `Client::order_book`, 현물 5,000단계, USD-M 1,000단계 |
| `Feed::Trades` | 두 시장 모두 `@trade`. 체결 하나에 이벤트 하나 |
| `Trade::id` | Binance의 체결 식별자. 두 시장과 두 전송 경로에서 같은 번호 |
| 가격과 수량이 0인 USD-M `@trade` 프레임 | Binance가 체결을 싣지 않고 소진한 체결 식별자입니다. `maxt`가 버리므로 가격 0짜리 `Trade`는 전달되지 않습니다 |
| USD-M 엔드포인트 | 피드에 따라 갈리는 두 개입니다. [USD-M의 두 진입점](#usd-m의-두-진입점)을 보세요 |

### REST와 스트림 대조하기

`Trade::id` 집합 하나면 두 시장 모두에서 REST 보충분과 실시간 구독의 중복이
걸러집니다. 대신 쓸 튜플도 없고 필요하지도 않습니다.

### `@aggTrade`를 쓰지 않는 이유

`@aggTrade`는 테이커 주문 하나가 한 가격에서 쓸어 담은 체결들을 메시지 하나로
묶습니다. `maxt`도 아래 `/market` 진입점에서 이 스트림을 싣습니다. 다만 구독은
`@trade`로 합니다.

| 걸림돌 | 내용 |
| --- | --- |
| 두 번째 식별자 공간 | `aggTrade`의 식별자는 체결이 아니라 묶음에 번호를 매기며 두 시장 어느 REST 호출도 그 번호를 돌려주지 않습니다. 두 전송 경로를 함께 쓰면 맞출 기준이 없습니다 |
| 대신 쓸 튜플 없음 | 수량이 묶인 체결들의 합이라 `(timestamp, price, quantity, taker_side)`로도 REST와 맞춰지지 않습니다 |

### USD-M의 두 진입점

Binance는 USD-M 시장 데이터를 한 호스트의 진입점 두 개로 나눠 보냅니다. 둘 중
어느 쪽도 지정하지 않은 소켓은 `/public`을 지정한 것처럼 처리됩니다.

| 진입점 | 스트림 | `maxt` 구독 |
| --- | --- | --- |
| `wss://fstream.binance.com/public/stream` | 체결 엔진이 변화 때마다 밀어 주는 것. `@trade`, `@depth*`, `@bookTicker` | `Feed::Trades`, `Feed::OrderBook` |
| `wss://fstream.binance.com/market/stream` | 집계 서비스가 만들어 내는 것. `@aggTrade`, `@kline_*`, `@ticker`, `@miniTicker`, `@markPrice`, `@forceOrder`, `@compositeIndex`, `!contractInfo`, `!assetIndex@arr` | `Feed::Ticker`, `Feed::Candles` |
| `wss://fstream.binance.com/private/ws` | 계정. `ORDER_TRADE_UPDATE`, `ACCOUNT_UPDATE`, `listenKeyExpired` | `subscribe_account` |

어긋난 요청을 거절하는 장치는 없습니다.

| 구독 대상 | 응답 |
| --- | --- |
| 다른 진입점에 속한 스트림 | `{"result": null, "id": 1}`, 그리고 그 스트림의 프레임은 한 개도 오지 않습니다 |
| Binance가 아예 발행하지 않는 스트림 이름 | 같은 응답이 오므로, 이 응답만으로는 데이터가 뒤따를지 알 수 없습니다 |
| 두 진입점에 걸친 피드 | 소켓 두 개가 `MarketStream` 하나로 합쳐집니다. 각 소켓이 따로 재접속하므로 `MarketEvent::Reconnected`는 장애 한 번에 한 번이 아니라 되살아난 소켓마다 한 번씩 옵니다 |

현물 시장 데이터는 나뉘어 있지 않습니다. `wss://stream.binance.com:9443/stream`
하나가 모든 피드를 싣습니다.

USD-M의 `subscribe_account`는
`wss://fstream.binance.com/private/ws?listenKey=<키>&events=ORDER_TRADE_UPDATE/ACCOUNT_UPDATE/listenKeyExpired`
를 엽니다.

**`events`는 참고 사항이 아니라 허용 목록입니다.** 소켓은 필터에 적힌 이벤트만
받습니다. 그 밖의 것은 오지 않습니다.

| `events` 필터 | `listenKeyExpired` |
| --- | --- |
| 없음 | 받음 |
| 이름을 적음. `maxt`가 보내는 형태 | 받음 |
| `ORDER_TRADE_UPDATE/ACCOUNT_UPDATE` | **못 받음** |
| `wss://fstream.binance.com/ws/<키>`는 필터와 무관하게 | **못 받음**. 이 경로는 사용자 데이터를 싣지 않습니다 |

USD-M이 발행하는 이벤트 전부와, 필터가 그 이름을 적는지 여부입니다.

| 이벤트 | 필터에 있음 | 전달 형태 |
| --- | --- | --- |
| `ORDER_TRADE_UPDATE` | 예 | `AccountEvent::Order` |
| `ACCOUNT_UPDATE` | 예 | `AccountEvent::Balance` |
| `listenKeyExpired` | 예 | `Error::Exchange`. 더 할 말이 없는 스트림을 계속 기다리지 않게 합니다 |
| `TRADE_LITE` | 아니요 | `ORDER_TRADE_UPDATE`가 이미 싣는 같은 체결을 더 일찍, 더 적은 필드로 보냅니다 |
| `MARGIN_CALL` | 아니요 | |
| `ACCOUNT_CONFIG_UPDATE` | 아니요 | |
| `CONDITIONAL_ORDER_TRIGGER_REJECT` | 아니요 | |
| `STRATEGY_UPDATE` | 아니요 | |
| `GRID_UPDATE` | 아니요 | |
| `ALGO_UPDATE` | 아니요 | |

필터가 이름을 적지 않은 이벤트는 오지 않고, 그 이벤트들은 `maxt`가 다루지도
않습니다. `eventStreamTerminated`는 USD-M이 발행하지 않아 표에 없습니다. 이
이벤트는 WebSocket API 세션을 끝내는데, 그 세션은 현물 소켓에만 있습니다.

`ACCOUNT_UPDATE`는 잔고나 포지션이 실제로 바뀔 때 오는데, 걸려 있는 주문은 둘 다
바꾸지 않습니다. 그 주문이 묶어 둔 증거금은 `ORDER_TRADE_UPDATE`의 `b` 필드, 곧
잡아 둔 매수 주문 금액에 실리고 `maxt`는 이를 노출하지 않습니다. `balances()`로
읽으세요.

### 현물 계정 스트림

현물에는 listen key가 없습니다. 현물 어댑터의 `subscribe_account`는
`wss://ws-api.binance.com:443/ws-api/v3`를 열고 서명된
`userDataStream.subscribe.signature` 하나를 보냅니다. 소켓은 인증 없이 열리고 그
프레임이 계정을 지목하므로 URL에 비밀이 들어가지 않고 살려 둘 키도 없습니다.
이쪽에는 이벤트를 거르는 장치가 없습니다. 현물 소켓은 계정이 만들어 내는 것을
모두 받고 `maxt`가 다루지 않는 것은 받은 뒤에 버립니다.

| 방법 | HMAC-SHA-256 키 | Ed25519 키 |
| --- | --- | --- |
| 요청마다 서명하는 `userDataStream.subscribe.signature`. `maxt`가 보내는 방법 | 됩니다 | 됩니다 |
| `session.logon` 뒤 `userDataStream.subscribe` | `-2028 HMAC-SHA-256 API key is not supported`, 이어서 `-1193 WebSocket session not authenticated` | 됩니다 |

**재연결은 프레임을 새로 서명합니다.** 구독 프레임은 만들어진 밀리초 시각을
서명에 담으므로 서명 하나가 소켓 하나만 구독합니다. `recvWindow`가 지난 뒤 다시
보낸 프레임에는 `-1021`이 돌아옵니다. `maxt`는 핸드셰이크마다 다시 서명하고
`recvWindow`로 Binance가 문서로 밝힌 최댓값 60,000 ms를 보냅니다. 이 값은
한계값이지 우회로가 아닙니다.

Binance가 거절한 구독은 열린 채 조용한 소켓이 아니라 Binance의 코드를 담은
스트림에 실린 `Error::Exchange`입니다. 이 오류가 나오면 다시 구독하세요.

### 아무것도 오지 않는 스트림

이벤트도 오류도 오지 않는 피드는 아무 일도 일어나지 않는 시장과 구별되지
않습니다. `maxt`에는 타이머도, 구독 응답도, 피드별 생존 신호도 없습니다.

| 의심 지점 | 확인 방법 |
| --- | --- |
| 피드 | 같은 대상을 REST로 물어보세요. `Client::ticker`와 `Client::candles`는 모든 USD-M 마켓에 응답하므로 스트림은 조용한데 REST가 답한다면 문제는 마켓이 아니라 스트림입니다 |
| 소켓 | `StreamConfig::idle_timeout_ms`를 설정하세요. 그 시간 동안 아무것도 보내지 않은 소켓은 끊고 다시 열리며 그 사실은 `MarketEvent::Reconnected`로 도착합니다 |
| 진입점 | 같은 스트림 이름을 다른 진입점에 raw 소켓으로 구독해 프레임을 세어 보세요 |

## 요청 할당량

Binance는 요청 수가 아니라 **IP당 분당 가중치**로 예산을 잡고 시장별로 따로
셉니다. 깊은 호가창은 티커보다 훨씬 비쌉니다. 모든 응답이 누적값을
`X-MBX-USED-WEIGHT-1M`에 실어 보냅니다. 각 시장의 상한은 `exchangeInfo` 응답의
`rateLimits` 배열에 1분 구간짜리 `REQUEST_WEIGHT` 항목으로 실립니다. 위 표의
6,000과 2,400도 여기서 읽은 값입니다.

`maxt`는 속도를 조절하지도, 그 헤더를 읽지도 않습니다. 예산을 넘기면 HTTP
429이고 `Error::is_rate_limited()`가 알려 줍니다. 429를 무시하면 2분에서 3일까지
자동으로 IP가 차단되니 첫 번째에서 물러서세요.

## 유령 포지션

`/fapi/v3/positionRisk`는 주문 하나만 걸려 있어도 그 심볼에 수량 0짜리 행을
만듭니다. `positions()`와 `positions_on(&market)`은 보유 포지션만 돌려주고,
크기가 없는 행은 포지션이 아니므로 `maxt`가 버립니다.

| 항목 | 동작 |
| --- | --- |
| 생기는 조건 | 그 심볼의 미체결 주문뿐이고 그 밖에는 없습니다. 빈 계정에서는 이 행이 나오지 않습니다 |
| 버리는 지점 | 이 어댑터가 아니라 공통 API입니다. 아래에 어떤 어댑터가 있든 크기가 0인 행은 모두 버리므로 이 보장은 거래소마다 하나가 아니라 필터 하나입니다. Hyperliquid는 닫힌 포지션을 거래소 단에서 `assetPositions`에 싣지 않아 결과가 같습니다 |
| 버린 행 | 어디에도 남기지 않습니다. 공개 정보인 마크 가격 말고 그 행이 하던 말은 그 심볼에 주문이 걸려 있다는 것뿐이고 `open_orders()`가 그것을 그대로 알려 줍니다 |
| `maxt`가 해석하지 못한 행 | 그대로 `Error::Decode`입니다. 해석에 성공하고 크기가 0인 행만 버립니다 |

## 주의할 점

| 필드 또는 호출 | 동작 |
| --- | --- |
| `Ticker::last_trade_time` | 언제나 `None`. 마지막 가격이 언제 체결됐는지 Binance는 밝히지 않습니다 |
| `Ticker::timestamp` | 체결 시각이 아니라 24시간 구간의 끝 |
| 현물 호가창 타임스탬프 | 읽은 시각. Binance는 현물 depth에 시계를 싣지 않습니다 |
| `Position::leverage`, `margin_mode` | `None`. `/fapi/v3/positionRisk`는 둘 다 싣지 않습니다. 심볼에 설정된 레버리지와 마진 모드는 `/fapi/v1/symbolConfig`에 있고 가중치도 같습니다 |
| 주문만 걸려 있는 심볼 | 포지션이 아닙니다. Binance는 하나로 보고하고 `maxt`는 버립니다. [유령 포지션](#유령-포지션) 참고 |
| `FundingPayment::rate` | `None`. 원장은 비율이 아니라 청구액을 기록합니다 |
| `MarginSummary::equity` | `totalMarginBalance`. 지갑 잔고에 미실현 손익을 더한 값 |
| `MarginSummary::margin_balance` | `totalInitialMargin`. 보유 포지션과 주문이 이미 잡아먹은 증거금입니다. 예산이 아니라 비용입니다 |
| `MarginSummary::available_balance` | `availableBalance`. 새로 열 때 쓸 수 있는 금액이고 셋 중 여유를 좌우하는 유일한 값입니다 |
| 마진 수치 셋 모두 | `USDT` 기준 |
| USD-M의 `Balance::locked` | 지갑 잔고에서 가용 잔고를 뺀 값, 0에서 멈춤 |
| 스트림 주문의 `created_at` | 현물은 생성 시각 `O`로 매깁니다. USD-M은 `ORDER_TRADE_UPDATE`가 생성 시각을 싣지 않아 `T`로 매기므로 걸려 있다가 나중에 체결된 USD-M 주문에서는 체결 시각이 들어갑니다 |
| `cancel_order`, `spot_order` | Binance가 발급한 숫자 주문 식별자만. 사용자 지정 식별자는 `Error::InvalidRequest` |
| `set_margin` | 필드마다 하나씩 최대 두 번의 호출이고 원자적이지 않습니다. 레버리지는 1 이상의 정수 배수여야 합니다 |
| 만기가 있는 선물 | `markets()`에서 제외. 무기한으로 보고하면 가격을 잘못 매깁니다 |
| 스트림 프레임에 모르는 quote 자산 | 엉뚱한 마켓 대신 `Error::Decode` |
| 인증 정보 없음 | `Error::Unsupported`가 아니라 `Error::Auth` |
| Binance가 거부한 인증 정보 | `Error::Auth`가 아니라 `Error::Exchange`. 서명이 틀리면 HTTP 400 `-1022`, 키가 틀렸거나 권한이 없으면 HTTP 401 `-2015`, 키가 없으면 HTTP 401 `-2014` |
| `recvWindow`를 벗어난 시각 `-1021` | `ExchangeErrorKind::Rejected`라서 `is_retryable()`은 `false`입니다. 시계를 맞추거나, 요청을 다시 만들어 한 번 보내세요 |
| 현물 계정 스트림 | listen key 없이 WebSocket API에 서명된 요청 하나이고, 재연결마다 다시 서명합니다. [현물 계정 스트림](#현물-계정-스트림) 참고 |
| Binance가 끝낸 현물 구독 | `eventStreamTerminated`를 코드로 담은 `Error::Exchange`. 현물에는 리슨 키가 없으므로 `listenKeyExpired`는 USD-M에만 옵니다 |
| USD-M listen key | 30분마다 REST로 연장하며 실패하면 Binance의 판정이 그대로 스트림에 전달됩니다. 그대로 두면 한 시간 안에 계정 변화를 싣지 않게 됩니다 |

## Binance 전용 호출

`Client::adapter()`를 통해 호출합니다.

| 메서드 | 결과 | 없는 시장 |
| --- | --- | --- |
| `spot_symbol_filters(&market)` | 호가 단위, 가격과 수량의 한계, 수량 단위, 최소 주문 금액 | USD-M |
| `spot_order(&market, id)` | 식별자로 주문 하나, 체결 완료와 취소까지 | USD-M |
| `usd_m_create_listen_key()` | USD-M 사용자 데이터 스트림 키 | 현물 |
| `usd_m_keepalive_listen_key(&key)` | 키를 60분 더 연장 | 현물 |
| `usd_m_close_listen_key(&key)` | 키를 닫습니다 | 현물 |

필터는 Binance의 모양을 그대로 유지합니다. 해당 종류의 필터가 없는 심볼에서
필드는 `None`이며 갓 상장된 페어에서는 흔합니다.

USD-M listen key의 수명은 `subscribe_account`가 대신 관리하므로 키 관련 호출
셋은 소켓을 직접 다루거나 키 하나를 여러 소비자가 나눠 쓰거나 재시작을 넘겨 키를
유지할 때 씁니다. `BinanceListenKey`는 소켓 URL에 들어가는 bearer 비밀값이라
`Debug` 출력에서 가려집니다.

```rust
use maxt::{Client, Exchange, Market, adapters::BinanceAdapter};

async fn tick_size() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::spot());
    let filters = client
        .adapter()
        .spot_symbol_filters(&Market::spot(Exchange::Binance, "BTC", "USDT"))
        .await?;

    if let Some(tick) = filters.tick_size {
        println!("prices move in steps of {tick}");
    }
    Ok(())
}
```

## 인증 정보

API key와 secret key입니다. 현물로 제한된 키는 선물 API에서 거절되고 반대도
마찬가지입니다.

```rust
use maxt::{Client, adapters::BinanceAdapter};

fn client() -> Client<BinanceAdapter> {
    let api_key = std::env::var("BINANCE_API_KEY").expect("BINANCE_API_KEY");
    let secret = std::env::var("BINANCE_SECRET_KEY").expect("BINANCE_SECRET_KEY");
    Client::new(BinanceAdapter::usd_m_futures().with_credentials(api_key, secret))
}
```

secret key는 프로세스 밖으로 나가지 않습니다. 각 요청의 쿼리 문자열을
HMAC-SHA256으로 서명하고 서명만 실어 보냅니다.

이 어댑터에는 테스트넷 호스트가 없습니다. 거래 권한을 끈 키로 시험하세요.

## 예제

`cargo run --example public_rest -- binance BTC USDT`

- [`public_rest.rs`](../../examples/public_rest.rs)
- [`public_stream.rs`](../../examples/public_stream.rs)
- [`private_account.rs`](../../examples/private_account.rs)
- [`private_stream.rs`](../../examples/private_stream.rs)

## Binance 공식 문서

| 주제 | 문서 |
| --- | --- |
| 할당량 | [현물 한도](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits) · [USD-M 일반 정보](https://developers.binance.com/docs/derivatives/usds-margined-futures/general-info) |
| 주문 규칙 | [USD-M `exchangeInfo`](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Exchange-Information) |
| 현물 | [REST](https://developers.binance.com/docs/binance-spot-api-docs/rest-api) · [WebSocket 스트림](https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams) |
| 현물 계정 스트림 | [사용자 데이터 스트림](https://developers.binance.com/docs/binance-spot-api-docs/user-data-stream) · [구독 요청](https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/user-data-stream-requests) · [요청에 서명하는 법](https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/request-security) |
| USD-M 선물 | [REST](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api) · [WebSocket 스트림](https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams) |

---

[공통 API](../common-api.ko.md) · [거래소 고르기](../providers.ko.md)
