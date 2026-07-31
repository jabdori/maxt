[English](CONTRIBUTING.md) | [한국어](CONTRIBUTING.ko.md)

# maxt 기여 안내

`maxt`는 네 거래소 앞에 하나의 Rust API를 둡니다.

## 개발 환경 준비

Rust 1.85 이상, edition 2024입니다. 둘 다 `Cargo.toml`에 고정되어 있습니다.

```sh
git clone https://github.com/jabdori/maxt
cd maxt
cargo test
```

거래소 계정도 네트워크도 필요 없습니다. 아래 검사는 전부 오프라인에서 돌고
이미 아는 값을 돌려주면서 연결을 여는 어댑터는 그 자체가 결함입니다.

## 검사 항목

`.github/workflows/ci.yml`이 푸시와 풀 리퀘스트마다 아래를 돌립니다. CI는 잡
전체에 `RUSTFLAGS: -D warnings`를 걸어 두므로 로컬에서도 걸고 시작하세요.

```sh
export RUSTFLAGS="-D warnings"

cargo fmt --all --check                    # 서식
cargo clippy --all-targets -- -D warnings  # 테스트와 예제까지 포함한 린트
cargo test --all-targets                   # 단위·통합 테스트
cargo test --doc                           # --all-targets가 건너뛰는 문서 테스트
cargo build --examples                     # 실행 가능한 예제가 여전히 컴파일되는지
cargo doc --no-deps                        # 문서 빌드와 문서 내 링크 확인
```

하나 더 있습니다. `--lib`는 `cfg(test)` 없이 빌드하므로 배포되는 코드만
검사합니다.

```sh
cargo clippy --lib -- -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic
```

