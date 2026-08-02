# maxt 바인딩 공통 계약

[English](README.md)

`maxt-bindings-common`은 Python, Dart, TypeScript 바인딩이 공유하는 요청,
응답, 오류, 스트림 계약을 정의합니다. 외부 언어에서 작성한 Adapter를 Rust
`Adapter`로 연결하는 `ForeignAdapter`도 제공합니다.

애플리케이션에서 직접 설치하는 패키지가 아닙니다. Rust, Python, Dart/Flutter,
TypeScript 패키지 중 사용하는 언어의 패키지를 설치하세요.

## 바인딩 정합성 검사

다음 테스트는 각 언어의 공개 Adapter, Client 메서드, 모델, enum, 생성 옵션이
Rust API와 일치하는지 검사합니다.

```sh
cargo test -p maxt-bindings-common --test language_binding_inventory --locked
```

요청 dispatch, 응답 또는 스트림 종료 계약을 변경했다면 공통 계약 테스트도
실행하세요.

```sh
cargo test -p maxt-bindings-common --locked
```

공개 계약은 [공통 API 명세](../../docs/common-api.ko.md)와
[기여 안내](../../CONTRIBUTING.ko.md)를 참고하세요.
