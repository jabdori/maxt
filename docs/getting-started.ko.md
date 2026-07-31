# 시작하기

[English](getting-started.md) | [한국어](getting-started.ko.md)

이 안내에서는 공개 REST 호출을 실행하고 공개 스트림 하나를 엽니다. 두 단계
모두 거래소 계정이 필요하지 않습니다.

## 설치

`maxt`는 Rust 1.85 이상이 필요하며 Git 저장소에서 설치합니다. 구독 이벤트를
읽는 데 필요한 `StreamExt`는 `futures-util`에서 제공합니다.

```toml
[dependencies]
maxt = { git = "https://github.com/jabdori/maxt" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

## 공개 시장 데이터 읽기

어댑터가 제공자를 선택합니다. `Client`는 공통 API를 노출하고, `Market`은 거래소,
마켓 종류, 기초 자산(base asset), 호가 자산(quote asset)을 식별합니다.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Market, MarketKind};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(UpbitAdapter::new());
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    let listed = client.markets(MarketKind::Spot).await?;
    let ticker = client.ticker(&market).await?;
    let book = client.order_book(&market, Some(5)).await?;

    println!("{} spot markets", listed.len());
    println!("{market}: {}", ticker.last_price);
    println!("spread: {:?}", book.spread());
    Ok(())
}
```

공통 데이터 규칙은 다음과 같습니다.

- 체결은 최신순, 캔들은 오래된 순, 호가창의 양쪽은 최우선 호가부터 정렬됩니다.
- 가격, 수량, 금액은 `f64`가 아닌 `maxt::Decimal`을 사용합니다.
- 제공자가 공개하지 않은 값은 0이 아니라 `None`입니다.
- 제공자별 요청 한도와 타임스탬프 세부 사항은 제공자마다 다릅니다.

필드를 회계나 주문 실행 판단에 사용하기 전에 [공통 API 레퍼런스](common-api.ko.md)를
확인하세요.

## 공개 스트림 구독하기

`Subscription`은 하나의 논리적 스트림입니다. 요청한 모든 피드를 요청한 모든
마켓에 적용합니다. 대부분의 어댑터는 소켓 하나를 사용하지만, Binance USD-M은
피드를 여러 소켓으로 나눈 뒤 반환하는 스트림에서 합칠 수 있습니다.

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
    while let Some(item) = stream.next().await {
        match item {
            Ok(MarketEvent::Trade(trade)) => {
                println!("{} {}", trade.price, trade.quantity)
            }
            Ok(MarketEvent::Reconnected) => {
                println!("reconnected; events during the gap were missed")
            }
            Ok(_) => {}
            Err(error) => eprintln!("stream report: {error}"),
        }
    }

    Ok(())
}
```

`Err` 항목은 문제를 알리지만 스트림을 종료하지 않습니다. `None`만 스트림이
끝났다는 뜻입니다. 스트림을 드롭하면 내부 연결을 모두 닫습니다. 계좌 스트림이
재연결된 뒤에는 로컬 상태를 신뢰하기 전에 REST로 잔고와 미체결 주문을 다시
읽으세요.

## 다른 제공자 선택하기

```rust
use maxt::adapters::{BinanceAdapter, BithumbAdapter, HyperliquidAdapter, UpbitAdapter};
use maxt::Client;

fn clients() {
    let _upbit = Client::new(UpbitAdapter::new());
    let _bithumb = Client::new(BithumbAdapter::new());
    let _binance_spot = Client::new(BinanceAdapter::spot());
    let _binance_usd_m = Client::new(BinanceAdapter::usd_m_futures());
    let _hyperliquid = Client::new(HyperliquidAdapter::new());
}
```

제공자 선택, 지원하는 캔들 간격, 주문 형식, 인증 정보는
[제공자 선택](providers.ko.md)을 참고하세요.

## 비공개 호출 전에 확인할 사항

어댑터에 인증 정보를 설정한 다음 `Client`로 감싸세요. 인증 정보를 설정하기 전에는
`Client::supports`가 비공개 기능에 `false`를 반환하고, 해당 호출은
`Error::Auth`를 반환합니다. 비공개 계좌와 거래 경로는 실시간 적합성 검사에
포함되지 않았으므로 읽기 전용 권한으로 시작하고 제공자별 제약을 직접 확인하세요.

[공통 API 레퍼런스](common-api.ko.md)와 실행 가능한
[`examples/`](../examples/)로 이어서 살펴보세요.
