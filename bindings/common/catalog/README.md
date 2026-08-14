# Official API inventory

[한국어](README.ko.md)

This directory pins the official API inventory at **2026-08-10**. `src/coverage.rs`
is the curated record of implementation and validation state; the source lists
here preserve every documented operation. They are not interchangeable.

All TSV files use ASCII English. The snapshot date is recorded here, rather
than repeated in filenames or TSV metadata.

## Files

| Path | Purpose |
| --- | --- |
| `binance/{manifest,coverage,products}.tsv` | Binance source operations, bridge, and product normalization |
| `bithumb/{manifest,coverage,classification}.tsv` | Bithumb source operations, bridge, and exposure classification |
| `upbit/{manifest,korea,coverage,classification}.tsv` | Upbit Global and Korea-only source operations, bridge, and classification |
| `hyperliquid/{manifest,coverage,unresolved}.tsv` | Hyperliquid source operations, bridge, and unresolved official scope |
| `audit/reviews.tsv` | Human semantic-audit inputs keyed to exact official operations |
| `audit/{ledger,queue,worklist,execution-checklist,platform-service-worklist}.tsv` | Generated audit ledger and derived queues |

Binance and Hyperliquid keep their `exposure` decision in the source rows so
their large lists are not duplicated. The `coverage_inventory` test verifies
every source row, lifecycle, and bridge mapping.

## Independent scope axes

This inventory does not conflate these three axes:

| Axis | Meaning |
| --- | --- |
| lifecycle | Whether the official documentation marks an operation active or deprecated |
| exposure | Whether maxt exposes it through the common `Client`/`Adapter`, an exchange-specific typed service, or a separate platform/protocol service |
| implementation | The audited current public-contract state: verified, a known contract gap, no implementation, a required design decision, or an official-source blocker. |

## Public-exposure classification

- `common_existing`: The current `Client`/`Adapter` common contract expresses
  the operation exactly.
- `common_and_provider`: The operation exposes both common and provider-specific
  results.
- `provider_typed`: A normal exchange-specific typed adapter API to implement.
  An ordinary unimplemented operation is not platform-limited.
- `platform_limited`: An operation requiring a separate platform or protocol
  service. It preserves contract calls, JSON-RPC, FIX/SBE, official
  partner/VIP/KYC eligibility, regional, and testnet/deployer constraints. It
  does not remove an active operation from manifest or implementation-audit
  scope. It is implemented at the required platform/protocol service boundary,
  not by the general exchange `Adapter`.
- `deprecated_excluded`: The official source explicitly marks the operation
  deprecated.

Operations in a current bridge use their actual `src/coverage.rs` mapping. An
active operation outside a bridge remains in the manifest; no Rust
implementation state follows from the bridge being absent. For Binance,
official partner, institutional, VIP, KYC, equity-entitlement boundaries and
FIX/SBE are `platform_limited`. For Hyperliquid, HyperEVM JSON-RPC/contracts,
CoreWriter, HIP-3/HIP-4 deployers, and explicit validator/testnet boundaries
are `platform_limited`. A deprecated lifecycle is always `deprecated_excluded`.

In this fixed snapshot, the general-SDK exposure classification is 57 Upbit,
47 Bithumb, 713 Binance, and 120 Hyperliquid operations: 937 total. That is an
exposure total, not remaining implementation work; it includes rows already
bridged or implemented. The separate active platform/protocol boundary has 340
Binance and 97 Hyperliquid operations. They remain implementation and
audit targets, but are not counted as the general exchange `Adapter` work for
the same release. Each product family requires an explicit decision: support it
through its own platform/protocol service, record it as `needs_design` until a
service or contract decision is made, or defer it from this release.

`OPERATIONS` currently has no `OperationMapping::PlatformLimited` bridge row.
The 437 operations are therefore classified as requiring separate services;
this does not claim that those service contracts exist, that they are
implemented, or that all must be implemented immediately.

The human-readable ledger separates current coverage from an audit conclusion
and its next action. `coverage_implementation_state` is the current curated
coverage value; it is not an audit verdict. The checked-in `audit/reviews.tsv`
records exact operation keys that a reviewer has read. A missing explicit
review is rendered from the fixed coverage state, never as an unknown state.

