# 시작하기

[English](getting-started.md) | [한국어](getting-started.ko.md)

공개 REST와 스트림에는 거래소 계정이 필요하지 않습니다.

## 설치

Rust 1.85 이상이 필요합니다. 스트림 예제는 `futures-util`의 `StreamExt`를
사용합니다.

```toml
[dependencies]
maxt = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

## 공개 시장 데이터

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

- 공통 타입·정렬·정밀도 계약: [공통 API 레퍼런스](common-api.ko.md)
- 요청 한도·필드 출처: [제공자별 레퍼런스](providers.ko.md)

## 공개 스트림

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

| 상태 | 계약 |
| --- | --- |
| `Some(Ok(event))` | 이벤트 |
| `Some(Err(error))` | 비종료 오류 |
| `None` | 스트림 종료 |
| `MarketEvent::Reconnected` | 연결 단절 구간의 이벤트 유실 |
| 내장 스트림 `Drop` | 내장 연결 작업 전체에 종료 신호 전달 |
| `close().await` | 어댑터의 비동기 정리 완료 대기 |

계좌 스트림의 `AccountEvent::Reconnected` 이후에는 `balances()`와
`open_orders()`로 상태를 다시 읽습니다.

## 다음 단계

- [제공자 선택](providers.ko.md)
- [공통 API 레퍼런스](common-api.ko.md)
- [실행 가능한 예제](../examples/)
