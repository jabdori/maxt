# maxt

[English](README.md) | [한국어](README.ko.md)

Upbit, Bithumb, Binance, Hyperliquid의 시장 데이터, 계좌, 주문, 스트림을
제공하는 타입 기반 비동기 API입니다.

## 만든 이유

- 여러 거래소를 같은 작업, 모델, 오류, 스트림 계약으로 사용합니다.
- 공통 작업은 `Client`, 거래소 전용 작업은 각 어댑터에서 제공합니다.
- 하나의 스키마에서 언어별 계약을 생성하고, 생성 코드와 컴파일된 네이티브 API의 정합성을 검사합니다.

## 설치

Rust 1.85 이상이 필요합니다.

```toml
[dependencies]
maxt = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Binance 예제

```rust,no_run
use maxt::adapters::BinanceAdapter;
use maxt::{Client, Exchange, Market};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::spot());
    let market = Market::spot(Exchange::Binance, "BTC", "USDT");

    let ticker = client.ticker(&market).await?;
    let filters = client.adapter().spot_symbol_filters(&market).await?;

    println!("{}", ticker.last_price);
    println!("{:?}", filters.tick_size);
    Ok(())
}
```

`ticker()`는 공통 API입니다. `spot_symbol_filters()`는 Binance Spot 전용이며
`Client::adapter()`를 통해 호출합니다.

공개 REST 예제 실행:

```sh
cargo run --example public_rest
```

## 지원 상태

- [x] Rust
- [x] Python
- [x] Dart / Flutter 네이티브
- [x] TypeScript / Node.js
- [x] TypeScript / Browser WebAssembly

## 문서

- [시작하기](docs/getting-started.ko.md)
- [공통 API](docs/common-api.ko.md)
- [거래소 지원](docs/providers.ko.md)
- Rust API: `cargo doc --open`
- [Python](bindings/python/README.ko.md)
- [Dart / Flutter](bindings/dart/README.ko.md)
- [TypeScript](bindings/typescript/README.ko.md)
- [브라우저 릴레이](relay/README.ko.md)
- [예제](examples/)
- [변경 기록](CHANGELOG.md)
- [기여 가이드](CONTRIBUTING.ko.md)

## 라이선스

MIT. [LICENSE](LICENSE)를 참고하세요.
