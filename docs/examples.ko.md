# 예제 안내

[English](examples.md) | [한국어](examples.ko.md)

현재 공개 API의 모든 메서드는 생성된 [영문 API-시나리오 맵](examples.md#api-to-scenario-map)에서
하나 이상의 사용자 작업과 연결됩니다. 이 표는 스키마(schema)에 새 공개 API가 추가되었지만
시나리오가 지정되지 않으면 코드 생성(code generation)이 실패하도록 검증합니다.

예제는 endpoint마다 하나씩 복제하지 않습니다. 대신 공개 시장 데이터, 캔들·이력,
스트림, 계좌 읽기, 안전한 요청, 파생상품, 각 거래소의 provider 기능처럼 사용자가
수행하는 작업 단위로 작성합니다. 예제 목록과 언어별 실행 파일은 다음에서 확인하세요.

- [Rust 예제](../examples/README.md)
- [Python 예제](../bindings/python/python/maxt/examples/README.md)
- [Dart 예제](../bindings/dart/example/README.md)
- [TypeScript 예제](../bindings/typescript/examples/README.md)

공개 조회는 실제 거래소에 연결합니다. 인증된 읽기 예제는 인증 정보가 없으면 설정
방법을 출력하고 종료합니다. 주문, 출금, 전송, 증거금 변경처럼 금융 쓰기를 하는 예제는
기본 실행에서 요청 객체만 만들고 네트워크에 제출하지 않습니다.
