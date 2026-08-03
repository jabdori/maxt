# maxt 바인딩 코드 생성기

[English](README.md)

`maxt-bindings-codegen`은 `maxt-bindings-common`에서 언어별 공개 API, 네이티브
façade, 구조적 wire 변환, 계약 목록을 생성하는 저장소 도구입니다. 애플리케이션
종속성은 아닙니다.

## 언어 하나 생성

```sh
cargo run -p maxt-bindings-codegen --no-default-features --features python --locked -- python
```

대상은 `rust`, `python`, `dart`, `typescript`입니다. feature와 마지막 인자는 같은
대상을 지정해야 합니다. 모든 언어를 의도적으로 갱신할 때만 둘 다 생략하세요.

파일을 쓰지 않고 검사하려면 다음을 실행합니다.

```sh
cargo run -p maxt-bindings-codegen --no-default-features --features python --locked -- python --check
cargo test -p maxt-bindings-common --features codegen --test schema_inventory --locked
```

## 생성 파일

| 대상 | 파일 |
| --- | --- |
| Rust | `bindings/common/generated/api.md` |
| Python | `python/maxt/_generated_contract.py`, `_generated_identifiers.py`, `_generated_api.py`, `_generated_delegate.py`, `_generated_wire.py`, `_native.pyi`; `src/generated/client_methods.rs`, `adapter_dispatch.rs`, `convert.rs`, `provider_convert.rs` |
| Dart | `lib/src/generated_contract.dart`, `generated_identifiers.dart`, `generated_adapter.dart`, `generated_client.dart`, `generated_provider_guard.dart`, `generated_provider_methods.dart`, `generated_delegate.dart`, `generated_wire_converters.dart`; `rust/src/api/generated_native_client.rs`, `adapter/generated_dispatch.rs`, `convert/generated_shape_guard.rs` |
| TypeScript | `src/generated/contract.ts`, `identifiers.ts`, `codec.ts`, `api.ts` |

`bindings/`로 시작하지 않는 경로는 해당 바인딩 디렉토리 기준입니다. 생성 파일은
직접 수정하지 마세요.

## 스키마 변경

`bindings/common/src/schema.rs`를 수정합니다.

- 공통 호출은 `ADAPTER_OPERATIONS`, 거래소 전용 호출은 `PROVIDERS`에 추가합니다.
- 닫힌 enum 또는 열린 식별자는 `IDENTIFIERS`에 추가합니다.
- 요청·응답 형식은 `records`, 태그가 있는 오류는 `unions`에 추가합니다.
- 바인딩에서 거래소 어댑터를 생성해야 하면 constructor metadata를 추가합니다.

그다음 갱신할 언어만 생성하고 검사합니다.

## 생성 영역과 수기 영역

모델, enum, 오류, Client·Adapter 메서드 목록, 거래소 전용 dispatch,
`Option`·list·page 매핑, 구조적 wire 변환처럼 반복되는 구조는 생성기가
담당합니다.

언어에 따라 의미가 달라지는 런타임 정책은 수기로 유지합니다. Python의 비동기
실행·GIL·객체 수명, Dart isolate·FRB handle, Node callback·Worker 수명,
브라우저 스트림 backpressure, 취소·종료 동작, 네이티브·WASM 로딩, 인증 정보와
relay 보안, 언어별 정밀도 규칙이 해당합니다.

Dart는 저장소 생성기를 먼저 실행한 다음 FRB를 실행합니다.

```sh
cargo run -p maxt-bindings-codegen --no-default-features --features dart --locked -- dart
cd bindings/dart
flutter_rust_bridge_codegen generate
```
