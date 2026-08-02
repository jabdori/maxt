# maxt

[English](README.md) | [한국어](README.ko.md)

Upbit, Bithumb, Binance, Hyperliquid의 시장 데이터, 계좌, 주문을 제공하는 타입 기반
비동기 Rust API입니다.

## 왜 maxt인가

`maxt`는 지원하는 모든 거래소에 같은 작업(operation), 요청·결과 타입, 구조화된
오류(error), 스트림 수명 주기(lifecycle)를 제공합니다. 공통 API에 없는 거래소
전용 기능은 `Client::adapter()`가 반환한 어댑터에서 호출할 수 있습니다.

## 설치

Rust 1.85 이상이 필요합니다.

```toml
[dependencies]
maxt = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## 예제

공개 REST와 시장 스트림에는 인증 정보가 필요하지 않습니다.

```rust,no_run
use maxt::adapters::BinanceAdapter;
use maxt::{Client, Exchange, Market};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::spot());
    let market = Market::spot(Exchange::Binance, "BTC", "USDT");
    let ticker = client.ticker(&market).await?;

    println!("{market}: {}", ticker.last_price);
    Ok(())
}
```

공개 REST 예제 실행:

```sh
cargo run --example public_rest
```

## 문서

- [시작하기](docs/getting-started.ko.md)
- [공통 API 레퍼런스](docs/common-api.ko.md)
- [거래소 지원표](docs/providers.ko.md)
- Rust API 레퍼런스: `cargo doc --open`
- [Python 바인딩](bindings/python/PYPI.md)
- [Dart / Flutter 바인딩](bindings/dart/README.md)
- [실행 가능한 예제](examples/)
- [변경 기록](CHANGELOG.md)
- [기여 가이드](CONTRIBUTING.ko.md)

## 바인딩 로드맵

Rust API가 기준 계약입니다. `완료`는 해당 바인딩이 같은 거래소 어댑터와 공통
동작을 제공한다는 뜻입니다.

| 바인딩 | Upbit | Bithumb | Binance | Hyperliquid |
| --- | --- | --- | --- | --- |
| Rust | 완료 | 완료 | 완료 | 완료 |
| Python | 완료 | 완료 | 완료 | 완료 |
| Dart / Flutter | 완료 | 완료 | 완료 | 완료 |
| TypeScript / Node.js | 예정 | 예정 | 예정 | 예정 |
| TypeScript / WebAssembly | 예정 | 예정 | 예정 | 예정 |

## 라이선스

MIT. [LICENSE](LICENSE)를 참고하세요.
