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
cargo run --manifest-path ../codegen/Cargo.toml --no-default-features --features python --locked -- python
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
cargo run --manifest-path ../codegen/Cargo.toml --no-default-features --features dart --locked -- dart
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
6. `bindings/common/src/schema.rs`에 거래소, 작업, 식별자, 레코드, 오류를
   등록합니다. 갱신할 언어의 생성 대상만 실행하세요. 생성 대상인 공개 API와
   구조 변환을 직접 포팅하지 마세요. Rust 변경과 모든 바인딩 변경을 하나의
   풀 리퀘스트에 묶지 않습니다.
7. 기능 계약 테스트, 거래소 레퍼런스의 영어·한국어 문서, 관련 예제를 갱신합니다.
8. 변경한 영역의 CI 검사만 실행합니다.

## 공식 API 목록

거래소 operation을 추가하기 전에는
[`bindings/common/catalog`](bindings/common/catalog/README.ko.md)의 고정된 공식 원본과
coverage 연결표를 먼저 갱신하세요. 원본 목록은 문서에 공개된 전체 operation을 기록하고,
`src/coverage.rs`는 구현됐거나 의도적으로 계획된 공개 surface만 기록합니다. 요청·응답·오류의
의미가 기존 `Adapter`/`Client`와 정확히 같을 때만 공통으로 분류하고, 그렇지 않으면 거래소 전용
typed API로 유지하세요. 단지 미구현이라는 이유로 일반 operation을 플랫폼 제한으로 분류하면 안
됩니다.

공통 `Adapter`, 공통 타입, `bindings/common/src/schema.rs` 변경은 거래소별 병렬 구현 전에
한 번만 결정하세요. Rust 제품군이 안정화된 뒤 바인딩을 생성하고, 최종 전체 빌드 전에는 provider
문서를 갱신하세요. 선택한 제품군, 생성 바인딩, 문서, 최종 검증이 모두 끝난 뒤에만 릴리스 태그를
생성하거나 푸시합니다.

## 배포

- `rust-vX.Y.Z`: crates.io
- `python-vX.Y.Z`: PyPI
- `dart-vX.Y.Z`: pub.dev
- `typescript-vX.Y.Z`: npm, Node.js와 브라우저 WebAssembly 포함

각 레지스트리 배포는 해당 언어 태그로 시작합니다. 태그 버전과 패키지
manifest 버전이 일치해야 합니다.

릴리스 태그는 반드시 하나씩 푸시하세요. GitHub는 한 번에 세 개를 초과하는
태그를 푸시하면 push 이벤트를 만들지 않습니다.

## 생성 파일

첫 줄에 생성 파일이라고 표시된 파일은 직접 수정하지 마세요. 전체 목록과 생성
순서는 [바인딩 코드 생성기 안내](bindings/codegen/README.ko.md)에 있습니다.
비동기 실행, 객체 수명, callback, 스트림 취소, 네이티브 로딩, 브라우저 보안 같은
Python·Dart 런타임 정책은 수기로 유지합니다.

공개 어댑터 계약은 모의(mock), 백테스트(backtest), 기록 데이터 어댑터에도 사용할 수 있습니다.
[외부 어댑터](docs/common-api.ko.md#외부-어댑터)를 참고하세요.

## 문서

- 개발자에게 필요한 계약을 식별자와 연산자로 작성합니다.
- 공통 계약은 `docs/common-api.ko.md`에 한 번만 정의하고, 거래소 한도와 원본 API
  매핑은 거래소 레퍼런스에 둡니다.
- 공개 소스 주석, 생성 API 문서 템플릿, 배포 예제는 영어로 작성합니다. 한국어 설명은
  `*.ko.md` 파일에 둡니다.
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