| `audit_result` | `next_action` | Meaning |
| --- | --- | --- |
| `verified` | `none` | Rust, public bindings, and verification evidence were reviewed. |
| `gap_found` | `needs_approval` | The operation works, but the official contract is not fully preserved. |
| `not_implemented` | `needs_approval` | The completed audit found no connected Rust, schema, or public-binding contract. |
| `needs_design` | `service_or_contract_decision` | A general Adapter would misstate the protocol or platform boundary. |
| `blocked` | `official_contract_required` | The official source does not publish a complete contract from which a manifest row can be implemented. |

`reason` explains the concrete basis. This deliberately removes
`MechanicallyConnected` from the final ledger: bridge, Rust, schema, generated
contract, binding, and validation columns retain the machine evidence without
claiming semantic completion. `audit-queue` is the filtered 937-row general-SDK
ledger, not an implementation schedule. The worklist contains only connected
`gap_found / needs_approval` rows; it remains unauthorized until the user
approves a fixed implementation batch.

The current semantic-audit result summary is below. It is not a remaining-work
count.

| Scope | Verified | Gap found | Not implemented | Needs design | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| General SDK | 143 | 51 | 701 | 42 | 937 |
| Separate platform/protocol boundary | 0 | 0 | 0 | 437 | 437 |

The general-SDK result is also fixed by exchange. It describes the current
public-contract state, not a scheduled implementation count.

| Exchange | Verified | Gap found | Not implemented | Needs design | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| Upbit | 42 | 15 | 0 | 0 | 57 |
| Bithumb | 32 | 15 | 0 | 0 | 47 |
| Binance | 41 | 14 | 649 | 9 | 713 |
| Hyperliquid | 28 | 7 | 52 | 33 | 120 |

Current coverage bridges contain 57 Upbit, 47 Bithumb, 57 Binance, and 35
Hyperliquid local rows: 196 total. Binance `mark_price` and `mark_prices` both
bridge to the one official `premiumIndex` operation, so the bridge has 195
unique official-operation targets. Within the local bridge rows, `coverage.rs`
implementation status is 144 Implemented, 51 Partial, and 1 Planned.
The 701 `not_implemented` rows are a completed audit finding: they have no
connected Rust, schema, or public-binding contract. They are not an approved
implementation batch. The 51 `gap_found` rows have a connected public surface
that coverage explicitly marks Partial; their worklist entries likewise require
approval before any implementation starts.
The 51 connected gaps are grouped into 40 local-operation units in
`audit/execution-checklist.tsv`; none is authorized merely by appearing there.

Where the official source lacks a complete request schema or operation list,
the inventory does not guess a count or discard the scope.
`hyperliquid/unresolved.tsv` records `blocked / official_contract_required`
with a source locator until an official schema permits manifest rows and an
audit result.

## Central contract decision

This inventory step adds no common `Adapter`/`Client` contract. The existing
market, book, trade, candle, order, balance, transfer, position, and funding
meanings in `src/adapter.rs` and `src/request.rs` remain the common surface. A
new common API is added centrally only after at least two exchanges verify the
same request, response, and error semantics by fixture. Until then, implement
it as a provider typed API.

The Hyperliquid rate-limit documentation names `blockList` and the Explorer
family but provides no complete request schema or operation list. Its count is
not guessed; it remains `blocked / official_contract_required` in
`hyperliquid/unresolved.tsv` until an official schema is available.

## Verification

```sh
cargo test -p maxt-bindings-common --locked --features codegen --test coverage_inventory
```

Check or explicitly update the ledger and derived lists with the
repository-native Cargo tool:

```sh
cargo run -p maxt-bindings-common --bin generate_audit_ledger --features codegen --locked -- --check
cargo run -p maxt-bindings-common --bin generate_audit_ledger --features codegen --locked -- --write
```

`--check` renders and compares entirely in memory without writing files.
Only `--write` updates the active 1,374-row ledger, 937-row filtered general-SDK
ledger, approval-candidate and execution-checklist files, and the 437-row
separate-service list. `coverage_inventory` verifies row/column counts,
allowed result/action pairs, review-key integrity, and derived grouping.

This test checks source row counts and fields, official bridges, regional Upbit
inventory, explicit deprecations, and the connection to current coverage rows.
It distinguishes the completed `not_implemented` finding from an approved
implementation batch.
Until the fixed Rust scope is complete, work runs only each feature's minimum
unit tests. One integration stage then closes schema, code generation, all
three language bindings, documentation, and full regression together.
