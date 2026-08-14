# 공식 API 목록 기준

[English](README.md)

이 디렉터리는 **2026-08-10** 기준 공식 API 목록을 고정합니다. `src/coverage.rs`는
구현·검증 상태를 기록하는 선별된 목록이고, 이 디렉터리의 원본 목록은 공식 문서에
공개된 operation 전체를 보존합니다. 둘을 같은 것으로 취급하지 않습니다.

TSV는 모두 ASCII 영어로 유지합니다. 기준일은 파일명과 TSV 메타데이터에 반복하지 않고 이
README에만 기록합니다.

## 파일 역할

| 경로 | 역할 |
| --- | --- |
| `binance/{manifest,coverage,products}.tsv` | Binance 원본 operation, 연결표, 제품 식별자 정규화 |
| `bithumb/{manifest,coverage,classification}.tsv` | Bithumb 원본 operation, 연결표, 공개 노출 분류 |
| `upbit/{manifest,korea,coverage,classification}.tsv` | Upbit Global·Korea 전용 원본 operation, 연결표, 분류 |
| `hyperliquid/{manifest,coverage,unresolved}.tsv` | Hyperliquid 원본 operation, 연결표, 공식 근거가 불완전한 범위 |
| `audit/reviews.tsv` | 정확한 공식 operation key별 사람이 검토한 의미 감사 입력 |
| `audit/{ledger,queue,worklist,execution-checklist,platform-service-worklist}.tsv` | 생성된 감사 원장과 파생 큐 |

Binance와 Hyperliquid는 큰 원본 목록을 복제하지 않고 각 행의 `exposure` 열에 분류를
기록합니다. `coverage_inventory` 테스트가 모든 원본 행의 분류·lifecycle·현재 연결표를
함께 확인합니다.

## 독립된 범위 축

이 목록은 아래 세 축을 섞지 않습니다.

| 축 | 의미 |
| --- | --- |
| lifecycle | 공식 문서가 현재 활성으로 표시하는지, deprecated로 표시하는지 |
| exposure | 공통 `Client`/`Adapter`, 거래소 전용 typed service, 또는 별도 플랫폼·프로토콜 service 중 어느 경계로 공개할지 |
| implementation | 감사된 현재 공개 계약 상태입니다. 검증 완료, 확인된 계약 결손, 미구현, 설계 결정 필요, 공식 원본 차단으로 구분합니다. |

## 공개 노출 분류

- `common_existing`: 현재 `Client`/`Adapter` 공통 계약이 그대로 표현하는 operation입니다.
- `common_and_provider`: 공통 결과와 거래소 전용 결과를 함께 제공하는 operation입니다.
- `provider_typed`: 일반적인 거래소 전용 typed adapter API로 구현할 대상입니다. 아직
  구현되지 않았다는 이유만으로 플랫폼 제한으로 분류하지 않습니다.
- `platform_limited`: 별도 플랫폼·프로토콜 service가 필요한 operation입니다. 계약 호출, JSON-RPC,
  FIX/SBE, 공식 partner/VIP/KYC 자격, 지역, testnet/deployer 제약을 이 행의 경계로 보존합니다.
  활성 operation을 manifest 또는 구현 감사 범위에서 제외한다는 뜻은 아닙니다. 일반 거래소
  `Adapter`가 아니라 필요한 플랫폼·프로토콜 service 경계에서 구현합니다.
- `deprecated_excluded`: 공식 문서가 명시적으로 deprecated라고 표시한 operation입니다.

현재 연결표에 있는 operation은 `src/coverage.rs`의 실제 mapping을 따릅니다. 연결표에
없는 active operation도 manifest에 남으며, 연결되지 않았다는 사실만으로 Rust 미구현으로
판정하지 않습니다. Binance에서는 공식
partner·institutional·VIP·KYC·equity entitlement 경계와 FIX/SBE가, Hyperliquid에서는
HyperEVM JSON-RPC/contract, CoreWriter, HIP-3·HIP-4 deployer 및 명시적 validator/testnet
경계가 `platform_limited`입니다. lifecycle이 deprecated이면 언제나
`deprecated_excluded`입니다.

