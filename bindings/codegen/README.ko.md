# maxt 바인딩 코드 생성기

[English](README.md)

`maxt-bindings-codegen`은 `maxt-bindings-common`의 바인딩 스키마에서 언어별
계약을 생성합니다. 애플리케이션에서 설치하는 패키지가 아니라 저장소 개발
도구입니다.

## 계약 생성

```sh
cargo run -p maxt-bindings-codegen --locked
```

TypeScript, Python, Dart가 공유하는 거래소, 기능, 오류, Adapter, Client,
provider 전용 API, wire DTO 계약을 생성합니다. 생성 파일은 직접 수정하지
마세요.

## 생성 결과 검사

```sh
cargo run -p maxt-bindings-codegen --locked -- --check
cargo test -p maxt-bindings-common --features codegen --test schema_inventory --locked
```

첫 번째 명령은 생성 파일이 오래되면 실패합니다. 두 번째 명령은 스키마와
Rust Adapter, Client, 오류 계약이 달라지면 실패합니다.
