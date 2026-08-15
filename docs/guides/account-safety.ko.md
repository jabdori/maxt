# 가이드: 계좌 조회와 안전한 금융 요청

[English](account-safety.md) | [한국어](account-safety.ko.md)

계좌 데이터, 주문 준비, 전송 흐름을 사용할 때 이 가이드를 참고하세요.

## 읽기 전용 인증 정보부터 시작하세요

예제가 필요한 최소 권한만 가진 키 쌍을 사용하세요. 실행 가능한 [계좌와 안전성
예제](../examples.md#account-and-assets)는 환경 변수 또는 컴파일 시 인증 정보가 있을
때만 잔고와 미체결 주문을 읽습니다. 인증 정보가 없으면 설정 방법을 출력하고
종료합니다.

비밀 값을 소스 코드, 커밋되는 설정 파일, 브라우저 빌드에 넣지 마세요. Hyperliquid의
주소 단위 Info 조회는 다릅니다. 공개 주소만 사용하고 private key가 필요하지
않습니다. 서명된 Hyperliquid 작업에는 signer가 필요합니다.

## 제출하기 전에 요청을 만드세요

`OrderRequest`, `WithdrawRequest`, 이력 요청, provider 요청 타입은 네트워크 요청 전에
로컬 형태를 검증합니다. 먼저 요청을 만들고 로그에 안전한 범위에서 검사하세요.
제공되는 안전성 예제는 의도적으로 `placeOrder`, `withdraw`, 취소, provider 금융 쓰기
메서드를 호출하지 않습니다.

provider가 실제 검증 endpoint를 제공한다면 실주문 대신 우선 사용하세요.

- Upbit: `testOrder`
- Binance: `testOrder`

응답은 dry-run 결과이며 실제 주문이 아닙니다. dry-run ID를 조회하거나 취소하지
마세요.

## 금융 쓰기는 애플리케이션에서 명시적으로 결정하세요

주문, 출금, 전송, 증거금 변경, 취소 제출은 금융 쓰기(financial write)입니다. 확인
정책, 멱등성 정책, 감사 기록, 재시도 판단은 일반 예제가 아니라 애플리케이션에
두세요. 생성된 [API-시나리오 맵](../examples.md#api-to-scenario-map)은 각 공개 메서드가
속한 작업을 표시하며, provider 문서는 거래소별 한도와 경합 조건을 설명합니다.