현재 고정 snapshot에서 일반 SDK 노출 분류는 Upbit 57개, Bithumb 47개, Binance 713개,
Hyperliquid 120개로 합계 937개입니다. 이 숫자는 잔여 구현 수가 아니라 이미 coverage에
연결되었거나 구현된 행까지 포함한 exposure 분류 합계입니다. 별도 플랫폼·프로토콜 경계의
활성 행은 Binance 340개와 Hyperliquid 97개입니다. 이들은 manifest·감사 대상으로 관리하되,
일반 거래소 `Adapter`의 동일 릴리스 구현 대상으로 합산하지 않습니다. 각 제품군은 별도
platform/protocol service 지원, service 또는 contract 결정이 필요한 `needs_design`, 또는 이번
릴리스 보류를 명시적으로 결정한 뒤 해당 service 경계에서만 구현합니다.

현재 `OPERATIONS`에는 `OperationMapping::PlatformLimited` 연결 행이 없습니다. 즉 437개는
별도 service가 필요하다고 분류된 상태일 뿐, 그 service 계약·구현이 존재하거나 모든 행을 즉시
구현해야 한다는 뜻이 아닙니다.

사람이 읽는 원장은 현재 coverage와 감사 결론, 다음 행동을 분리합니다.
`coverage_implementation_state`는 현재 선별 coverage 값일 뿐 감사 판정이 아닙니다.
검토자가 실제로 읽은 공식 operation key는 `audit/reviews.tsv`에 고정합니다. 명시적인 review
record가 없는 행도 고정 coverage 상태에서 판정하며, 상태 불명으로 남기지 않습니다.

| `audit_result` | `next_action` | 의미 |
| --- | --- | --- |
| `verified` | `none` | Rust, 공개 바인딩, 검증 근거를 확인했습니다. |
| `gap_found` | `needs_approval` | 동작하지만 공식 계약 일부를 보존하지 못합니다. |
| `not_implemented` | `needs_approval` | 감사 결과 연결된 Rust·schema·공개 바인딩 계약이 없습니다. |
| `needs_design` | `service_or_contract_decision` | 일반 Adapter에 넣으면 프로토콜 또는 플랫폼 경계를 잘못 표현합니다. |
| `blocked` | `official_contract_required` | 공식 원본이 manifest 행을 구현할 완전한 계약을 공개하지 않았습니다. |

`reason` 열은 구체적 근거를 기록합니다. 따라서 최종 원장에서
`MechanicallyConnected`를 제거했습니다. bridge, Rust, schema, 생성 계약, 바인딩, 검증 열이
기계적 근거를 그대로 보존하되 의미상 완료를 주장하지 않습니다. `audit-queue`는 일반 SDK
937개를 필터한 원장일 뿐 구현 일정이 아닙니다. worklist에는 연결된
`gap_found / needs_approval` 행만 담으며, 사용자가 고정 구현 배치를 승인하기 전까지는 구현을
시작하지 않습니다.

현재 의미 감사 결과 집계는 아래와 같습니다. 잔여 구현 수가 아닙니다.

| 범위 | Verified | Gap found | Not implemented | Needs design | 합계 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 일반 SDK | 143 | 51 | 701 | 42 | 937 |
| 별도 플랫폼·프로토콜 경계 | 0 | 0 | 0 | 437 | 437 |

일반 SDK 결과는 거래소별로도 고정합니다. 이 표는 현재 공개 계약 상태이며, 구현 일정 수가 아닙니다.

| 거래소 | Verified | Gap found | Not implemented | Needs design | 합계 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Upbit | 42 | 15 | 0 | 0 | 57 |
| Bithumb | 32 | 15 | 0 | 0 | 47 |
| Binance | 41 | 14 | 649 | 9 | 713 |
| Hyperliquid | 28 | 7 | 52 | 33 | 120 |

