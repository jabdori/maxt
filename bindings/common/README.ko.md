# maxt 바인딩 공통 계약

[English](README.md)

`maxt-bindings-common`은 언어 바인딩이 공유하는 요청, 응답, 오류, 스트림과
`ForeignAdapter` 브리지를 정의합니다. 애플리케이션에서는 Rust, Python,
Dart/Flutter, TypeScript 중 사용하는 언어의 패키지를 설치하세요.

## 단일 원본

`src/schema.rs`는 언어별 공개 API 생성의 단일 원본(source of truth)입니다.

| 변경 | 스키마 등록 위치 |
| --- | --- |
| 공통 메서드 | `ADAPTER_OPERATIONS` |
| 거래소 전용 메서드 또는 constructor | `PROVIDERS` |
| enum 또는 열린 식별자 | `IDENTIFIERS` |
| 요청·응답 모델 | `binding_schema()`의 `records` |
| 태그가 있는 오류 | `binding_schema()`의 `unions` |

실제 동작은 Rust 어댑터가 구현합니다. 스키마는 외부 언어 경계를 설명하며 Rust
구현과 일치해야 합니다.

## 스키마 변경 검사

```sh
cargo test -p maxt-bindings-common --features codegen --test schema_inventory --locked
cargo run -p maxt-bindings-codegen --no-default-features --features python --locked -- python --check
```

`python`은 갱신할 바인딩으로 바꾸세요. 스키마 변경으로 생성 결과가 달라지면
먼저 `--check` 없이 생성한 다음 검사합니다.

[코드 생성기 안내](../codegen/README.ko.md),
[공통 API 명세](../../docs/common-api.ko.md),
[기여 안내](../../CONTRIBUTING.ko.md)를 함께 참고하세요.
