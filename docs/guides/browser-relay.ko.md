# 가이드: 브라우저 WebAssembly와 릴레이 경계

[English](browser-relay.md) | [한국어](browser-relay.ko.md)

Dart Web 또는 TypeScript 브라우저 바인딩을 배포할 때 이 가이드를 사용하세요.

## 공개 호출은 직접 연결할 수 있습니다

공개 시장 데이터 호출에는 릴레이 없이 브라우저 바인딩을 초기화하세요. [Dart](../examples.md#browser-relay)와
[TypeScript](../examples.md#browser-relay) 브라우저 예제는 기본적으로 이 방식입니다.
브라우저의 CORS와 네트워크 정책은 계속 적용됩니다.

## 서명된 호출에는 릴레이와 명시적 허용이 필요합니다

브라우저에서 인증 정보나 private key로 어댑터를 설정할 때는 신뢰할 수 있는
`relayUrl`과 명시적인 insecure-browser credential 허용을 모두 지정해 초기화하세요.
릴레이는 서명된 HTTP 요청과 일부 WebSocket handshake를 전달합니다. 사용자 인증
수단도 아니고 비밀값 저장소도 아닙니다.

릴레이는 애플리케이션 인증과 속도 제한이 적용된 TLS ingress 뒤에 있어야 합니다.
upstream 허용 목록은 좁게 유지하세요. 정확한 배포 변수, 프로토콜, 보안 제한은
[릴레이 레퍼런스](../../relay/README.ko.md)에 있습니다.

## 신뢰 경계를 보이게 유지하세요

SDK는 이 설정 없이 브라우저 인증 정보를 거부합니다. 공개 웹 앱을 키 보관자로
조용히 바꾸는 생성자가 생기지 않게 하기 위해서입니다. 서명된 브라우저 호출이
필요하지 않다면 릴레이를 배포하지 마세요.
