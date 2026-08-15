# maxt

[English](README.md) | [한국어](README.ko.md)

Binance, Upbit, Bithumb, Hyperliquid의 시장 데이터, 계좌, 주문, 스트림을
제공하는 타입 기반 비동기 API입니다. Rust를 핵심으로 하고 Python,
Dart/Flutter, TypeScript에는 같은 생성 계약을 노출합니다.

## 지원 거래소와 경계

| 거래소 | 일반 어댑터(Adapter) 경계 | 시작점 |
| --- | --- | --- |
| **Binance** | 현물(Spot), USD-M 무기한 선물 | `BinanceAdapter::spot()` — 이 README의 기본 예제 |
| **Upbit** | 한국·싱가포르·인도네시아·태국 현물 | `UpbitAdapter::new()` 또는 `with_region(...)` |
| **Bithumb** | 현물; KRW 계좌·주문 관련 거래소 전용 API | `BithumbAdapter::new()` |
| **Hyperliquid** | 메인넷·테스트넷 현물과 무기한 선물 | `HyperliquidAdapter::new()` 또는 `testnet()` |

모든 내장 어댑터는 공통 `Client` API의 문서화된 일부를 지원합니다. 공개 시장
데이터와 시장 스트림에는 계정 설정이 필요하지 않습니다. 계좌 조회, 주문, 전송은
거래소별 설정이 필요하며 지역, 거래소 구분(venue), 계정 권한에 따라 제한될 수
있습니다. Hyperliquid에는 공개 주소만 필요하고 로컬 서명은 필요하지 않은 주소 단위
Info 조회도 있으며, 서명 작업에는 signer가 필요합니다. 정확한 생성자와 경계는
[거래소 지원](docs/providers.ko.md), 작업(operation)별 지원·검증 상태는 생성된
[endpoint reference](bindings/common/generated/api.md)를 확인하세요.

Binance 테스트넷(testnet) 생성자, Hyperliquid HIP-3 DEX·결과형 자산(outcome asset)은
노출하지 않습니다. endpoint reference에서 매핑한 작업과 아직 계획 또는 미매핑인
거래소 제품을 구분합니다.

## 만든 이유

`maxt`는 여러 거래소를 함께 사용하는 애플리케이션을 위해 만들었습니다. 거래소나
언어를 바꿀 때마다 새로운 SDK 사용법을 익히지 않는 개발 경험을 지향합니다.

- 거래소와 지원 언어가 달라도 공통 기능은 같은 API 구조, 모델, 오류, 스트림 계약으로 사용합니다.
- 공통 기능은 `Client`, 거래소 전용 기능은 타입이 명확한 어댑터를 통해 제공합니다.
- 하나의 스키마에서 각 언어의 공개 API와 계약을 생성하고, 컴파일된 네이티브 API와의 정합성을 검사합니다.

## 빠른 시작: Binance 현물

기본 예제는 인증 정보 없이 Binance 현물 `BTC/USDT`를 읽습니다. 공통 API인
`ticker`와 Binance 전용 `spot_average_price`를 함께 호출하지만 주문을 제출하지는
않습니다.

## Rust 설치

Rust 1.85 이상이 필요합니다.

```toml
[dependencies]
maxt = "0.2.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust,no_run
use maxt::adapters::BinanceAdapter;
use maxt::{Client, Exchange, Market};

#[tokio::main]
async fn main() -> maxt::Result<()> {
    let client = Client::new(BinanceAdapter::spot());
    let market = Market::spot(Exchange::Binance, "BTC", "USDT");

    let ticker = client.ticker(&market).await?;
    let average = client.adapter().spot_average_price(&market).await?;

    println!("{}", ticker.last_price);
    println!("{}분 평균: {}", average.minutes, average.price);
    Ok(())
}
```

`ticker()`는 공통 API입니다. `spot_average_price()`는 Binance Spot 전용이며
`Client::adapter()`를 통해 호출합니다.

공개 REST 예제 실행:

```sh
cargo run --example public_rest
```

코드를 바꾸지 않고 다른 공개 거래소를 살펴보려면 다음처럼 실행하세요.
`cargo run --example public_rest -- upbit BTC KRW`

## 언어별 패키지

| 언어 | 패키지 안내 | 실행 가능한 Binance 예제 |
| --- | --- | --- |
| Rust | 이 README와 [시작하기](docs/getting-started.ko.md) | [`examples/public_rest.rs`](examples/public_rest.rs) |
| Python | [Python 패키지 안내](bindings/python/README.ko.md) | [`bindings/python/examples/binance_public_ticker.py`](bindings/python/examples/binance_public_ticker.py) |
| Dart / Flutter | [Dart 패키지 안내](bindings/dart/README.ko.md) | [`bindings/dart/example/main.dart`](bindings/dart/example/main.dart) |
| TypeScript | [TypeScript 패키지 안내](bindings/typescript/README.ko.md) | [`bindings/typescript/examples/binance-public-ticker.mjs`](bindings/typescript/examples/binance-public-ticker.mjs) |

Dart 패키지는 Android, iOS, Linux, macOS, Windows, Web을 지원합니다. TypeScript
패키지는 Node.js와 브라우저 WebAssembly를 지원합니다.

## 문서

- [시작하기](docs/getting-started.ko.md)
- [공통 API](docs/common-api.ko.md)
- [거래소 지원](docs/providers.ko.md)
- [endpoint 지원 reference](bindings/common/generated/api.md)
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
