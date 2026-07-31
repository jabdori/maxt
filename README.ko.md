# maxt

[English](README.md) | [한국어](README.ko.md)

`maxt`(Multi-Asset eXchange Toolkit)는 암호화폐 거래소 네 곳의 시세, 계좌, 주문을
하나의 Rust API로 다룹니다. Upbit, Bithumb, Binance(현물과 USD 마진 무기한 선물),
Hyperliquid(현물과 무기한 선물)를 지원합니다.

## 빠르게 시작하기

`maxt`는 패키지 레지스트리에 없습니다.

```toml
[dependencies]
maxt = { git = "https://github.com/jabdori/maxt" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

가격을 읽는 데는 인증 정보가 필요 없습니다.

```rust,no_run
use maxt::adapters::UpbitAdapter;
use maxt::{Client, Exchange, Market};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let upbit = Client::new(UpbitAdapter::new());
    let btc_krw = Market::spot(Exchange::Upbit, "BTC", "KRW");
    let ticker = upbit.ticker(&btc_krw).await?;
    println!("{btc_krw} last {}", ticker.last_price);

    Ok(())
}
```

`cargo run --example public_rest`로
[프로그램 전체](examples/public_rest.rs)를 실행합니다.

## 문서

- [시작하기](docs/getting-started.ko.md)
- [공통 API](docs/common-api.ko.md)
- [거래소 고르기](docs/providers.ko.md)
- [`examples/`](examples/)
- [기여 안내](CONTRIBUTING.ko.md)

## 라이선스

MIT. [LICENSE](LICENSE)를 참고하세요.
