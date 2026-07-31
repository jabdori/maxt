# 시작하기

[English](getting-started.md) | [한국어](getting-started.ko.md)

API 키는 마지막 두 단계에만 필요합니다.

## 설치

`maxt`는 패키지 레지스트리에 없습니다. Rust 1.85 이상이 필요합니다.

```toml
[dependencies]
maxt = { git = "https://github.com/jabdori/maxt" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

## 1. 어댑터 고르기

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
    // 요청을 보내기 전에 로컬에서 답합니다.
    assert!(hyperliquid.supports(Feature::FundingRates));
    assert!(!binance_spot.supports(Feature::FundingRates));
    assert!(!bithumb.supports(Feature::CandleStream));
}
```

어댑터 타입은 넷, 거래 시장 구성은 다섯입니다. Binance의 두 거래 시장은
`BinanceAdapter` 하나가 맡습니다. 거래소가 다르면 타입도 다르므로 한 변수가 둘을
담지는 못합니다. 실행 시점에 거래소를 정하려면
[`examples/public_rest.rs`](../examples/public_rest.rs)처럼 어댑터를 박싱해
`Client<Box<dyn Adapter>>`로 다루고 `Market::new(exchange, kind, base, quote)`로
거래 시장을 만드세요.

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

- `Market`은 거래소, 종류, 기준 자산, quote 자산으로 이루어집니다. 같은 페어의
  현물과 무기한 선물은 플래그로 갈리는 하나가 아니라 서로 다른 둘입니다.
- 가격과 수량은 `f64`가 아니라 `Decimal`입니다.
- 거래소가 발행하지 않는 값은 0이 아니라 `None`입니다.

## 3. 실시간 피드 구독하기

거래 시장과 피드를 몇 개 넣든 구독 하나는 연결 하나입니다.

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

- 스트림은 `None`으로만 끝납니다. 항목에 `?`를 붙이지 말고 매칭하세요.
- 스트림을 드롭하면 연결이 닫힙니다.
- Bithumb은 캔들 스트림을 발행하지 않습니다. 캔들을 요청한 구독은 통째로
  실패합니다. `client.supports(Feature::CandleStream)`으로 먼저 물어보세요.
- `supports`는 기능 단위로 답하지 인자 단위로 답하지 않습니다. Upbit은
  `Feature::CandleStream`을 지원한다고 답하면서도
  `Feed::Candles(Interval::Day1)`은 `Error::Unsupported`로 거절합니다
  ([공통 API](common-api.ko.md#feature와-clientsupports)).

## 4. 인증 정보를 넣고 계좌 읽기

인증 정보는 소스가 아니라 환경에서 읽습니다.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Feature};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
    let secret_key = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
    let client = Client::new(UpbitAdapter::new().with_credentials(access_key, secret_key));

    // 인증 정보가 없으면 false이고 호출은 아예 나가지 않습니다.
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

여기까지는 조회 권한만 있는 키로 충분합니다.

`client.subscribe_account()`는 `AccountEvent::Balance`와 `AccountEvent::Order`를
받습니다. `AccountEvent::Reconnected` 뒤에는 잔고와 미체결 주문을 REST로 다시
읽으세요.

## 5. 주문 내기

거래 권한이 있는 키가 필요합니다. 테스트 네트워크를 공개한 곳은 Hyperliquid뿐입니다.
Upbit에서 아래 주문은 실제 자금이 걸립니다.

`OrderRequest::limit`은 거래 시장, 방향, `Size`, 가격을 그 순서대로 받습니다.
취소에는 반환된 `Order`에 담긴 거래소 자체 주문 ID를 넘깁니다.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Decimal, Exchange, Market, OrderRequest, Side, Size, TimeInForce};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let access_key = std::env::var("UPBIT_ACCESS_KEY").expect("UPBIT_ACCESS_KEY");
    let secret_key = std::env::var("UPBIT_SECRET_KEY").expect("UPBIT_SECRET_KEY");
    let client = Client::new(UpbitAdapter::new().with_credentials(access_key, secret_key));
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    // 가격은 이 문서가 아니라 호가창에서 옵니다. 여기에 적어 둔 숫자는 시세가
    // 내려온 날 테이커로 체결됩니다. 거래소가 돌려준 가장 깊은 매수 호가는 다른
    // 모든 매수 호가보다 낮으므로, 그 가격의 매수는 매도 호가를 넘지 않습니다.
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

호가 단위, 수량 단위, 최소 주문 금액은 거래소마다 다르고 서명 전에 주문을 그
값에 맞춰 검사하는 곳은 Hyperliquid뿐입니다.
[주문 정밀도와 최소 주문 크기](common-api.ko.md#주문-정밀도와-최소-주문-크기).
Hyperliquid에는 시장가 주문 종류가 아예 없고 quote 자산 기준 크기도 보편적이지
않습니다. 주문 레퍼런스 전체는 [공통 API](common-api.ko.md#주문).

## 다음에 볼 것

- `cargo run --example` [`public_rest`](../examples/public_rest.rs),
  [`public_stream`](../examples/public_stream.rs),
  [`private_account`](../examples/private_account.rs),
  [`private_stream`](../examples/private_stream.rs)
- [공통 API](common-api.ko.md), [파생상품](common-api.ko.md#파생상품-읽기-예제)
  포함
- [거래소 고르기](providers.ko.md): [Upbit](providers/upbit.ko.md),
  [Bithumb](providers/bithumb.ko.md), [Binance](providers/binance.ko.md),
  [Hyperliquid](providers/hyperliquid.ko.md)
