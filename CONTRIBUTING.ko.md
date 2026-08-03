# 기여하기

[English](CONTRIBUTING.md) | [한국어](CONTRIBUTING.ko.md)

## 개발 환경

`maxt`는 Rust 2024 edition과 Rust 1.85 이상을 사용합니다. CI 환경은 최신 stable
Rust, Python 3.14.2, uv 0.10.4, Flutter stable입니다.

```sh
git clone https://github.com/jabdori/maxt.git
cd maxt
cargo test -p maxt -p maxt-bindings-common --all-targets --locked
```

테스트는 픽스처(fixture)와 로컬 모의 서버(mock server)를 사용합니다. 실제 거래소를
호출하는 테스트는 기본 실행에서 제외됩니다.

## CI 검사

변경한 영역의 명령만 실행하세요. 각 바인딩은 독립 CI, Cargo 잠금 파일(lockfile),
배포 태그를 사용합니다.

### Rust workspace

```sh
cargo fmt -p maxt -p maxt-bindings-common -p maxt-bindings-codegen --check
cargo clippy -p maxt -p maxt-bindings-common --all-targets --locked -- -D warnings
cargo clippy -p maxt-bindings-codegen --no-default-features --features rust --all-targets --locked -- -D warnings
cargo clippy -p maxt --lib --locked -- \
  -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic
cargo test -p maxt -p maxt-bindings-common --all-targets --no-default-features --locked
cargo test -p maxt-bindings-codegen --no-default-features --features rust --all-targets --locked
cargo test -p maxt -p maxt-bindings-common --doc --no-default-features --locked
cargo build -p maxt --examples --locked
cargo package -p maxt --locked
```

문서 테스트는 `src/lib.rs`가 포함하는 영어·한국어 Markdown의 Rust 코드 블록을
컴파일합니다.

### Python 바인딩

`bindings/python`에서 실행하세요.

```sh
uv lock --check
uv sync --frozen --all-groups
cargo run --manifest-path ../codegen/Cargo.toml --no-default-features --features python --locked -- python --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --no-default-features --locked
uv run --frozen maturin develop --locked
MAXT_REQUIRE_NATIVE_TESTS=1 uv run --frozen pytest
uv run --frozen mypy python/maxt
uv build --out-dir dist
uv run --frozen twine check dist/*
```

### Dart / Flutter 바인딩

`bindings/dart`에서 실행하세요.

```sh
cargo run --manifest-path ../codegen/Cargo.toml --no-default-features --features dart --locked -- dart --check
cargo clippy --manifest-path rust/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path rust/Cargo.toml --all-targets --locked
dart pub get
cargo install flutter_rust_bridge_codegen --version 2.12.0 --locked
flutter_rust_bridge_codegen generate
perl -pi -e 's/[ \t]+$//' lib/src/rust/*.freezed.dart
cargo fmt --manifest-path rust/Cargo.toml
git diff --exit-code -- lib/src/rust rust/src/frb_generated.rs
test -z "$(git status --porcelain --untracked-files=all -- lib/src/rust rust/src/frb_generated.rs)"
dart format --output=none --set-exit-if-changed .
dart analyze --fatal-warnings
dart test --chain-stack-traces
flutter test
dart pub publish --dry-run
```

CI는 Markdown, Rust, TOML에 기여자 컴퓨터의 절대 경로가 포함된 경우에도
실패합니다.

## 구조

| 경로 | 역할 |
| --- | --- |
| `src/adapter.rs` | Rust `Adapter` 계약 |
| `src/client.rs` | 공통 동작과 정규화 |
| `src/types/` | 시장, 계좌, 주문, 스트림 공통 값 |
| `src/adapters/<provider>/` | 거래소 REST, 스트림, 비공개 API, 응답 파싱(parsing) |
| `src/transport/` | 공통 HTTP·WebSocket 전송 계층 |
| `bindings/common/` | 언어 중립 바인딩 계약 |
| `bindings/python/` | Python 패키지(package), PyO3 브리지(bridge) |
| `bindings/dart/` | Dart 패키지, Rust 브리지, 생성된 브리지 코드 |
| `tests/` | Rust 계약, 통합, 기본 실행에서 제외된 실제 호출 테스트(live test) |
| `docs/` | 공통·거래소 레퍼런스 |

가장 가까운 어댑터의 구조를 따르세요. 구조만 맞추기 위한 빈 파일은 추가하지
마세요.

## 어댑터 체크리스트

1. Rust `Exchange` 항목을 추가하거나 재사용하고 어댑터 모듈을 공개합니다.
2. `exchange()`를 구현하고, 현재 설정된 기능만 `supports()`에서 반환합니다. 비공개
   기능에는 인증 정보 설정 여부가 포함됩니다.
3. 지원하는 `Adapter` 메서드를 재정의합니다. 기본 구현은
   `Error::Unsupported`를 반환합니다.
4. 공통 정렬, 범위, `Option`, `Decimal`, `Timestamp`, 오류, 스트림 수명 주기 계약을
   유지합니다.
5. 파싱, 요청 생성, 서명, 기능, 거래소 한도의 픽스처 테스트를 추가합니다.
6. 각 언어 바인딩은 별도 변경으로 갱신하고 해당 언어 생성 대상을 실행합니다.
   Rust 변경과 모든 바인딩 변경을 하나의 풀 리퀘스트에 묶지 않습니다.
7. 기능 계약 테스트, 거래소 레퍼런스의 영어·한국어 문서, 관련 예제를 갱신합니다.
8. 변경한 영역의 CI 검사만 실행합니다.

## 배포

- `rust-vX.Y.Z`: crates.io
- `python-vX.Y.Z`: PyPI
- `dart-vX.Y.Z`: pub.dev
- `typescript-vX.Y.Z`: npm, Node.js와 브라우저 WebAssembly 포함

Dart 첫 버전은 수동으로 배포한 뒤 pub.dev 태그 패턴을 `dart-v{{version}}`으로
설정합니다. npm 첫 배포에는 `NPM_TOKEN`이 필요합니다. 이후
`release-typescript.yml`에 신뢰할 수 있는 배포(Trusted Publishing)를 설정하고
토큰을 제거합니다.

공개 어댑터 계약은 모의(mock), 백테스트(backtest), 기록 데이터 어댑터에도 사용할 수 있습니다.
[외부 어댑터](docs/common-api.ko.md#외부-어댑터)를 참고하세요.

## 문서

- 개발자에게 필요한 계약을 식별자와 연산자로 작성합니다.
- 공통 계약은 `docs/common-api.ko.md`에 한 번만 정의하고, 거래소 한도와 원본 API
  매핑은 거래소 레퍼런스에 둡니다.
- 영어·한국어 문서의 구조와 계약을 맞춥니다. 문장 순서가 아닌 의미를 옮깁니다.

## Live test

공개 거래소 엔드포인트(endpoint)만 호출하며 인증 정보는 사용하지 않습니다.

```sh
cargo test --test live_conformance -- --ignored --nocapture
```

거래소 가용성, 요청 한도, 상장 변경에 따라 실패할 수 있습니다.

## 보안

- API 키(key), 비밀 값(secret), 개인 키(private key), 서명된 요청, `.env`, 비공개
  거래소 payload를
  커밋하지 않습니다.
- 가능하면 읽기 전용 또는 testnet 인증 정보를 사용합니다.
- 픽스처, 로그(log), 이슈(issue), 풀 리퀘스트(pull request)에 인증 정보를 남기지 않습니다.
- 노출된 인증 정보는 폐기하고 교체합니다.
