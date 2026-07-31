# 기여하기

[English](CONTRIBUTING.md) | [한국어](CONTRIBUTING.ko.md)

## 개발 환경

`maxt`는 Rust 1.85 이상과 Rust edition 2024를 사용합니다. 저장소는 툴체인을
고정하지 않으며 CI는 stable 툴체인을 사용합니다.

```sh
git clone https://github.com/jabdori/maxt.git
cd maxt
cargo test
```

테스트는 픽스처와 로컬 모의 서버를 사용합니다.

## 검사

풀 리퀘스트 전에 CI와 동일한 검사를 실행합니다.

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

CI는 `main` 브랜치 푸시와 풀 리퀘스트에서 실행됩니다. 라이브 테스트는 기본 검사와
CI에서 제외됩니다.

## 구조

| 경로 | 역할 |
| --- | --- |
| `src/adapter.rs` | 공개 `Adapter` 계약 |
| `src/client.rs` | 공통 API와 정규화 |
| `src/types/` | 마켓·계좌·주문·스트림 타입 |
| `src/adapters/<provider>/` | REST·인증·스트림·파싱 |
| `src/transport/` | HTTP·WebSocket 전송 |
| `tests/` | 계약·통합·라이브 테스트 |
| `docs/` | 공통 계약·제공자별 제약 |

가장 가까운 어댑터의 구조를 따릅니다. 대칭만 맞추려고 빈 파일을 만들지 않습니다.

## 내장 어댑터 체크리스트

1. `Exchange` 항목을 추가하거나 재사용하고 어댑터 모듈을 공개합니다.
2. `exchange()`와 `supports()`를 구현합니다. `supports()`는 인증 정보 유무를 포함한
   현재 인스턴스의 기능을 반환해야 합니다.
3. 지원하는 `Adapter` 메서드만 재정의합니다. 나머지는 기본
   `Error::Unsupported`를 사용합니다.
4. 공통 전송 계층과 `Client`의 정렬·검증·`Option`·`Decimal`·`Timestamp`
   계약을 유지합니다.
5. 파싱·요청·서명·`supports()`·제공자 경계값의 픽스처 테스트를 추가합니다.
6. 기능 계약 테스트, 영어·한국어 제공자 문서, 영향을 받는 예제를 갱신합니다.
7. 전체 검사를 실행합니다. 실제 엔드포인트 동작을 바꾼 경우 라이브 테스트도
   실행합니다.

크레이트 외부의 모의·백테스트·기록 데이터 어댑터도 공개 `Adapter`를 구현할 수
있습니다. [외부 어댑터 계약](docs/common-api.ko.md#외부-어댑터)

## 문서

- 숙련된 Rust 개발자를 기준으로 작성합니다.
- 조건은 풀어쓰지 않습니다. 예: `from <= open_time < to`.
- 공통 계약은 `common-api`, 요청 한도와 거래소 고유 동작은 제공자 문서에 둡니다.
- 영어와 한국어 문서의 구조를 맞춥니다. 문장 순서가 아니라 의미를 번역합니다.

## 라이브 테스트

```sh
cargo test --test live_conformance -- --ignored --nocapture
```

라이브 테스트는 인증 정보 없이 공개 거래소 엔드포인트를 호출합니다. 비공개 계좌·주문
경로는 포함하지 않습니다. 거래소 가용성, 요청 한도, 상장 상태 변경으로 실패할 수
있습니다.

## 보안

- API 키, 비밀 키, 개인 키, 서명된 요청, `.env`, 비공개 페이로드를 커밋하지 않습니다.
- 인증 기능 개발에는 읽기 전용 또는 테스트넷 인증 정보를 사용합니다.
- 픽스처, 로그, 이슈, 풀 리퀘스트에 인증 정보를 남기지 않습니다.
- 노출된 인증 정보는 폐기하고 교체합니다.
