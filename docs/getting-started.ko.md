# 시작하기

[English](getting-started.md) | [한국어](getting-started.ko.md)

단계는 다섯이고 각 단계를 따로 실행합니다. API 키는 마지막 두 단계에만 필요합니다.

## 설치

`maxt`는 패키지 레지스트리에 없으므로 저장소를 의존성으로 지정합니다. Rust
1.85 이상이 필요합니다. 3단계에서 구독의 이벤트를 꺼낼 때 쓰는 `StreamExt`는
`futures-util`에 있습니다.

```toml
[dependencies]
maxt = { git = "https://github.com/jabdori/maxt" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

## 1. 어댑터 고르기

어댑터 하나가 거래소 하나와 통신합니다. `Client`는 그 어댑터를 감싸 공통 API를
내놓습니다.

```rust
use maxt::adapters::{
    BinanceAdapter, BinanceMarket, BithumbAdapter, HyperliquidAdapter, UpbitAdapter,
};
use maxt::{Client, Exchange, Feature};

fn adapters() {
    let upbit = Client::new(UpbitAdapter::new());
    let bithumb = Client::new(BithumbAdapter::new());
    let binance_spot = Client::new(BinanceAdapter::spot());
    let binance_perp = Client::new(BinanceAdapter::usd_m_futures());
    let hyperliquid = Client::new(HyperliquidAdapter::new());

    assert_eq!(upbit.exchange(), Exchange::Upbit);
    // 거래소 하나에 거래 시장 둘, 생성 시점에 고정됩니다.
    assert_eq!(binance_spot.adapter().venue(), BinanceMarket::Spot);
    assert_eq!(binance_perp.adapter().venue(), BinanceMarket::UsdMFutures);
    // 각각이 무엇을 하는지는 요청을 보내기 전에 로컬에서 답합니다.
    assert!(hyperliquid.supports(Feature::FundingRates));
    assert!(!binance_spot.supports(Feature::FundingRates));
    assert!(!bithumb.supports(Feature::CandleStream));
}
```

다섯은 서로 다른 타입이므로 한 변수가 이것을 담았다가 저것을 담지는 못합니다.
실행 시점에 거래소를 정하려면
[`examples/public_rest.rs`](../examples/public_rest.rs)처럼 어댑터를 박싱해
`Client<Box<dyn Adapter>>`로 다룹니다. 거래소가 실행 시점 값이면 마켓의 종류도
대개 실행 시점 값이고 그럴 때 쓰는 생성자가
`Market::new(exchange, kind, base, quote)`입니다. 각각이 무엇을 못 하는지는
[거래소 고르기](providers.ko.md)에 있습니다.

## 2. 공개 시세 읽기

인증 정보가 필요 없습니다.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Market, MarketKind};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(UpbitAdapter::new());
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    let markets = client.markets(MarketKind::Spot).await?;
    println!("{} lists {} spot markets", client.exchange(), markets.len());

    let ticker = client.ticker(&market).await?;
    println!("{market} last {}", ticker.last_price);

    let book = client.order_book(&market, Some(5)).await?;
    println!("spread {:?}", book.spread());
    Ok(())
}
```

세 가지 규칙이 `maxt` 전체에 그대로 적용됩니다.

- `Market`은 거래소, 종류, 기준 자산, quote 자산으로 이루어집니다. 같은 종목을
  거래소가 부르는 이름으로 바꾸는 일은 어댑터가 합니다. 같은 페어의 현물과 무기한
  선물은 플래그 하나가 다른 한 마켓이 아니라 서로 다른 두 마켓입니다.
- 가격과 수량은 `Decimal`이고 `f64`는 아닙니다.
- 거래소가 발행하지 않는 값은 0이 아니라 `None`입니다. `ticker.volume`이
  `None`이라면 거래소가 거래량을 두고 아무 말도 하지 않았다는 뜻입니다.

## 3. 실시간 피드 구독하기

구독은 마켓과 피드를 지정합니다. 어느 쪽을 몇 개 넣든 구독 하나는 연결 하나가
됩니다.

