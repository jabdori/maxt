# 시작하기

[English](getting-started.md) | [한국어](getting-started.ko.md)

공개 REST와 시장 스트림에는 거래소 계정이 필요하지 않습니다.

## 설치

Rust 1.85 이상이 필요합니다. 스트림 예제는 `futures_util::StreamExt`를
사용합니다.

```toml
[dependencies]
maxt = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

## 시장 데이터 조회

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Market, MarketKind};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(UpbitAdapter::new());
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

    let markets = client.markets(MarketKind::Spot).await?;
    let ticker = client.ticker(&market).await?;
    let book = client.order_book(&market, Some(5)).await?;

    println!("{} spot markets", markets.len());
    println!("{market}: {}", ticker.last_price);
    println!("spread: {:?}", book.spread());
    Ok(())
}
```

- 공통 타입과 계약: [공통 API 레퍼런스](common-api.ko.md)
- 거래소 한도와 필드 출처: [거래소 지원](providers.ko.md)

## 시장 스트림 열기

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
            Err(error) => eprintln!("stream error: {error}"),
        }
    }

    Ok(())
}
```

항목별 오류, 재연결, 종료, 명시적 정리는
[스트림 상태](common-api.ko.md#상태)를 참고하세요.

## 다음 단계

- [거래소 지원](providers.ko.md): 생성자, 인증 정보, 거래소 한도
- [공통 API 레퍼런스](common-api.ko.md): 요청, 스트림, 오류, 비공개 API
- [실행 가능한 예제](../examples/)
