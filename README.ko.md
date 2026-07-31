# maxt

[English](README.md) | [한국어](README.ko.md)

`maxt`는 Upbit, Bithumb, Binance, Hyperliquid의 시장 데이터, 계좌, 주문을
정적 타입으로 다루는 Rust API입니다. 거래소에만 있는 기능은 각 어댑터에서
그대로 사용할 수 있습니다.

## 빠른 시작

`maxt`는 Rust 1.85 이상이 필요하며 패키지 레지스트리에는 배포되지 않았습니다.

```toml
[dependencies]
maxt = { git = "https://github.com/jabdori/maxt" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

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

전체 공개 REST 예제는 다음 명령으로 실행합니다.

```sh
cargo run --example public_rest
```

## 문서

- [시작하기](docs/getting-started.ko.md): 공개 REST와 스트리밍
- [공통 API 레퍼런스](docs/common-api.ko.md): 타입, 정렬, 오류, 비공개 호출
- [제공자 선택](docs/providers.ko.md): 생성자와 제공자별 차이
- [실행 가능한 예제](examples/)
- [기여 안내](CONTRIBUTING.ko.md)

## 검증 범위

2026-07-31에 Upbit 한국, Bithumb, Binance Spot, Binance USD-M, Hyperliquid
메인넷의 대표 마켓 하나씩을 대상으로 공개 REST와 스트리밍 API를 실시간
검증했습니다. 이 검사는 인증 정보를 사용하지 않습니다. 비공개 계좌와 거래 경로는
오프라인에서 테스트했지만 실시간으로 검증하지 않았습니다.

## 라이선스

MIT. [LICENSE](LICENSE)를 참고하세요.
