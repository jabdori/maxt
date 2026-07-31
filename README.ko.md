# maxt

[English](README.md) | [한국어](README.ko.md)

Upbit, Bithumb, Binance, Hyperliquid의 시장 데이터·계좌·주문을 위한 비동기 Rust
API입니다. 거래소가 달라도 같은 `Client` 메서드와 타입을 사용합니다. 거래소 전용
메서드는 각 어댑터에 남겨 둡니다.

## 설치

Rust 1.85 이상이 필요합니다.

```toml
[dependencies]
maxt = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## 예제

공개 시장 데이터에는 인증 정보가 필요하지 않습니다.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Market};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(UpbitAdapter::new());
    let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
    let ticker = client.ticker(&market).await?;

    println!("{market}: {}", ticker.last_price);
    Ok(())
}
```

공개 REST 예제:

```sh
cargo run --example public_rest
```

## 문서

- [시작하기](docs/getting-started.ko.md)
- [공통 API 레퍼런스](docs/common-api.ko.md)
- [제공자 선택](docs/providers.ko.md)
- [API 문서](https://docs.rs/maxt)
- [실행 가능한 예제](examples/)
- [변경 기록](CHANGELOG.md)
- [기여 가이드](CONTRIBUTING.ko.md)

## 라이선스

MIT. [LICENSE](LICENSE)를 참고하세요.