`tests/live_conformance.rs`는 소켓을 열기 때문에 `#[ignore]`가 붙어 있고 위
명령 중 어느 것도 이 테스트를 돌리지 않습니다.
[라이브 적합성 검사](#라이브-적합성-검사)를 보세요.

두 번째 잡 `scope`는 `*.md`, `*.rs`, `*.toml`을 훑어 아래 둘 중 하나라도 있으면
실패시킵니다.

| 거절 대상 | 대신 |
| --- | --- |
| `maxt`가 패키지 레지스트리에 배포되어 있다는 주장. 레지스트리에서 받아 오는 설치 명령, `maxt`의 레지스트리 링크, 호스팅된 API 문서 링크. | README와 [시작하기](docs/getting-started.ko.md)가 그러듯 저장소를 의존성으로 지정합니다. |
| 기여자 홈 디렉터리 밖으로 나가는 절대 경로. | 산문, 주석, 설정의 경로는 저장소 루트 기준 상대 경로입니다. |

## 기능이 갈 곳

| 갈 곳 | 조건 | 구현 방식 |
| --- | --- | --- |
| 공통 API | 의미를 바꾸지 않고 모든 거래소가 실어 나를 수 있을 때 | [`Adapter`](src/adapter.rs)가 메서드를 선언하고 [`Client`](src/client.rs)가 넘겨주며 [`Feature`](src/feature.rs)가 이름을 붙입니다 |
| 어댑터의 고유 메서드 | 공통 API가 실으려면 사실이 아닌 말을 하거나, 호출할 가치를 만든 성질을 버려야 할 때 | `UpbitAdapter`, `BinanceAdapter` 등에 붙는 `pub` 메서드입니다. [`Client::adapter`](src/client.rs)로 접근합니다: `client.adapter().order_books(&markets, Some(5))` |
| 아무 곳도 아닌 곳, 트레이트 기본 구현 | 거래소가 아예 제공하지 않을 때 | 메서드를 구현하지 마세요. `Adapter`는 모든 메서드의 기본값이 `Error::Unsupported`라서, 없는 기능은 호출 지점에서 이름과 함께 보고됩니다 |

없다고 결론짓기 전에 거래소 엔드포인트 목록을 직접 확인하세요. 이 저장소는 없는
기능이라고 잘못 적은 적이 여러 번 있습니다.

고유 메서드는 모두 그 이유를 문서 주석에 적어 두었습니다. 하나 더 만들기 전에 먼저
읽으세요.

| 메서드 | 이유 |
| --- | --- |
| `UpbitAdapter::order_books`, `UpbitAdapter::tickers` | 요청 한 번으로 여러 마켓에 답합니다. `Client::order_book`을 거치면 마켓 30개가 호출 30번이 됩니다. 두 메서드 모두 목록 길이에 상한을 두지 않고 Upbit도 상한을 공시하지 않으므로, 목록이 충분히 길면 `Error::Exchange`가 돌아옵니다. |
| `UpbitAdapter::market_events` | Upbit의 유의 표시와 그보다 약한 주의 표시를 한 번에 읽습니다. `Client::markets`는 유의 표시가 붙은 마켓을 `MarketStatus::Unknown`으로 보고하고 주의 표시는 다루지 않습니다. 주의 사유는 여기서만 읽습니다. |
| `BithumbAdapter::market_warnings` | Bithumb의 유의 종목입니다. 표시가 붙어도 거래는 그대로 열려 있습니다. `MarketStatus`에 해당하는 값이 없어 `Client::markets`는 `Unknown`으로 보고하고 표시 문구는 여기에 원문 그대로 남습니다. |
| `BithumbAdapter::market_alerts` | Bithumb의 주의 종목입니다. 엔드포인트가 따로 있고 사유와 단계, 종료 시점을 함께 싣습니다. `MarketStatus`에는 셋 다 없고 이 지정으로 마켓이 `Active`에서 벗어나지도 않습니다. |
| `BinanceAdapter::spot_symbol_filters` | 호가 단위, 수량 단위, 최소 주문 금액이 주문 접수 여부를 가르는데 거래소마다 표현 방식이 다릅니다. 타입도 Binance 모양 그대로 둡니다. |
| `BinanceAdapter::spot_order` | 체결된 주문과 취소된 주문에도 답합니다. `Client::open_orders`는 정의상 그럴 수 없습니다. |
| `BinanceAdapter::usd_m_create_listen_key`, `usd_m_keepalive_listen_key`, `usd_m_close_listen_key` | `Client::subscribe_account`가 이 수명 주기를 이미 돌립니다. 이 셋은 소켓을 직접 몰 때, 그러니까 키 하나를 여러 소비자가 나눠 쓰거나 재시작을 넘겨 유지할 때 씁니다. |
| `HyperliquidAdapter::non_funding_ledger` | 입금, 출금, 이체, 청산은 어느 마켓에도 속하지 않습니다. `FundingPayment`로 보고하려면 스친 적도 없는 마켓 이름을 붙여야 합니다. |
| `HyperliquidAdapter::asset_context` | `FundingRate`는 이미 부과된 펀딩을 기록합니다. 미결제약정과 oracle 가격은 대응하는 공통 개념이 아예 없습니다. |

고유 메서드를 새로 만든다면 같은 문장을 그 메서드에도 써야 합니다. 정직한 답이
"공통 API에 넣어도 잘 맞지만 여기에 먼저 썼을 뿐"이라면, 모든 어댑터의 공통
API에 넣으세요.

### `supports()`는 사실이어야 합니다

`Adapter::supports`는 기능 확인, 라우팅 로직, 거래소별 문서가 모두 읽는 값입니다.
[`tests/unsupported_is_honest.rs`](tests/unsupported_is_honest.rs)가 `Feature`
전체와 어댑터 구성의 곱집합을 양방향으로 검사합니다.

- `supports(f) == false`면 그 호출은 같은 `f`를 지목하는 `Error::Unsupported`로
  거절해야 합니다. 전송 오류도, 성공도, 다른 기능 이름도 안 됩니다. 정직한 거절이
  하나 더 있는데, 인증 정보가 없을 때 나오는 `Error::Auth`입니다.
- `supports(f) == true`면 그 호출은 `Unsupported`로 답하지 않아야 합니다.
  `offline_probe`는 회선에 나가기 전에 판정이 끝나는 입력을 씁니다. 다른 거래소의
  마켓, 형식이 틀린 지갑 주소가 그런 입력입니다. 그런 입력이 없는 자리에서는
  `None`과 이유를 함께 돌려주고 프로브 개수 하한이 테스트가 텅 비는 일을 막습니다.

이 파일은 네트워크를 전혀 건드리지 않습니다.

호출에 넘기는 인자는 여기서 검사하지 않습니다. `Feature::Candles`는 모든
어댑터에서 `true`이지만 그 거래소가 다루지 않는 간격은 여전히 `Unsupported`입니다.
간격은
`every_baseline_interval_is_mapped_on_the_exchanges_that_can_be_asked_offline`이
맡고 이 테스트의 `BASELINE_INTERVALS`는 어댑터가 아니라 네 거래소 문서에서 읽어
옵니다. 프로브로 다른 거래소의 마켓을 쓰기 때문에 심볼 표를 먼저 만드는
Hyperliquid는 빠지고, 간격 표는 자체 유닛 테스트가 단언합니다. 이름을 바꿀
때도 이 한계를 그대로 담으세요.

`supports()`는 손으로 나열하지 말고 `Feature`의 헬퍼 두 개를 기준으로 쓰고
예외마다 주석을 다세요. `supports()`는 지금 만들어 둔 그대로의 어댑터를 두고
답하므로, 인증 정보 없이 만든 어댑터는 인증 정보가 필요한 기능에 전부 `false`를
답합니다.

```rust
use maxt::Feature;

struct ExampleAdapter {
    credentials: Option<(String, String)>,
}

impl ExampleAdapter {
    fn supports(&self, feature: Feature) -> bool {
        if feature.is_derivatives_only() {
            return false;
        }
        // Bithumb의 공개 WebSocket은 체결, 호가창, 티커를 실어 나르지만
        // 캔들은 없습니다.
        if matches!(feature, Feature::CandleStream) {
            return false;
        }
        if feature.needs_credentials() {
            return self.credentials.is_some();
        }
        true
    }
}

fn check() {
    let public = ExampleAdapter { credentials: None };
    let keyed = ExampleAdapter {
        credentials: Some(("access".to_string(), "secret".to_string())),
    };

    // 공개 시세는 양쪽에 열려 있고 주석을 달아 둔 예외 하나는 양쪽에서
    // 닫혀 있습니다.
    assert!(public.supports(Feature::Ticker));
    assert!(!public.supports(Feature::CandleStream));
    assert!(!keyed.supports(Feature::CandleStream));

    // 인증 조건, 그리고 키로는 열리지 않는 파생상품 조건.
    assert!(!public.supports(Feature::Balances));
    assert!(keyed.supports(Feature::Balances));
    assert!(!keyed.supports(Feature::Positions));
}
```

지원하지 않는 기능이 아닌 경우가 하나 있습니다. 현물 거래소에 무기한 선물 목록을
물으면 오류가 아니라 빈 목록이 돌아옵니다.

## 거래소 추가하기

`src/adapters/bithumb/`이 네 어댑터 중 가장 작고 나머지와 같은 모양입니다.
시작하기 전에 처음부터 끝까지 읽으세요.

1. **`Exchange` 배리언트를 추가합니다.** `src/types/market.rs`에 있습니다.
   `Exchange::id`와 `Exchange::display_name`은 모든 경우를 나열하는 `match`이므로,
   컴파일러가 바꿀 자리를 전부 짚어 줍니다.

2. **`src/adapters/<name>/`을 만들고 관심사별로 파일을 나눕니다.** 더 쪼개는 것은
   이유가 있을 때만 합니다. Hyperliquid는 헤더가 아니라 지갑 키로 서명하므로 서명이
   `sign.rs`에 따로 있고 거래소 모양 그대로인 공개 타입 두 개는 `native.rs`에
   있습니다.

   | 파일 | 내용 |
   | --- | --- |
   | `mod.rs` | 어댑터 타입, 생성자와 인증 정보, `impl Adapter` |
   | `rest.rs` | 공개 REST: `HttpRequest`를 돌려주는 요청 생성 함수와 그 요청을 보내는 호출 |
   | `private.rs` | 서명이 붙는 REST 호출과 그 서명 로직 |
   | `stream.rs` | WebSocket 구독 프레임과 프레임 해석 |
   | `parse.rs` | 거래소의 페이로드 타입과 `maxt` 타입으로의 변환 |

3. **`src/adapters/mod.rs`에 등록합니다.** 비공개 `mod <name>;`을 넣고 어댑터와
   어댑터가 돌려주는 거래소 고유 공개 타입을 `pub use`로 내보냅니다.

4. **`Adapter`를 구현합니다.** `exchange()`와 `supports()`는 필수입니다. 거래소가
   실제로 제공하는 메서드만 구현하고 나머지는 기본 구현으로 둡니다. 생성자는
   실패하지 않게 두세요. HTTP 전송을 만드는 일은 TLS 백엔드가 초기화를 거부할 때만
   실패하므로, 그 실패는 타입 안에 담아 두었다가 네트워크가 필요한 첫 호출에서
   보고하세요.

   ```rust
   use maxt::{Adapter, BoxFuture, Exchange, Feature, MarketInfo, MarketKind, Result};

   struct ExampleAdapter;

   impl Adapter for ExampleAdapter {
       fn exchange(&self) -> Exchange {
           Exchange::Upbit
       }

       fn supports(&self, feature: Feature) -> bool {
           matches!(feature, Feature::Markets)
       }

       fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
           let _ = kind;
           Box::pin(async move { Ok(Vec::new()) })
       }
   }

   fn check() {
       assert_eq!(ExampleAdapter.exchange(), Exchange::Upbit);
       assert!(ExampleAdapter.supports(Feature::Markets));
       // 트레이트 기본값으로 남겨 둔 것은 이 어댑터가 한 줄도 쓰지 않아도
       // 스스로 없다고 이름까지 밝혀 보고합니다.
       assert!(!ExampleAdapter.supports(Feature::Ticker));
   }
   ```

5. **`reqwest`나 `tokio_tungstenite`를 직접 쓰지 말고 `src/transport/`를
   거치세요.** REST에는 `HttpTransport`와 `HttpRequest`, 소켓에는 `ws::connect`를
   씁니다. 재연결, 하트비트, 호출자가 지정한 오버플로 정책은
   `src/transport/ws.rs`에 있습니다. 요청은 `HttpRequest`를 돌려주는 평범한 함수로
   만드세요. 그래야 경로와 쿼리, 거절을 네트워크 없이 테스트합니다.

6. **테스트를 작성합니다.** 코드 옆의 `#[cfg(test)] mod tests` 안에 둡니다.

   | 테스트 | 요건 |
   | --- | --- |
   | 파싱 | 거래소가 문서에 실은 예제를 `const` 문자열로 그대로 넣고, 바로 위 주석에 문서 URL을 답니다. 거래소가 참조 페이지를 내놓지 않으면 URL 자리에 그 사정을 적으세요. `upbit/parse.rs`의 오류 응답이 그런 경우입니다. 본인 계좌에서 받은 응답은 붙여 넣지 마세요. |
   | 요청 생성 | 경로, 쿼리, 범위를 벗어난 `limit`이 요청이 만들어지기 전에 걸러지는지. |
   | 서명 벡터 | 거래소가 직접 공개한 예제를 씁니다. `binance/private.rs`는 Binance가 문서에 실은 키·쿼리·서명과 대조하고 `hyperliquid/sign.rs`는 문서의 키가 문서의 주소를 만들어 내는지 확인합니다. 거래소가 아무것도 공개하지 않으면 거래소가 검증하는 방식 그대로 검증하세요. `upbit/private.rs`는 자신이 만든 JWT를 되읽어, 서명에 쓴 시크릿으로는 검증되고 다른 키로는 검증되지 않음을 확인합니다. |
   | `supports()` | 그룹별로 하나씩. 현물 거래소가 파생 기능을 거절하는지, 비공개 쪽이 인증 정보가 있을 때만 열리는지, 공개 시세는 인증 정보 없이도 열려 있는지. |
   | 비공개 호출이 네트워크 이전에 실패하는지 | 인증 정보가 없는 어댑터의 모든 계좌 호출은 `Error::Auth`를 돌려줍니다. |
   | `tests/unsupported_is_honest.rs` | 여기서 쓰지 않습니다. 7단계에서 다룹니다. |

7. **[`tests/unsupported_is_honest.rs`](tests/unsupported_is_honest.rs)에
   등록합니다.** 손댈 곳은 두 군데뿐이고 어댑터마다 반드시 써야 하는 테스트도
   없습니다.

   `upbit()`, `bithumb()`, `binance()`, `hyperliquid()` 옆에 `Case`를 돌려주는
   생성자를 하나 더하고 그 어댑터의 구성마다 `every_configuration()`에 밀어
   넣으세요. 익명 구성과 인증 정보를 넣은 구성은 별개의 케이스이고 둘 다 여기에
   들어갑니다. `binance()`는 거래 시장이 갈리는 경우까지 보여 줍니다.

   | `Case` 필드 | 값 |
   | --- | --- |
   | `name` | 실패가 지목하는 이름, 구성마다 하나씩: `"upbit"`과 `"upbit+keys"` |
   | `client` | `boxed(..)`를 거친 어댑터. 그래서 모든 케이스가 한 타입입니다 |
   | `market` | 이 거래소가 실제로 상장한 마켓 |
   | `elsewhere` | 상장하지 않은 마켓. 회선 앞에서 멈춰야 하는 프로브에 씁니다 |
   | `checks_markets_offline` | 상장되지 않은 마켓을 연결 없이 거절하는지. 심볼 표를 먼저 만드는 Hyperliquid는 `false`입니다 |
   | `checks_credentials_offline` | 형식이 틀린 인증 정보를 연결 전에 거절하는지. Hyperliquid의 지갑 주소처럼 틀릴 만한 형식이 있는 경우가 아니면 `false`입니다 |
   | `credentialed` | 이 구성에 인증 정보를 주었는지 |

   그다음 `missing_credentials_read_the_same_way_on_every_exchange`의 네 칸 배열에
   어댑터를 더하세요. 이 파일에서 `every_configuration()`이 아닌 목록은 이것
   하나뿐입니다. 길이가 타입에 박혀 있으므로 컴파일러가 그 자리를 짚어 줍니다.

   그 밖에는 새 테스트가 필요하지 않습니다.
   `a_feature_an_adapter_declines_is_declined_by_the_call_behind_it`이 현물 전용
   구성 전부에서 파생상품 기능 전부를 이미 덮고
   `every_private_feature_is_closed_until_credentials_are_supplied`도 같은 목록을
   순회합니다. 공통 호출 하나를 어떤 거래소가 유독 다르게 답한다면, 그때만 이름을 붙여
   테스트를 따로 쓰세요.
   `hyperliquid_serves_recent_trades_over_rest_as_well_as_live`가 그런 경우입니다.

   거래소에 물어 확인한 것만 이름에 담고 문서에 없다는 이유로 짐작한 부재는 담지
   마세요. 그 테스트가 왜 있느냐면, 이 파일이 한동안 멀쩡히 살아 있는 엔드포인트를 두고
   Hyperliquid의 info 레퍼런스에 없다는 이유만으로 없다고 단언했기 때문입니다.

   `Feature` 배리언트를 더하는 것은 같은 파일에 하는 별개의 변경입니다.
   `ALL_FEATURES`는 길이가 고정된 배열이고 `call`에는 기능마다 `match` 갈래가
   있으므로, 새 배리언트를 연결하기 전까지 둘 다 컴파일되지 않습니다.

8. **`docs/providers/`에 영어와 한국어로 거래소 문서를 추가합니다.** 두 언어의
   내용이 같아야 하고 지원 여부에 관한 모든 서술은 `supports()`와 일치해야
   합니다.

9. **예제가 여전히 빌드되는지 확인합니다.** `cargo build --examples`. 어울리는
   예제 하나에 새 거래소를 넣으세요. 거래소마다 예제를 하나씩 만들지는 않습니다.

## 지켜야 하는 규칙

부탁이 아니라 강제입니다.

| 규칙 | 강제 수단 |
| --- | --- |
| 테스트 모듈 밖에서 `unwrap`, `expect`, `panic!`을 쓰지 않습니다. `Error`를 돌려주세요. | 위의 `cargo clippy --lib` 명령 |
| `unsafe`는 금지입니다. | `Cargo.toml`의 `[lints.rust] unsafe_code = "forbid"`. 컴파일러가 거부하고 `forbid`는 `allow`로 뒤집히지 않습니다. |
| 모든 공개 항목에 문서를 답니다. | `Cargo.toml`의 `missing_docs = "warn"` |
| 공개 열거형에는 `#[non_exhaustive]`를 붙여, 나중에 배리언트를 추가해도 호출자가 깨지지 않게 합니다. | 리뷰 |
| 금액은 `rust_decimal::Decimal`이고 `f64`는 쓰지 않습니다. `serde_json::Number`가 들고 있는 텍스트에서 만들고 부동소수점에서 만들지 마세요. | 리뷰 |
| 주석은 무엇이 아니라 왜를 설명합니다. 요청이 왜 그 모양인지, 거래소의 어떤 동작이 그렇게 만들었는지, 그것이 바뀌면 무엇이 깨지는지. | 리뷰 |

`serde_json`은 `arbitrary_precision`으로 설정되어 있어 `serde_json::Number`가
거래소가 보낸 자릿수를 그대로 들고 있습니다. `1386929.37231066771348207123`과
`313245537093.97363`이 테스트 스위트에 있는 이유는 둘 다 `f64`를 통과하지 못하기
때문입니다. 거래소 페이로드가 지나는 경로에는 `f64`가 하나도 없습니다.

## 실제 거래소를 상대로 시험할 때

네 거래소 모두 공개 기능은 인증 정보 없이 동작합니다. 서명이 붙는 경로를 고칠
때만 계좌에 손을 대고, 그것을 테스트 스위트에는 넣지 마세요. 저장소의 테스트는
오프라인에서 돕니다.

| 거래소 | 테스트넷 |
| --- | --- |
| Hyperliquid | 있고 `maxt`가 지원합니다. 호스트가 다르고 서명 도메인도 다릅니다. |
| Binance | Binance는 제공하지만 `maxt`는 연결해 두지 않았습니다. `BinanceMarket::rest_base_url`은 운영 호스트를 돌려주고 이를 바꿀 방법이 없으므로, 지금 `maxt`에 넣은 Binance 인증 정보는 실계좌에 작용합니다. 테스트넷 생성자를 추가하는 변경은 환영합니다. |
| Upbit, Bithumb | 아예 공개하지 않습니다. 두 거래소의 비공개 경로는 실제 자금이 든 실계좌를 상대합니다. |

```rust
use maxt::Client;
use maxt::adapters::HyperliquidAdapter;

let client = Client::new(HyperliquidAdapter::testnet().with_wallet(
    "0x0000000000000000000000000000000000000000",
    "0x0123456789012345678901234567890123456789012345678901234567890123",
));
assert!(client.adapter().is_testnet());
```

테스트넷 서명은 메인넷 다이제스트에서 복원되지 않습니다.
`hyperliquid/sign.rs::mainnet_and_testnet_signatures_are_not_interchangeable`이
이를 단언합니다. 계좌 자체의 키보다 승인된 API 지갑 키를 쓰세요. 거래는 되지만
출금은 안 됩니다. Upbit과 Bithumb에서는 권한을 최소로 묶은 키를 쓰세요. 지금 고치는 부분을
재현할 만큼만 열어 두면 됩니다.

### 라이브 적합성 검사

명령 하나입니다. 저장소에서 연결을 여는 것도 이것뿐입니다.

```sh
cargo test --test live_conformance -- --ignored --nocapture
```

검사 대상은 스스로 정합니다. 거래소 구성마다 스트리밍 기능 네 가지를
`Client::supports`에 묻고 `true` 하나가 구독 하나가 됩니다. 호가창 스트림을 어느
깊이까지 붙잡을지는 그 거래소 제공자 문서의 `Feed::OrderBook` 행에서 읽어 옵니다.
그래서 문서가 약속한 깊이를 바꾸면 단언하는 값도 함께 바뀝니다.

| 주장 | 확인 방법 |
| --- | --- |
| 그 피드를 싣는다 | 그 피드 자신의 이벤트 종류가 0이 아닌 개수로 도착. 구독 성공은 소켓이 열렸다는 뜻일 뿐입니다 |
| 그 피드가 해독된다 | `Err` 항목 0개 |
| 캔들 스트림이 봉을 닫는다 | `Min1` 봉 경계 두 번에 걸쳐 `Candle::closed`가 붙은 이벤트가 최소 하나 |
| 호가창 스트림 깊이가 문서와 같다 | 모든 이벤트가 제공자 문서에 적힌 한 쪽당 단계 수를 싣는지 |
| 시계가 시계다 | 모든 타임스탬프가 읽는 기계 기준 5분 전에서 30초 후 사이. 스트림과, 시계를 싣는 공개 REST 호출 세 개 모두 |

이 벽시계 허용 범위는 배율을 잘못 읽은 필드, UTC 자리에 현지 벽시계를 담은 필드,
기계 자신의 시계가 틀린 경우를 잡아냅니다. 손으로 쓴 픽스처만 보는 단위 테스트에는
셋 다 정상으로 보입니다.

| 항목 | 수치 |
| --- | --- |
| 실행 시간 | 약 3분. 모든 구독을 동시에 연 채 150초, 여기에 접속 시간과 REST 호출이 더해집니다 |
| 왜 150초인가 | 검사가 보고 있는 동안 `Min1` 봉이 열리고 닫혀야 하고 150초는 1분 중 어느 지점에서 시작하든 경계를 두 번 넘습니다 |
| 연결 | `Feed`와 거래소 쌍마다 하나. 오류가 여러 피드를 실은 소켓이 아니라 피드 하나에 귀속되게 하기 위해서입니다 |
| 마켓 | 거래 시장마다 가장 활발한 마켓 하나. 개수 0이 한산한 시간이 아니라 죽은 피드를 뜻하도록 |
| 인증 정보 | 하나도 읽지 않고 읽어도 쓰지 않습니다 |

깨끗하게 끝나면 `24 of 24 checks passed`가 나옵니다. **모든 쌍이 통과해야 하고,
빨간 줄을 그대로 두는 것은 허용하지 않습니다.** 빨간 줄은 아니라고 밝혀내기 전까지
회귀로 취급하세요.

쌍마다 한 줄씩 찍고 실패는 쌍의 이름과 그 근거가 된 숫자를 함께 말합니다.
아래 둘은 단위 테스트가 전부 통과하는 상태에서 그대로 빠져나갔던 결함입니다.

```text
bithumb OrderBook          FAIL  0 events, 1087 errors, levels none; no event of this feed's own kind arrived; first error: could not read exchange response: `timestamp` is not a millisecond timestamp: 1785401551669967
hyperliquid Candles(Min1)  FAIL  125 events, 0 errors, 0 settled, clock off by 61423ms at worst; no Min1 candle closed across 150s and at least two window boundaries
```

가장 큰 숫자가 아니라 문구를 읽으세요. 둘째 줄의 결함은 `0 settled`이고,
`clock off by 61423ms`는 이 검사가 허용하는 범위 안입니다.

| 실패 문구 | 뜻 |
| --- | --- |
| `N errors, first error: ...` | 프레임은 오는데 `maxt`가 읽지 못합니다. 파싱 결함이거나 거래소가 페이로드를 바꾼 것입니다 |
| `0 events, 0 errors` | 구독은 받아들여졌는데 아무것도 오지 않았습니다. 네트워크보다 엔드포인트를 먼저 의심하세요. Binance USD-M은 지금 붙어 있는 진입점에 없는 스트림이라도 `SUBSCRIBE`를 받아들입니다. 그러고는 오류도 종료도 없이 영원히 아무것도 보내지 않습니다. 어댑터가 [해당 스트림을 싣는 진입점](docs/providers/binance.ko.md#usd-m의-두-진입점)으로 보내기 전까지 USD-M의 `Ticker`와 `Candles(Min1)`이 죽은 것처럼 보였던 이유가 이것입니다. `maxt`의 결함이라고 부르기 전에 같은 대상을 REST로 물어보고 raw 소켓으로 프레임을 세어 보세요 |
| `0 settled` | 프레임은 오는데 어느 봉도 끝났다고 알려 주지 않습니다. `Candle::closed`는 다른 어느 것도 확인하지 않는 약속입니다 |
| `clock off by ...` | 타임스탬프가 벽시계에서 멀리 있습니다. 기계 자신의 시계를 먼저 보고 그다음 그 필드의 문서상 배율을 보세요 |

### 검사하지 않는 것

| 검사하지 않는 것 | 이유 |
| --- | --- |
| 비공개 쪽 전부: 잔고, 미체결 주문, `place_order`, `cancel_order`, `subscribe_account` | 공개 읽기 전용 엔드포인트만 씁니다. 서명에는 실제 키가 필요하고 Upbit과 Bithumb은 테스트넷을 내놓지 않으므로, 주문을 넣는 검사는 실제 돈을 씁니다 |
| Hyperliquid 테스트넷 | 같은 어댑터를 다른 호스트에 겨눈 것입니다. 호가창과 캔들이 얇아 개수 0이 아무것도 말해 주지 못합니다 |
| 거래 시장마다 마켓 하나를 뺀 나머지, `Min1`을 뺀 나머지 간격 | 유동성 있는 마켓 하나와 공통으로 가장 짧은 간격이라야 3분 안에서 개수 0과 봉 경계가 뜻을 가집니다 |
| `ticker`, `order_book`, `trades` 바깥의 REST | 시계를 싣는 REST는 이 셋뿐입니다. 페이징, 마켓 목록, 펀딩 내역은 오프라인에서 검사합니다 |
| 가격이나 수량이 맞는지 | 이벤트를 세고 시계를 읽을 뿐입니다 |
| 피드가 계속 살아 있는지 | 돌아간 3분만 보고할 뿐, 다음 한 시간은 아무 말도 하지 않습니다 |

## 절대 커밋에 들어가서는 안 되는 것

- API 키, 시크릿 키, 지갑 개인 키, JWT, listen key.
- 회선에서 캡처한 서명된 요청. 서명은 시크릿과 요청으로부터 함께 만들어집니다.
- 실계좌의 응답 일체: 잔고, 주문 내역, 원장 항목, 주소. 테스트 페이로드는 거래소가
  문서에 실은 예제에서 출발합니다. 예제가 같게 만들어 둔 두 필드를 테스트가 달리
  써야 할 때는 고쳐도 됩니다. 대신 `binance/stream.rs`처럼 주석에 그 사정을
  적으세요.
- `.env` 파일. `.gitignore`가 이미 `/.env`, `/.env.*`, `*.pem`, `*.key`를
  제외합니다. 우회하지 마세요.

인증 정보를 이미 푸시했다면 먼저 거래소에서 키를 교체하세요. Git 히스토리는 비밀을
지울 수 있는 곳이 아닙니다.
