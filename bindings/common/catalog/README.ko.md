# 공식 API 목록 기준

[English](README.md)

이 디렉터리는 2026-08-10 기준 공식 API 목록을 고정합니다. `src/coverage.rs`는
구현·검증 상태를 기록하는 선별된 목록이고, 이 디렉터리의 원본 목록은 공식 문서에
공개된 operation 전체를 보존합니다. 둘을 같은 것으로 취급하지 않습니다.

## 파일 역할

| 파일 | 역할 |
| --- | --- |
| `*-2026-08-10.tsv` | 거래소 공식 operation 원본 목록. Binance·Hyperliquid 행에는 공개 노출 분류(`exposure`)도 함께 기록 |
| `*-coverage-2026-08-10.tsv` | 현재 `OPERATIONS` 행과 공식 원본 operation의 연결표 |
| `binance-products-2026-08-10.tsv` | Binance 공식 제품명과 로컬 제품 식별자의 연결표 |
| `*-classification-2026-08-10.tsv` | Upbit·Bithumb의 operation별 공개 노출 분류 |
| `hyperliquid-unresolved-2026-08-10.tsv` | 공식 문서가 완전한 목록을 제공하지 않아 수를 확정할 수 없는 범위 |
| `audit-ledger-2026-08-10.tsv` | 활성 공식 행 1,374개 전체의 감사 초안 원장. 연결된 Implemented 행도 제외하지 않음 |
| `audit-queue-2026-08-10.tsv` | 원장에서 일반 SDK 937개 전체를 추출한 감사 큐. Unreviewed는 구현 목록이 아님 |
| `implementation-worklist-2026-08-10.tsv` | 원장에서 `Partial`·`Planned`·`Blocked` 공식 행 52개를 파생한 후보 목록. 독립 작업 52개를 뜻하지 않음 |
| `execution-checklist-2026-08-10.tsv` | 후보 52행을 중복 없는 41개 local operation 실행 단위로 묶은 체크리스트 |
| `platform-service-worklist-2026-08-10.tsv` | 원장에서 `platform_limited`로 파생한 별도 플랫폼·프로토콜 service 목록 |

Binance와 Hyperliquid는 큰 원본 목록을 복제하지 않고 각 행의 `exposure` 열에 분류를
기록합니다. `coverage_inventory` 테스트가 모든 원본 행의 분류·lifecycle·현재 연결표를
함께 확인합니다.

## 독립된 범위 축

이 목록은 아래 세 축을 섞지 않습니다.

| 축 | 의미 |
| --- | --- |
| lifecycle | 공식 문서가 현재 활성으로 표시하는지, deprecated로 표시하는지 |
| exposure | 공통 `Client`/`Adapter`, 거래소 전용 typed service, 또는 별도 플랫폼·프로토콜 service 중 어느 경계로 공개할지 |
| implementation | Rust 공식 계약 보존 상태와 검증 근거. 현재 bridge가 없는 행은 이 축이 아직 감사되지 않은 것이며, 미구현으로 자동 단정하지 않습니다. |

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
platform/protocol service 지원, 권한·프로토콜 근거 부족 `Blocked`, 또는 이번 릴리스 보류를
명시적으로 결정한 뒤 해당 service 경계에서만 구현합니다.

현재 `OPERATIONS`에는 `OperationMapping::PlatformLimited` 연결 행이 없습니다. 즉 437개는
별도 service가 필요하다고 분류된 상태일 뿐, 그 service 계약·구현이 존재하거나 모든 행을 즉시
구현해야 한다는 뜻이 아닙니다.

감사 초안 원장의 `audit_status`는 `MechanicallyConnected`, `Partial`, `Planned`,
`Unreviewed`, `Blocked` 중 하나입니다. `MechanicallyConnected`는 coverage·schema·생성
계약의 기계적 연결만 뜻하며 의미상 `Complete`가 아닙니다. 공식 계약의 request/parse,
응답 필드 보존, facade, fixture 검증 locator를 operation별로 확인하기 전에는 Complete를
부여하지 않습니다. 공식 행이 bridge에 없으면 `Unreviewed`로 남기며 미구현으로 단정하지 않습니다.
`audit-queue`는 일반 SDK 937개 전체입니다. Partial·Planned·Blocked 공식 행 52개는 41개 local operation과
23개 mapping method로 겹치므로 52개 독립 구현 작업으로 취급하지 않습니다. 실제 실행 후보는
`execution-checklist`의 41개 단위입니다. 새 발견 항목은
다음 배치 backlog로만 기록합니다.

