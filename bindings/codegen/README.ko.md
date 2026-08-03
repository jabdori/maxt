# maxt 바인딩 코드 생성기

[English](README.md)

`maxt-bindings-codegen`은 `maxt-bindings-common`의 바인딩 스키마에서 언어별
계약을 생성합니다. 애플리케이션에서 설치하는 패키지가 아니라 저장소 개발
도구입니다.

## 계약 생성

```sh
cargo run -p maxt-bindings-codegen --no-default-features --features python --locked -- python
```

대상은 `rust`, `python`, `dart`, `typescript`입니다. 지정한 포트의 결과만
갱신합니다. 대상을 생략하면 모든 결과를 갱신합니다. 생성 파일은 직접
수정하지 마세요.

## 생성 결과 검사

```sh
cargo run -p maxt-bindings-codegen --no-default-features --features python --locked -- python --check
cargo test -p maxt-bindings-common --features codegen --test schema_inventory --locked
```

첫 번째 명령은 선택한 포트의 생성 결과만 검사합니다. 두 번째 명령은 Rust
소스 스키마를 검사합니다.