현재 coverage의 local 행은 Upbit 57개, Bithumb 47개, Binance 57개, Hyperliquid 35개로
합계 196개입니다. 이 중 Binance의 `mark_price`와 `mark_prices` 두 local 행이 하나의 공식
`premiumIndex` operation에 연결되므로, 고유 공식 operation 연결 수는 195개입니다. 이 local
행 안에서 `coverage.rs`의 Rust 구현 상태 분류는 Implemented 144개, Partial 51개, Planned 1개입니다.
701개 `not_implemented` 행은 감사가 끝난 결과입니다. 연결된 Rust·schema·공개 바인딩 계약이
없습니다. 다만 이것은 승인된 구현 배치가 아닙니다. 51개 `gap_found` 행은 연결된 공개 surface가
있지만 coverage가 명시적으로 Partial로 표시한 행이며, 이 worklist도 구현 전에 승인이 필요합니다.
연결된 51개 결손은 `audit/execution-checklist.tsv`에서 40개의 local operation 단위로 묶습니다.
목록에 있다는 사실만으로 구현이 승인되는 것은 아닙니다.

공식 request schema 또는 완전한 operation 목록이 없는 범위는 숫자를 추측해 분모에서 제거하지
않습니다. `hyperliquid/unresolved.tsv`에 `blocked / official_contract_required`와 source locator를
남기고, 공식 schema가 생기면 manifest 행과 audit result로 승격합니다.

## 중앙 계약 결정

이번 목록 고정 단계에서는 공통 `Adapter`/`Client` 계약을 추가하지 않습니다.
`src/adapter.rs`와 `src/request.rs`에 이미 있는 시장·호가·체결·캔들·주문·잔고·입출금·포지션·펀딩
의 정확한 의미만 공통으로 유지합니다. 새 공통 API는 최소 두 거래소에서 같은 요청·응답·오류
의미가 fixture로 확인될 때만 중앙에서 한 번 추가합니다. 그 전에는 provider typed API로
구현합니다.

Hyperliquid Explorer는 공식 rate-limit 문서가 `blockList`와 Explorer API family의 존재는
명시하지만 완전한 request schema/operation 목록은 제공하지 않습니다. 따라서 수를 임의로
세지 않고, 원본 schema가 제공되기 전까지 `hyperliquid/unresolved.tsv`의
`blocked / official_contract_required` 상태로 유지합니다.

## 검증

```sh
cargo test -p maxt-bindings-common --locked --features codegen --test coverage_inventory
```

원장과 파생 목록은 저장소 표준 Cargo 도구로 확인하거나 명시적으로 갱신합니다.

```sh
cargo run -p maxt-bindings-common --bin generate_audit_ledger --features codegen --locked -- --check
cargo run -p maxt-bindings-common --bin generate_audit_ledger --features codegen --locked -- --write
```

`--check`는 메모리에서 렌더링한 결과와 현재 파일을 비교할 뿐 파일을 쓰지 않습니다. `--write`만
활성 1,374행, 일반 SDK 필터 원장 937행, 승인 후보·실행 단위 목록, 별도 플랫폼 service 437행을 갱신합니다.
`coverage_inventory`는 행 수·열 수·허용된 result/action 조합, review key 무결성, 파생 관계를
검증합니다.

이 테스트는 원본 목록의 행 수·필드 수·공식 연결표·지역별 Upbit 목록·명시적 deprecated
operation과 현재 coverage 행의 연결을 검사합니다. `not_implemented` 감사 결과와 승인된 구현
배치를 분리합니다. 고정 범위의 Rust 구현을 모두 끝낼 때까지는
각 기능의 최소 단위 테스트만 수행합니다. 이후 한 번의 통합 단계에서 schema·코드 생성·세 언어
바인딩·문서·전체 회귀 검증을 함께 닫습니다.
