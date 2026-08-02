# maxt TypeScript

[English](README.md)

TypeScript 패키지는 개발 중이며 아직 배포하지 않았습니다. 패키지 이름은
`@jabdori/maxt`로 확정했으며 Node와 브라우저 계약 테스트를 모두 통과한
`0.1.0` 릴리스부터 설치할 수 있습니다.

## 개발

```sh
npm ci
npm run build
npm run test:unit
npm run build:node
```

공통 wire DTO와 API 목록은 `maxt-bindings-common` 계약에서 생성합니다.

```sh
cargo run -p maxt-bindings-codegen --locked
cargo run -p maxt-bindings-codegen --locked -- --check
```

공개 Node와 브라우저 API가 실제로 동작하면 설치 명령과 Binance Spot
`BTC/USDT` 공통 API 및 provider 전용 API 예제를 추가합니다. 현재 구현되지
않은 facade를 사용할 수 있는 것처럼 문서화하지 않습니다.