```rust,no_run
use futures_util::StreamExt;
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Feed, Market, MarketEvent, Subscription};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(UpbitAdapter::new());
    let subscription = Subscription::new()
        .market(Market::spot(Exchange::Upbit, "BTC", "KRW"))
        .feed(Feed::Trades);

    let mut stream = client.subscribe(&subscription).await?;
    while let Some(event) = stream.next().await {
        match event {
            Ok(MarketEvent::Trade(t)) => println!("{} {} {:?}", t.price, t.quantity, t.taker_side),
            // 소켓이 끊겼다가 다시 붙었습니다. 그 사이에 발행된 것은 놓쳤습니다.
            Ok(MarketEvent::Reconnected) => println!("reconnected; there is a gap behind us"),
            Ok(other) => println!("{other:?}"),
            // 스트림은 이것을 보고한 뒤에도 계속 돕니다.
            Err(error) => eprintln!("reported, still subscribed: {error}"),
        }
    }
    Ok(())
}
```

항목에 `?`를 붙이지 말고 항목을 매칭하세요.

- `Err`는 스트림이 지나쳐 가며 보고하는 내용입니다. 읽지 못한 프레임이거나,
  일시적이라고 보기 어려워진 재연결입니다. 더 올 것이 없다는 뜻은 `None`뿐이므로,
  첫 `Err`에서 반환하면 곧 회복될 구독을 버립니다.
- 스트림을 드롭하면 연결이 닫힙니다.
- 모든 거래소가 모든 피드를 싣지는 않습니다. Bithumb은 캔들 스트림을 발행하지
  않고 캔들을 요청한 구독은 통째로 실패합니다. 그 피드가 조용히 빠지는 것이
  아닙니다.
  그 답에 따라 프로그램의 동작이 달라져야 한다면
  `client.supports(Feature::CandleStream)`으로 먼저 물어보세요.
- `supports`가 답한 `true`는 모든 인자를 약속하지 않습니다. Upbit은
  `Feature::CandleStream`을 지원한다고 답하면서도
  `Feed::Candles(Interval::Day1)`은 거절합니다. 일봉을 스트림으로 발행하지 않기
  때문입니다. 미리 확인했더라도 호출 지점에서 `Error::Unsupported`를 처리하고
  [공통 API](common-api.ko.md#feature와-clientsupports)를 보세요.

## 4. 인증 정보를 넣고 계좌 읽기

인증 정보는 소스가 아니라 환경에서 읽습니다. 어댑터마다 해당 거래소가 발급하는
형태를 그대로 받습니다. Upbit은 access key와 secret key 한 쌍입니다.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Feature};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
    let secret_key = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
    let client = Client::new(UpbitAdapter::new().with_credentials(access_key, secret_key));

    // `supports`는 지금 구성된 그대로의 어댑터에 답합니다. 인증 정보가 없으면
    // false이고 호출은 아예 나가지 않습니다.
    if !client.supports(Feature::Balances) {
        return Ok(());
    }

    for balance in client.balances().await? {
        if !balance.total().is_zero() {
            println!("{} {} available", balance.asset, balance.available);
        }
    }
    for order in client.open_orders().await? {
        println!("{} {:?} {:?} (id {})", order.market, order.side, order.status, order.id);
    }
    Ok(())
}
```

위 코드에는 조회 권한만 있는 키로 충분합니다.

`client.subscribe_account()`는 3단계와 같은 모양이고
`AccountEvent::Balance`와 `AccountEvent::Order`를 흘려보냅니다. 이쪽의
`Reconnected`는 시세 쪽보다 무겁습니다. 끊긴 사이에 체결이 일어났을 수 있으므로,
로컬 상태를 다시 믿기 전에 잔고와 미체결 주문을 REST로 다시 읽으세요.

## 5. 주문 내기

이 단계에는 거래 권한이 있는 키가 필요합니다. 네 거래소 중 테스트 네트워크를
공개한 곳은 Hyperliquid뿐입니다. Upbit에서 아래 주문은 실제 자금이 걸린 실제
주문입니다. 그래서 호가창이 실제로 담고 있는 가장 깊은 매수 호가로 주문을 내고
곧바로 취소합니다.

`OrderRequest::limit`은 마켓, 방향, `Size`로 감싼 크기, 가격을 그 순서대로
받습니다. `Size::Base`와 `Size::Quote`는 그 숫자가 어느 자산 기준인지 밝히므로
원화로 매긴 시장가 매수와 비트코인으로 매긴 시장가 매수가 헷갈리지 않습니다.
취소에는 반환된 `Order`에 담긴 거래소 자신의 주문 ID를 넘깁니다.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Decimal, Exchange, Market, OrderRequest, Side, Size, TimeInForce};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
    let secret_key = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
    let client = Client::new(UpbitAdapter::new().with_credentials(access_key, secret_key));
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    // 가격은 이 문서가 아니라 호가창에서 옵니다. 여기에 적어 둔 숫자는 시세가 그
    // 위에 머무는 동안에만 호가창에 걸리고, 시세가 내려온 날에는 아무 예고 없이
    // 테이커로 체결됩니다. 거래소가 돌려준 가장 깊은 매수 호가는 구조적으로 다른
    // 모든 매수 호가보다 낮으므로, 시세가 어떻든 매도 호가를 넘지 않습니다.
    let book = client.order_book(&market, None).await?;
    let Some(deepest_bid) = book.bids.last() else {
        println!("no bids on the book");
        return Ok(());
    };

    let order = client
        .place_order(
            &OrderRequest::limit(
                market.clone(),
                Side::Buy,
                // 0.001 BTC. 여기에 `Size::Quote`를 썼다면 0.001 KRW가 됩니다.
                Size::Base(Decimal::new(1, 3)),
                deepest_bid.price,
            )
            // 그 조회와 이 호출 사이에 호가창이 움직였다면 테이커로 체결되지 않고
            // 그대로 거절됩니다.
            .time_in_force(TimeInForce::PostOnly),
        )
        .await?;

    println!("{} is {:?}", order.id, order.status);

    // `is_live`가 "아직 체결될 수 있는가"를 묻는 검사입니다. 취소는 호가창과
    // 경합하므로 보낸 주문이 아니라 돌아온 주문을 믿으세요.
    if order.status.is_live() {
        let cancelled = client.cancel_order(&market, &order.id).await?;
        println!("{} filled, {} withdrawn", cancelled.filled_quantity, cancelled.remaining_quantity);
    }
    Ok(())
}
```

