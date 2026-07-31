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

`BTCUSDT`는 양쪽에 서로 다른 가격으로 존재합니다. 종류가 맞지 않는 `Market`을
넘기면 네트워크에 닿기 전에 `Error::InvalidRequest`입니다. `supports`는 마켓이
아니라 어댑터를 두고 답하므로 이 실수를 잡지 못합니다.

| 다른 점 | 현물 | USD-M 선물 |
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
| 스트림 주문의 `created_at` | 주문이 만들어진 시각 `O` | 이벤트 자신의 시각 `T` |
| 가중치 예산 | IP당 분당 6,000 | IP당 분당 2,400 |

매핑 하나가 양방향을 함께 담당하므로 어떤 간격은 REST와 `Feed::Candles`
*양쪽*에서 닿거나 양쪽 어디에서도 닿지 않습니다. 두 시장의 차이는 `Sec1`
하나뿐입니다. USD-M은 `Feature::Candles`를 지목하는 `Error::Unsupported`로 1초
캔들을 거절하고 `Interval`이 이름 붙일 수 있는 나머지 간격은 모두 제공합니다.

**단 요청이 나머지 면에서 올바를 때만 그렇습니다.** 아래의 `limit`·구간 검사가
간격 조회보다 먼저 돌기 때문에, USD-M에 `Sec1`을 `limit` 0이나 뒤집힌 구간,
페이지 상한을 넘는 범위와 함께 요청하면 `Unsupported`가 아니라 해당 필드를
지목하는 `Error::InvalidRequest`가 돌아옵니다. `Unsupported`만 매칭해 다른
거래소로 넘어가는 코드는 그대로 흘러내립니다. 둘 다 매칭하거나, 간격으로 분기하기
전에 요청을 먼저 검증하세요. Binance USD-M의 `Sec1`은
[`supports`가 답한 `true`도 호출 지점에서 다시 확인해야 하는 이유](../common-api.ko.md#true도-호출-지점에서-다시-확인해야-합니다)로
공통 API가 드는 유일한 실사례이므로, 라우터가 가장 먼저 부딪히는 경우입니다.

`BinanceAdapter::default()`는 현물입니다.

## 상한

요청을 만들기 전에 검사합니다.

| 호출 | 허용 범위 | 벗어나면 |
| --- | --- | --- |
| `trades` | `limit` 1~1,000 | `Error::InvalidRequest` |
| `order_book` | 위 표의 깊이 중 하나 | `Error::InvalidRequest` |
| `candles` | `limit`은 제한 없음. 최대 100번의 호출까지 페이지를 대신 넘깁니다 | `limit` 0, 또는 현물에서 캔들 100,000개와 USD-M에서 150,000개를 넘는 구간은 `Error::InvalidRequest` |
| `funding_rates`, `funding_payments` | `limit` 1~1,000, **기본값 100** | `Error::InvalidRequest` |

Binance가 제공하지 않는 깊이는 반올림하지 않고 거절합니다.

캔들 상한은 각 시장이 호출당 주는 양의 100페이지입니다. 그보다 더 거슬러
올라가는 `from`을 `limit` 없이 주면 `from`을 지목하는 `Error::InvalidRequest`이고,
왕복 100번을 돌고 나서 발견하는 대신 첫 호출 전에 올라옵니다.

**이력의 기본값에 걸려 넘어지기 쉽습니다.** `limit` 없는 `HistoryRequest`는
100건을 요청합니다. 페이지를 읽고 100건을 세고서 다 따라잡았다고 결론짓는 루프는
아무것도 결론짓지 못합니다. `Page::next`가 `None`이 될 때까지 따라가세요.

## 주문 정밀도와 최소 주문 크기

Binance는 `exchangeInfo` 목록에 실린 필터로 심볼마다 답합니다. `PRICE_FILTER`가
호가 단위를, `LOT_SIZE`가 수량 단위를, `NOTIONAL`이 받아들이는 가격 곱하기 수량의
최솟값을 담습니다. `maxt`는 주문을 이 값들과 대조하지도, 값에 맞춰 반올림하지도
않으므로, 호가 단위에서 벗어난 가격은 여기서 나오는 `Error::InvalidRequest`가
아니라 Binance의 거절로 돌아옵니다.

현물에서는 [`spot_symbol_filters`](#binance-전용-호출)가 대신 읽어 줍니다.
USD-M에는 그런 호출이 없고 `maxt`의 다른 무엇도 이 숫자를 담지 않으니
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

`Trade::id`는 Binance 자신의 체결 식별자이므로, 두 시장 모두 식별자 집합
하나로 REST 보충분과 실시간 구독의 중복을 제거할 수 있습니다. 대신 쓸 튜플도
없고 필요하지도 않습니다.

2026-07-30에 `wss://fstream.binance.com/stream`의 `btcusdt@trade`를 60초 받아
`https://fapi.binance.com/fapi/v1/trades?symbol=BTCUSDT&limit=1000`과 대조한
결과입니다.

| 측정 항목 | 개수 |
| --- | --- |
| REST 구간 안에 들어온 스트림 체결 | 1,000 |
| 그중 식별자가 REST에 있고 가격·수량·시각·테이커 방향이 모두 같은 것 | 1,000 |
| 그중 식별자가 REST에 없는 것 | 0 |
| 스트림에서 중복된 식별자 | 0 |
| 같은 1분 동안의 소진 식별자 프레임. 그중 REST에 실린 것은 없음 | 1,186개 중 14개 |

### `@aggTrade`를 쓰지 않는 이유

`@aggTrade`는 테이커 주문 하나가 한 가격에서 쓸어 담은 체결들을 메시지 하나로
묶습니다. 위와 같은 체결 1,000건이 1,000개가 아니라 319개 메시지가 됩니다. 그게
이점의 전부이고, `maxt`는 그 이점을 택하지 않습니다.

| 택하지 않는 이유 | |
| --- | --- |
| 두 번째 식별자 공간 | `aggTrade`의 식별자는 체결이 아니라 묶음에 번호를 매기며, 두 시장 어느 REST 호출도 그 번호를 돌려주지 않습니다. 두 전송 경로를 함께 쓰면 맞출 기준이 없습니다 |
| 대신 쓸 튜플이 없음 | `@aggTrade`의 수량은 묶인 체결들의 합이므로 `(timestamp, price, quantity, taker_side)` 튜플로도 REST와 대조되지 않습니다. 같은 체결 1,000건에서 묶음 319개 중 141개가 개별 REST 체결 어느 것과도 맞지 않았습니다. 그중 39%가 체결을 둘 이상 묶었기 때문입니다 |

이 스트림은 실려 있고, 아래의 `/market` 진입점에서는 `BTCUSDT` 기준 25초에
117개 프레임을 보냅니다. 쓰지 않는 이유는 식별자 공간이지 수신 여부가
아닙니다.

### USD-M의 두 진입점

Binance는 USD-M 시장 데이터를 한 호스트의 진입점 두 개로 나눠 보내며, 경로를
지정하지 않는 `/stream`과 `/ws`는 2026-04-23에 폐지했습니다. 둘 중 어느 쪽도
지정하지 않은 소켓은 `/public`을 지정한 것처럼 처리됩니다.

| 진입점 | 싣는 스트림 | `maxt`가 여기로 보내는 것 |
| --- | --- | --- |
| `wss://fstream.binance.com/public/stream` | 체결 엔진이 변화 때마다 밀어 주는 것. `@trade`, `@depth*`, `@bookTicker` | `Feed::Trades`, `Feed::OrderBook` |
| `wss://fstream.binance.com/market/stream` | 집계 서비스가 만들어 내는 것. `@aggTrade`, `@kline_*`, `@ticker`, `@miniTicker`, `@markPrice`, `@forceOrder`, `@compositeIndex`, `!contractInfo`, `!assetIndex@arr` | `Feed::Ticker`, `Feed::Candles` |
| `wss://fstream.binance.com/private/ws` | 계정. `ORDER_TRADE_UPDATE`, `ACCOUNT_UPDATE`, `listenKeyExpired` | `subscribe_account` |

어긋난 요청을 거절하는 장치는 없습니다. 한쪽 진입점의 소켓도 다른 쪽 스트림을
지정한 `SUBSCRIBE`를 받아들이고 `{"result": null, "id": 1}`로 응답한 뒤,
해당 스트림의 프레임을 한 개도 보내지 않습니다. Binance가 아예 발행하지 않는
스트림 이름에도 같은 응답이 오므로, 이 응답만으로는 데이터가 뒤따를지 알 수
없습니다. 폐지된 경로의 소켓이 체결과 호가창은 보내면서 캔들과 티커에는
오류도 종료도 없이 영원히 조용한 이유가 이것입니다.

2026-07-30에 `BTCUSDT`로 25초씩, 엔드포인트마다 일곱 스트림을 모두 지정한
`SUBSCRIBE` 프레임 하나를 보내고 프레임을 각자의 `stream` 이름으로 센
결과입니다.

| 엔드포인트 | `@trade` | `@depth20@100ms` | `@bookTicker` | `@aggTrade` | `@kline_1m` | `@ticker` | `@markPrice@1s` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 폐지된 경로 `/stream` | 141 | 229 | 896 | 0 | 0 | 0 | 0 |
| `/public/stream` | 149 | 229 | 1,993 | 0 | 0 | 0 | 0 |
| `/market/stream` | 0 | 0 | 0 | 117 | 47 | 12 | 25 |

현물 시장 데이터는 나뉘어 있지 않습니다. `wss://stream.binance.com:9443/stream`
하나가 모든 피드를 싣습니다. Binance의 현물 시장 데이터 문서는 진입점을 하나도
언급하지 않고 폐지 공지도 싣지 않았습니다. 옮겨 간 것은 USD-M의 시장 데이터뿐이니
두 거래 시장이 함께 옮겼다고 넘겨짚지 마세요.

USD-M의 계정 소켓도 함께 옮겼습니다. `subscribe_account`는 이제
`wss://fstream.binance.com/private/ws?listenKey=<키>&events=ORDER_TRADE_UPDATE/ACCOUNT_UPDATE/listenKeyExpired`
를 엽니다.

**`events`는 참고 사항이 아니라 허용 목록입니다.** 소켓은 필터에 적힌 이벤트만
받고 그 밖의 것은 받지 않으므로, `maxt`가 처리하면서 적지 않은 이벤트는 `maxt`가
영영 받을 수 없는 이벤트입니다. 2026-07-31에 listen key 하나를 공유한 소켓 넷으로,
`DELETE /fapi/v1/listenKey`를 보내 만료 이벤트를 서버가 밀어내게 하고 잰
결과입니다.

| 소켓의 `events` | `listenKeyExpired` |
| --- | --- |
| `events` 매개변수 없음 | 받음 |
| `listenKeyExpired` | 받음 |
| `maxt`가 보내는 `ORDER_TRADE_UPDATE/ACCOUNT_UPDATE/listenKeyExpired` | 받음 |
| `ORDER_TRADE_UPDATE/ACCOUNT_UPDATE` | **못 받음** |
| 폐지된 `wss://fstream.binance.com/ws/<키>`. 필터 없음 | **못 받음** |

USD-M이 발행하는 이벤트 전부와, `maxt`가 요청하는지 여부입니다.

| 이벤트 | 요청 | 이유 |
| --- | --- | --- |
| `ORDER_TRADE_UPDATE` | 예 | 주문 변화. `AccountEvent::Order`로 읽습니다 |
| `ACCOUNT_UPDATE` | 예 | 잔고 변화. `AccountEvent::Balance`로 읽습니다 |
| `listenKeyExpired` | 예 | 이 소켓의 키가 만료됐다는 뜻. `Error::Exchange`로 올려서, 더 할 말이 없는 스트림을 계속 기다리지 않게 합니다 |
| `TRADE_LITE` | 아니요 | `ORDER_TRADE_UPDATE`가 이미 싣는 같은 체결을 더 일찍, 더 적은 필드로 보냅니다. 요청하면 체결 하나를 두 번 보고하게 됩니다 |
| `MARGIN_CALL` | 아니요 | Binance 자신의 문서가 매매 판단에 쓰지 말라고 한 위험 안내입니다. `AccountEvent`에 담을 자리가 없습니다 |
| `ACCOUNT_CONFIG_UPDATE` | 아니요 | 레버리지와 멀티에셋 모드인데, `maxt`는 둘 다 보고하지 않습니다. Binance 포지션의 `leverage`와 `margin_mode`는 언제나 `None`입니다 |
| `CONDITIONAL_ORDER_TRIGGER_REJECT` | 아니요 | 발동된 TP/SL이 거절됐다는 뜻이고, `maxt`는 조건부 주문을 넣지 않습니다 |
| `STRATEGY_UPDATE` | 아니요 | `maxt`가 만들지 않는 Binance 자체 그리드 전략입니다 |
| `GRID_UPDATE` | 아니요 | 그 전략의 하위 주문이고, Binance 문서에서 폐기 예정입니다 |
| `ALGO_UPDATE` | 아니요 | 알고리즘 주문이고, `maxt`는 넣지 않습니다 |

위 표에서 `maxt`가 요청하지 않는 이벤트는 모두 받아도 그대로 버릴 것들입니다.

`eventStreamTerminated`는 USD-M이 발행하지 않으므로 표에 없습니다. 이 이벤트는
WebSocket API 세션을 끝내는데, 그 세션은 현물 소켓에만 있습니다. `maxt`는 이
이벤트를 처리하므로, 처리하면서도 여기서 일부러 요청하지 않는 유일한
이벤트입니다.

이 URL에서 측정한 것과 측정하지 않은 것입니다.

| 주장 | 근거 |
| --- | --- |
| Binance가 이 `/private` 형식을 발행합니다 | 변경 공지 자신의 예시를 옮긴 것 |
| 소켓이 열리기만 한 것이 아니라 이 계정을 싣고 있습니다 | 측정. 키를 삭제하자 이 URL로 `listenKeyExpired`가 밀려왔습니다 |
| 사용자 데이터에서 폐지된 경로는 죽었습니다 | 측정. 같은 키로 연 `wss://fstream.binance.com/ws/<키>`는 같은 삭제에서 아무것도 받지 못했습니다 |
| 만료된 키가 소비자에게 도달합니다 | `maxt`로 측정. `subscribe_account`가 코드 `listenKeyExpired`를 담은 `Error::Exchange`를 냈고, 이 필터를 고치기 전의 같은 실행은 아무것도 내지 않았습니다 |
| `ORDER_TRADE_UPDATE`가 도착합니다 | 측정. 지정가 주문을 넣자 `x=NEW`, `X=NEW`가, 취소하자 `x=CANCELED`, `X=CANCELED`가 밀려왔고, 주문 식별자가 REST 등록 응답 및 REST 조회와 일치했습니다 |
| `ACCOUNT_UPDATE`가 도착합니다 | **측정하지 않았습니다.** 그 주문을 넣을 때도 취소할 때도 오지 않았습니다. Binance는 잔고나 포지션이 실제로 바뀔 때 `ACCOUNT_UPDATE`를 밀어 주는데, 호가창에 얹힌 주문은 둘 다 바꾸지 않습니다. 해석기는 Binance가 발행한 페이로드로 대신 고정해 두었습니다 |

**잡아 둔 증거금은 `ACCOUNT_UPDATE`로 오지 않습니다.** 호가창에 얹힌 주문은
잔고도 포지션도 움직이지 않으면서 증거금만 잡으므로, 그것을 알려고
`ACCOUNT_UPDATE`를 기다리면 영영 기다리게 됩니다. 이 값은
`ORDER_TRADE_UPDATE`의 `b` 필드, 즉 잡아 둔 매수 주문 금액에 실립니다.
2026-07-31 측정으로 주문이 얹혀 있는 동안 `b`는 `"5"`, 취소 뒤에는 `"0"`이었고,
같은 시점의 `Balance::locked`는 각각 `0.25000000`과 `0.00000000`으로 그 금액을
20배 레버리지로 나눈 값이었습니다. `maxt`는 `b`를 노출하지 않습니다. 잡힌 금액은
`balances()`로 읽으세요.

이제 만료된 키는 두 경로로 알려지고, 둘 다 필요합니다. 갱신기는 30분마다 REST로
키를 연장하고 실패를 스트림에 흘려보내는데, 이것은 갱신기가 Binance에 닿지 못하는
경우를 덮습니다. 이벤트는 키가 다른 경로로 무효가 된 경우를 덮고, 이쪽은 `maxt`의
어떤 REST 호출로도 알아챌 수 없습니다.

### 현물 계정 스트림

현물의 사용자 데이터 스트림은 listen key를 쓰지 않고, 쓸 키도 이제 없습니다.
Binance는 2025-04-07에 `wss://stream.binance.com:9443`의 `listenKey`를 폐기
예정으로 알렸고, 키를 발급하던 엔드포인트를 2026-02-20 07:00 UTC에 제거했습니다.
지금 현물 문서에는 API 키로 WebSocket API를 통해 구독하라는 말만 남아 있습니다.

| 2026-02-20 07:00 UTC에 제거된 것 | 대체 |
| --- | --- |
| `POST`, `PUT`, `DELETE /api/v3/userDataStream` | 없음. 만들거나 연장하거나 닫을 키가 없습니다 |
| `userDataStream.start`, `.ping`, `.stop` | `userDataStream.subscribe.signature`. 소켓 하나당 서명된 요청 하나 |

현물 어댑터의 `subscribe_account`는 `wss://ws-api.binance.com:443/ws-api/v3`를
열고 서명된 `userDataStream.subscribe.signature` 하나를 보냅니다. 소켓은 인증
없이 열리고 그 프레임이 계정을 지목하므로, URL에 비밀이 들어가지 않고 살려 둘
키도 없습니다.

이쪽에는 이벤트를 거르는 장치가 없습니다. 구독 프레임에 이벤트 목록이 없고 잘못
적을 `events` 매개변수도 없으므로, 현물 소켓은 계정이 만들어 내는 것을 모두 받고
`maxt`는 다루지 않는 것을 미리가 아니라 받은 뒤에 버립니다.

같은 구독에 이르는 방법이 둘 있고, 어느 쪽을 쓸 수 있는지는 인증 정보의 종류가
정합니다.

| 방법 | HMAC-SHA-256 키 | Ed25519 키 |
| --- | --- | --- |
| 요청마다 서명하는 `userDataStream.subscribe.signature` | 됩니다 | 됩니다 |
| `session.logon` 뒤 `userDataStream.subscribe` | `-2028 HMAC-SHA-256 API key is not supported`, 이어서 `-1193 WebSocket session not authenticated` | 됩니다 |

`maxt`는 앞의 것을 보냅니다. 두 종류 모두에 통하므로 어느 쪽 키도 `session.logon`을
거칠 필요가 없습니다.

2026-07-31에 실제 HMAC-SHA-256 키로, 잔고도 미체결 주문도 없는 현물 계정에서 잰
결과입니다.

| 주장 | 근거 |
| --- | --- |
| `POST /api/v3/userDataStream`은 사라졌습니다 | 측정. 소켓을 열기도 전에 nginx 오류 페이지와 함께 `410 Gone` |
| 현물 구독이 받아들여집니다 | 측정. `maxt`가 여는 소켓에서 `{"status":200,"result":{"subscriptionId":0}}` |
| 그 소켓이 실제로 이 계정을 싣고 있습니다 | 측정. 150초를 조용히 보낸 뒤 보낸 `userDataStream.unsubscribe`에 `{"subscriptionId":0,"event":{"e":"eventStreamTerminated"}}`가 밀려왔습니다. 구독이 살아 있는 소켓에만 오는 프레임입니다 |
| 잘못된 서명이 침묵과 구별됩니다 | `maxt`로 측정. 틀린 secret으로 연 스트림은 `-1022`를 담은 `Error::Exchange`를 내고, 같은 실행을 맞는 secret으로 하면 아무것도 나오지 않습니다 |
| 조용한 계정에서도 소켓이 유지됩니다 | 측정. `maxt`로 200초 동안 이벤트도 오류도 재연결도 없었습니다 |
| 잔고나 주문 이벤트가 제대로 해석됩니다 | **측정하지 않았습니다.** 계정에 잔고도 미체결 주문도 없고, 이벤트를 만들려고 주문을 넣지도 않았습니다. 해석기는 Binance가 발행한 페이로드로 대신 고정해 두었습니다 |

넷째 줄이 예전 방식으로는 얻을 수 없던 것입니다. 지어낸 listen key도 핸드셰이크는
통과하고 그 뒤로 조용했으므로, 틀린 스트림과 한가한 계정이 똑같아 보였습니다. 이제
거절된 구독은 열린 채 아무 말 없는 소켓이 아니라 스트림 위의 오류입니다.

**재연결은 프레임을 새로 서명합니다.** 구독 프레임은 만들어진 밀리초 시각을
서명에 담으므로 서명 하나가 소켓 하나를 구독합니다. `recvWindow`가 지난 뒤
다시 보낸 프레임은 Binance가 거부하고, 그것을 되풀이하는 재연결 루프는 아무것도
싣지 않는 소켓만 계속 엽니다. `maxt`는 핸드셰이크마다 다시 서명합니다. Upbit와
Bithumb에서 핸드셰이크마다 authorization 헤더를 새로 만드는 것과 같은 방식입니다.

2026-07-31 측정입니다. `subscribe_account`를 75초 뒤 끊기는 로컬 중계기로
통과시켜 Binance를 상대로 실제 재연결을 일으켰습니다.

| 재연결이 보낸 것 | Binance의 응답 |
| --- | --- |
| 첫 소켓이 구독했던 프레임을 그대로 다시 보냄 | `400 -1021 Timestamp for this request is outside of the recvWindow` |
| 그 핸드셰이크를 위해 서명한 프레임. `maxt`가 보내는 것 | `{"status":200,"result":{"subscriptionId":0}}` |

`recvWindow`는 여전히 중요합니다. 서명 *하나*가 얼마나 긴 끊김을 견디는지, 그래서
재연결 자체에 걸리는 시간을 얼마나 덮어 주는지를 정하기 때문입니다. `maxt`는
Binance가 문서로 밝힌 최댓값 60,000 ms를 보냅니다. 2026-07-31에 소켓 하나로 잰
결과입니다.

| 서명 시점 | `recvWindow` | 응답 |
| --- | --- | --- |
| 50초 전 | 없음. Binance 기본값 5,000 ms | `-1021 Timestamp for this request is outside of the recvWindow` |
| 50초 전 | `maxt`가 보내는 60,000 | `{"status":200,"result":{"subscriptionId":0}}` |
| 90초 전 | 60,000 | `-1021`. 60,000은 한계값이지 우회로가 아닙니다 |

다른 이유로 Binance가 재구독을 거절하면 열린 채 조용한 소켓이 아니라 그 이유의
코드를 담은 `Error::Exchange`가 나옵니다. 이 오류를 보시면 다시 구독하세요.

두 진입점에 걸친 피드를 함께 지정한 USD-M 구독은 소켓 두 개로 열려 하나의
`MarketStream`으로 합쳐집니다. 각 소켓은 따로 재접속하므로, 이런 구독은
장애 한 번에 한 번이 아니라 되살아난 소켓마다 `MarketEvent::Reconnected`를
한 번씩 알립니다.

### 아무것도 오지 않는 스트림

이벤트도 오류도 오지 않는 피드는 아무 일도 일어나지 않는 시장과 구별되지
않습니다. `maxt`는 타이머도, 구독 응답도, 피드별 생존 신호도 알려 주지 않으므로
이 둘을 갈라 주는 장치가 API 안에는 없습니다.

| 무엇을 가리려면 | 이렇게 하세요 |
| --- | --- |
| 이 피드가 실려 있기는 한가 | 같은 대상을 REST로 물어보세요. `Client::ticker`와 `Client::candles`는 모든 USD-M 마켓에 응답하므로, 스트림은 조용한데 REST가 답한다면 문제는 마켓이 아니라 스트림입니다 |
| 소켓이 살아 있는가 | `StreamConfig::idle_timeout_ms`를 설정하세요. 그 시간 동안 아무것도 보내지 않은 소켓은 끊고 다시 열리며, 다시 열린 사실은 `MarketEvent::Reconnected`로 도착합니다 |
| 이 엔드포인트만의 문제인가 | 같은 스트림 이름을 다른 진입점에 raw 소켓으로 구독해 위 표처럼 프레임을 세어 보세요 |

조용한 피드를 제보하기 전에 위를 먼저 해 보세요.

## 요청 할당량

Binance는 요청 수가 아니라 **IP당 분당 가중치**로 예산을 잡고 시장별로 따로
셉니다. 깊은 호가창은 티커보다 훨씬 비쌉니다. 모든 응답이 누적값을
`X-MBX-USED-WEIGHT-1M`에 실어 보냅니다.

Binance는 상한이 아니라 방식을 문서로 밝힙니다. 각 시장이 자신의 상한을
`exchangeInfo` 응답의 `rateLimits` 배열에 1분 구간짜리 `REQUEST_WEIGHT` 항목으로
실어 보냅니다. 위 표의 두 숫자, 현물 6,000과 USD-M 2,400도 거기서 왔습니다. 여기
적힌 숫자를 믿기보다 직접 읽어 오는 편이 낫습니다.

`maxt`는 속도를 조절하지도, 그 헤더를 읽지도 않습니다. 예산을 넘기면 HTTP
429이고 `Error::is_rate_limited()`가 알려 줍니다. 429를 무시하면 2분에서 3일까지
자동으로 IP가 차단되니 첫 번째에서 물러서세요.

## 유령 포지션

`/fapi/v3/positionRisk`는 주문 하나만 얹혀 있어도 그 심볼에 수량 0짜리 행을
만듭니다. `positions()`가 돌려주는 것은 보유 포지션이고 크기 없는 행은 그것이
아니므로, `maxt`는 이 행을 버립니다.

포지션이 하나도 없는 입금된 USD-M 계정에서 2026-07-31에 잰 결과입니다.

| 계정 상태 | 원본 엔드포인트 | `positions()` | `open_orders()` | `Balance::locked` |
| --- | --- | --- | --- | --- |
| XRPUSDT 지정가 주문 하나가 얹힘 | 1행, `positionAmt` `0.0` | 0 | 1 | 0.25000000 |
| 그 주문을 취소함 | `[]` | 0 | 0 | 0.00000000 |

바뀐 것은 가운데 열입니다. 필터를 넣기 전에는 얹힌 주문 하나 때문에
`positions()`가 계정이 거래한 적도 없는 거래 시장에 `quantity: 0`, `side: None`인
`Position` 하나를 돌려줬습니다.

| 물음 | 답 |
| --- | --- |
| 무엇이 이 행을 만드는가 | 그 심볼의 미체결 주문뿐이고 그 밖에는 없습니다. 빈 계정에서는 절대 보이지 않으며, 검토를 일곱 번 도는 동안 드러나지 않은 이유가 이것입니다 |
| `positions_on(&market)`은 어떻게 하는가 | 똑같이 합니다. 주문만 얹힌 거래 시장은 빈 목록으로 답하고, 이는 닫힌 포지션을 거래소 단에서 `assetPositions`에 아예 싣지 않는 Hyperliquid와 같습니다 |
| 어디서 버리는가 | 이 어댑터가 아니라 공통 API입니다. `positions()`와 `positions_on()`은 아래에 어떤 어댑터가 있든 크기가 0인 행을 모두 버리므로, 이 보장은 거래소마다 하나가 아니라 필터 하나입니다 |
| 그 0짜리 행을 어딘가에 남겨 두는가 | 아니요. 공개 정보인 마크 가격 말고 그 행이 하던 말은 그 심볼에 주문이 얹혀 있다는 것뿐이고, `open_orders()`가 그것을 그대로 말합니다 |
| `maxt`가 해석하지 못한 행도 버리는가 | 아니요. 해석에 성공하고 크기가 0인 행만 버립니다. 형식이 어긋난 행은 그대로 `Error::Decode`로 보고합니다 |

## 주의할 점

| 필드 또는 호출 | 예상할 것 |
| --- | --- |
| `Ticker::last_trade_time` | 언제나 `None`. 마지막 가격이 언제 체결됐는지 Binance는 밝히지 않습니다 |
| `Ticker::timestamp` | 체결 시각이 아니라 24시간 구간의 끝 |
| 현물 호가창 타임스탬프 | 읽은 시각. Binance는 현물 depth에 시계를 싣지 않습니다 |
| `Position::leverage`, `margin_mode` | `None`. `/fapi/v3/positionRisk`는 둘 다 싣지 않습니다. Binance는 심볼에 설정된 레버리지와 마진 모드를 `/fapi/v1/symbolConfig`에 두며 가중치도 같으므로, 필요한 호출자는 그것을 읽습니다 |
| 주문만 얹혀 있는 심볼 | 포지션이 아닙니다. Binance는 하나로 보고하고 `maxt`는 버립니다. [유령 포지션](#유령-포지션) 참고 |
| `FundingPayment::rate` | `None`. 원장은 비율이 아니라 청구액을 기록합니다 |
| `MarginSummary::equity` | `totalMarginBalance`. 지갑 잔고에 미실현 손익을 더한 값 |
| `MarginSummary::margin_balance` | `totalInitialMargin`. 보유 포지션과 주문이 이미 잡아먹은 증거금입니다. 예산이 아니라 비용입니다 |
| `MarginSummary::available_balance` | `availableBalance`. 새로 열 때 쓸 수 있는 금액이고 셋 중 여유를 좌우하는 유일한 값입니다 |
| 마진 수치 셋 모두 | `USDT` 기준 |
| USD-M의 `Balance::locked` | 지갑 잔고에서 가용 잔고를 뺀 값, 0에서 멈춤 |
| 스트림 주문의 `created_at` | 현물은 생성 시각 `O`로 매깁니다. USD-M은 `ORDER_TRADE_UPDATE`가 생성 시각을 싣지 않아 `T`로 매기므로, 호가창에 얹혔다가 나중에 체결된 USD-M 주문에서는 체결 시각이 들어갑니다 |
| `cancel_order`, `spot_order` | Binance가 발급한 숫자 주문 식별자만. 사용자 지정 식별자는 `Error::InvalidRequest` |
| `set_margin` | 필드마다 하나씩 최대 두 번의 호출이고 원자적이지 않습니다. 레버리지는 1 이상의 정수 배수여야 합니다 |
| 만기가 있는 선물 | `markets()`에서 제외. 무기한으로 보고하면 가격을 잘못 매깁니다 |
| 스트림 프레임에 모르는 quote 자산 | 엉뚱한 마켓 대신 `Error::Decode` |
| 인증 정보 없음 | `Error::Unsupported`가 아니라 `Error::Auth` |
| Binance가 거부한 인증 정보 | `Error::Auth`가 아니라 `Error::Exchange`. 서명이 틀리면 HTTP 400 `-1022`, 키가 틀렸거나 권한이 없으면 HTTP 401 `-2015`, 키가 없으면 HTTP 401 `-2014`. 2026-07-31 측정 |
| `recvWindow`를 벗어난 시각 `-1021` | `ExchangeErrorKind::Rejected`라서 `is_retryable()`은 `false`입니다. 시계를 맞추거나, 요청을 다시 만들어 한 번 보내세요. 루프는 둘 다 풀지 못합니다 |
| 현물 계정 스트림 | listen key 없이 WebSocket API에 서명된 요청 하나이고, 재연결마다 다시 서명합니다. [현물 계정 스트림](#현물-계정-스트림) 참고 |
| 거절된 현물 구독 | 열린 채 조용한 소켓이 아니라 Binance의 코드를 담은 스트림 위의 `Error::Exchange` |
| Binance가 끝낸 현물 구독 | Binance가 밀어 준 이벤트 이름 그대로 `listenKeyExpired` 또는 `eventStreamTerminated`를 코드로 담은 `Error::Exchange` |
| 연장되지 않는 USD-M listen key | Binance 자신의 판정을 그대로 스트림에 전달합니다. 그대로 두면 한 시간 안에 계정 변화를 싣지 않게 됩니다 |

## Binance 전용 호출

`Client::adapter()`를 통해 호출합니다.

| 메서드 | 돌려주는 것 | 없는 곳 |
| --- | --- | --- |
| `spot_symbol_filters(&market)` | 호가 단위, 가격과 수량의 한계, 수량 단위, 최소 주문 금액 | USD-M |
| `spot_order(&market, id)` | 식별자로 주문 하나, 체결 완료와 취소까지 | USD-M |
| `usd_m_create_listen_key()` | USD-M 사용자 데이터 스트림 키 | 현물 |
| `usd_m_keepalive_listen_key(&key)` | 키를 60분 더 연장 | 현물 |
| `usd_m_close_listen_key(&key)` | 키를 닫습니다 | 현물 |

주문 규칙을 똑같이 표현하는 거래소는 둘도 없어서 필터는 Binance의 모양을 그대로
유지합니다. 해당 종류의 필터가 없는 심볼에서 필드는 `None`이며 갓 상장된
페어에서는 흔합니다.

위 세 가지는 USD-M 전용이고, 현물 쪽이 다른 이름으로 숨어 있어서가 아닙니다.
현물에는 2026-02-20부터 listen key 자체가 없습니다. USD-M listen key의 수명은
`subscribe_account`가 대신 관리합니다. 소켓을 직접 다루거나, 키 하나를 여러
소비자가 나눠 쓰거나, 재시작을 넘겨 키를 유지할 때만 꺼내세요.
`BinanceListenKey`는 소켓 URL에 들어가는 bearer 비밀값이라 `Debug` 출력에서
가려집니다.

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
HMAC-SHA256으로 서명하고 전송되는 것은 서명뿐입니다.

이 어댑터에는 Binance 테스트넷 호스트가 없습니다. 거래 권한을 끈 키로
시험하세요.

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
