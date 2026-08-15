# 시작하기

[English](getting-started.md) | [한국어](getting-started.ko.md)

이 가이드는 Binance 현물(Spot)부터 시작합니다. 공개 REST와 시장 스트림에는 거래소
계정이 필요하지 않습니다.

## 설치

Rust 1.85 이상이 필요합니다. 스트림 예제는 `futures_util::StreamExt`를
사용합니다.

```toml
[dependencies]
maxt = "0.3.3"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

## 시장 데이터 조회

```rust,no_run
use maxt::adapters::BinanceAdapter;
use maxt::{Client, Exchange, Market, MarketKind};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::spot());
    let market = Market::spot(Exchange::Binance, "BTC", "USDT");

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
use maxt::adapters::BinanceAdapter;
use maxt::{Client, Exchange, Feed, Market, MarketEvent, Subscription};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::spot());
    let subscription = Subscription::new()
        .market(Market::spot(Exchange::Binance, "BTC", "USDT"))
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
- [작업 중심 예제](examples.ko.md): 모든 공개 API 시나리오의 실행 가능한 소스
- [Binance 첫 조회 튜토리얼](tutorials/binance-first-read.ko.md): 모든 언어에서 같은 첫 조회
- [Python 패키지](../bindings/python/README.ko.md), [Dart / Flutter 패키지](../bindings/dart/README.ko.md), [TypeScript 패키지](../bindings/typescript/README.ko.md): 언어별 설정과 실행 가능한 Binance 예제
- [Rust 예제 색인](../examples/README.md)

이 가이드는 의도적으로 공개 호출만 사용합니다. 계좌, 주문, 전송 작업은 `Client`를
만들기 전에 어댑터를 설정하세요. Hyperliquid의 계좌 조회는 서명 없이 공개 조회 주소를
사용할 수 있지만, 서명 작업에는 signer가 필요합니다. 정확한 어댑터 설정은
[거래소 지원](providers.ko.md)을 참고하세요.
