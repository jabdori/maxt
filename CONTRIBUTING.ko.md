# 기여하기

[English](CONTRIBUTING.md) | [한국어](CONTRIBUTING.ko.md)

## 개발 환경

`maxt`는 Rust 1.85 이상이 필요하며 Rust edition 2024를 사용합니다. 저장소는
툴체인을 고정하지 않으며 지속적 통합(CI)은 현재 안정 버전(stable) 툴체인을
사용합니다.

```sh
git clone https://github.com/jabdori/maxt.git
cd maxt
cargo test
```

테스트는 거래소 엔드포인트 대신 픽스처와 로컬 모의 서버를 사용합니다. Cargo 캐시가
비어 있으면 Rust 의존성을 다운로드할 수 있습니다.

## 검사

풀 리퀘스트를 열기 전에 CI와 같은 검사를 실행하세요.

```sh
export RUSTFLAGS="-D warnings"
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo test --doc
cargo build --examples
cargo doc --no-deps
cargo clippy --lib -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic
```

CI는 `main` 브랜치 푸시와 풀 리퀘스트에서 실행됩니다. 실제 거래소를 사용하는
테스트는 기본적으로 무시되며 CI에 포함되지 않습니다.

## 구조

- `src/adapter.rs`: 공개 어댑터 계약
- `src/client.rs`: 공통 API 의미와 정규화
- `src/types/`: 공유 마켓, 계좌, 주문, 스트림 타입
- `src/adapters/<provider>/`: 제공자별 REST, 비공개 호출, 스트림, 파싱 코드
- `src/transport/`: 공유 HTTP와 WebSocket 전송 계층
- `tests/`: 계약 테스트, 통합 테스트, 기본적으로 무시되는 라이브 테스트
- `docs/`: 공통 레퍼런스와 제공자별 제약

제공자 내부 구현에는 서명이나 네이티브 프로토콜 모듈이 추가될 수 있습니다. 형태를
맞추기 위한 빈 파일을 만들지 말고 가장 가까운 어댑터의 구조를 따르세요.

## 어댑터 체크리스트

이 크레이트에 거래소 어댑터를 추가할 때는 다음을 확인하세요.

1. `Exchange` 열거형 항목(variant)을 추가하거나 기존 항목을 재사용하고 어댑터 모듈을
   공개합니다.
2. `exchange`를 구현하고, 인증 정보 유무를 포함한 현재 어댑터 구성에 맞게
   `supports`를 정확히 보고합니다.
3. 지원하는 `Adapter` 메서드만 재정의합니다. 선택적 메서드는 이미
   `Error::Unsupported`를 반환합니다.
4. 공유 전송 계층을 사용하고 공통 정렬·검증·`Option`·`Decimal`·`Timestamp`
   계약을 지킵니다.
5. 파싱, 요청, 서명, 기능 보고, 제공자별 경계값에 대한 픽스처 테스트를 추가합니다.
6. 기능 테스트, 영어와 한국어 제공자 페이지, 영향을 받는 예제를 갱신합니다.
7. 위의 모든 검사를 실행합니다. 네트워크 사용을 의도한 경우에만 라이브 테스트를
   실행합니다.

공개 트레이트(trait)는 이 크레이트 밖의 모의 객체, 기록 데이터 어댑터, 백테스트에서도
구현할 수 있습니다.

```rust
use maxt::{Adapter, BoxFuture, Exchange, Feature, MarketInfo, MarketKind};

struct EmptyUpbit;

impl Adapter for EmptyUpbit {
    fn exchange(&self) -> Exchange {
        Exchange::Upbit
    }

    fn supports(&self, feature: Feature) -> bool {
        matches!(feature, Feature::Markets)
    }

    fn markets(&self, _kind: MarketKind) -> BoxFuture<'_, maxt::Result<Vec<MarketInfo>>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}
```

`exchange`와 `supports`는 필수입니다. 모든 작업에는 기본
`Error::Unsupported` 구현이 있습니다. 외부 어댑터도 `Client`에 문서화된 정렬,
검증, 정규화 계약을 지켜야 합니다. 실제 거래소를 새로 추가하려면 `maxt`에 새로운
`Exchange` 열거형 항목이 필요합니다.

## 라이브 테스트

기본적으로 무시되는 적합성 테스트는 공개 거래소 엔드포인트에 연결하며 인증 정보는
필요하지 않습니다.

```sh
cargo test --test live_conformance -- --ignored --nocapture
```

2026-07-31 기준으로 Upbit 한국 `BTC/KRW`, Bithumb `BTC/KRW`, Binance Spot
`BTC/USDT`, Binance USD-M `BTC/USDT` 무기한 선물, Hyperliquid 메인넷
`BTC/USDC` 무기한 선물의 대표적인 공개 REST와 스트리밍 동작을 검사합니다.
비공개 계좌나 거래 작업은 실시간으로 검사하지 않습니다. 거래소 가용성, 요청 한도,
마켓 변경으로도 테스트가 실패할 수 있습니다.

## 보안

- API 키, 비밀 키, 개인 키, 서명된 요청, `.env` 파일, 비공개 거래소 페이로드를
  커밋하지 마세요.
- 제공자가 지원한다면 비공개 경로를 개발할 때 읽기 전용 또는 테스트넷 인증 정보를
  사용하세요.
- 픽스처, 로그, 이슈, 풀 리퀘스트에 비밀정보를 남기지 마세요.
- 노출된 인증 정보는 폐기하고 교체하세요.