실행 단위의 소유권은 공용 계약 중복을 막기 위해 고정합니다: 공용 공통 계약 11개,
Upbit 4개, Bithumb 4개, Binance 15개, Hyperliquid 7개입니다. 공용 11개는 한 담당자만
소유하며 Upbit·Bithumb 담당자가 나누어 수정하지 않습니다.

감사 초안의 기계 상태 집계는 다음과 같습니다. `MechanicallyConnected`를 의미상
Complete로 해석하거나 `Unreviewed`를 잔여 구현 수로 해석하면 안 됩니다.

| 범위 | MechanicallyConnected | Partial | Planned | Unreviewed | 합계 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Upbit 일반 SDK | 42 | 15 | 0 | 0 | 57 |
| Bithumb 일반 SDK | 32 | 15 | 0 | 0 | 47 |
| Binance 일반 SDK | 41 | 14 | 1 | 657 | 713 |
| Hyperliquid 일반 SDK | 28 | 7 | 0 | 85 | 120 |
| 일반 SDK 합계 | 143 | 51 | 1 | 742 | 937 |

현재 coverage의 local 행은 Upbit 57개, Bithumb 47개, Binance 57개, Hyperliquid 35개로
합계 196개입니다. 이 중 Binance의 `mark_price`와 `mark_prices` 두 local 행이 하나의 공식
`premiumIndex` operation에 연결되므로, 고유 공식 operation 연결 수는 195개입니다. 이 local
행 안에서 `coverage.rs`의 Rust 구현 상태 분류는 Implemented 144개, Partial 51개, Planned 1개입니다.
따라서 `937 - 196`은 잔여
구현 수가 아니라 아직 bridge에 연결하지 않은 일반 SDK 분류 행 수일 뿐입니다. bridge 밖 행은
기존 Rust 코드 존재 여부와 공식 계약 보존 여부를 operation별로 감사하기 전에는 미구현으로
보고하지 않습니다.

공식 request schema 또는 완전한 operation 목록이 없는 범위는 숫자를 추측해 분모에서 제거하지
않습니다. `hyperliquid-unresolved-2026-08-10.tsv`에 `Blocked` 근거와 source locator를
남기고, 공식 schema가 생기면 manifest 행과 implementation 상태로 승격합니다.

## 중앙 계약 결정

이번 목록 고정 단계에서는 공통 `Adapter`/`Client` 계약을 추가하지 않습니다.
`src/adapter.rs`와 `src/request.rs`에 이미 있는 시장·호가·체결·캔들·주문·잔고·입출금·포지션·펀딩
의 정확한 의미만 공통으로 유지합니다. 새 공통 API는 최소 두 거래소에서 같은 요청·응답·오류
의미가 fixture로 확인될 때만 중앙에서 한 번 추가합니다. 그 전에는 provider typed API로
구현합니다.

Hyperliquid Explorer는 공식 rate-limit 문서가 `blockList`와 Explorer API family의 존재는
명시하지만 완전한 request schema/operation 목록은 제공하지 않습니다. 따라서 수를 임의로
세지 않고, 원본 schema가 제공되기 전까지 `hyperliquid-unresolved-2026-08-10.tsv`의
`Blocked` 상태로 유지합니다.

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
활성 1,374행, 감사 큐 937행, 후보 52행, 실행 단위 41행, 별도 플랫폼 service 437행을 갱신합니다.
`coverage_inventory`는 행 수·열 수·상태값·파생 관계와 41개/23개 중복 제거 수를 검증합니다.

이 테스트는 원본 목록의 행 수·필드 수·공식 연결표·지역별 Upbit 목록·명시적 deprecated
operation과 현재 coverage 행의 연결을 검사합니다. coverage 연결 수와 구현 상태 수는 별도
집계하며, 연결되지 않은 행을 잔여 구현으로 단정하지 않습니다. 고정 범위의 Rust 구현을 모두 끝낼 때까지는
각 기능의 최소 단위 테스트만 수행합니다. 이후 한 번의 통합 단계에서 schema·코드 생성·세 언어
바인딩·문서·전체 회귀 검증을 함께 닫습니다.