호가 단위, 수량 단위, 최소 주문 금액은 거래소마다 다릅니다. 다섯 거래소 구성 중
둘이 이를 노출하고, 그중 하나만 서명 전에 주문을 그 값에 맞춰 검사합니다. 실제
크기를 잡기 전에
[주문 정밀도와 최소 주문 크기](common-api.ko.md#주문-정밀도와-최소-주문-크기)를
보세요. Hyperliquid에는 시장가 주문 종류가 아예 없고 quote 자산 기준 크기도
보편적이지 않으므로 고른 거래소의 페이지를 읽으세요. 자세한 내용은
[공통 API](common-api.ko.md#주문)에 있습니다.

## 다음에 볼 것

- [`examples/`](../examples/)에 이 단계들의 바탕이 된 네 프로그램이 있습니다.
  [`public_rest.rs`](../examples/public_rest.rs),
  [`public_stream.rs`](../examples/public_stream.rs),
  [`private_account.rs`](../examples/private_account.rs),
  [`private_stream.rs`](../examples/private_stream.rs). 실행은
  `cargo run --example public_rest`처럼 합니다. `public_stream.rs`는 3단계에
  체결 건수와 시간 제한을 붙여 스스로 끝나도록 만든 것입니다.
- [공통 API](common-api.ko.md): 에러, `Decimal`, 타임스탬프, 구독, 페이지 조회,
  그리고 거래소 고유 메서드에 닿는 방법. 위의 어느 단계도 파생상품 절반을
  건드리지 않았습니다. 무기한 선물 시장의 포지션, 마진, 펀딩, 레버리지는
  [거기서 끝까지 훑습니다](common-api.ko.md#파생상품-읽기-예제).
- [거래소 고르기](providers.ko.md), 그리고 고른 거래소의 페이지:
  [Upbit](providers/upbit.ko.md), [Bithumb](providers/bithumb.ko.md),
  [Binance](providers/binance.ko.md), [Hyperliquid](providers/hyperliquid.ko.md).
