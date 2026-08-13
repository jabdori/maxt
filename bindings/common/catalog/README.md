# Official API inventory

[한국어](README.ko.md)

This directory pins the official API inventory at 2026-08-10. `src/coverage.rs`
is the curated record of implementation and validation state; the source lists
here preserve every documented operation. They are not interchangeable.

## Files

| File | Purpose |
| --- | --- |
| `*-2026-08-10.tsv` | Official exchange operation source list; Binance and Hyperliquid also carry their per-operation `exposure` decision |
| `*-coverage-2026-08-10.tsv` | Mapping from current `OPERATIONS` rows to source operations |
| `binance-products-2026-08-10.tsv` | Binance official product to local-product mapping |
| `*-classification-2026-08-10.tsv` | Per-operation public-exposure classification for Upbit and Bithumb |
| `hyperliquid-unresolved-2026-08-10.tsv` | Scope whose exact count cannot be derived from official documentation |
| `audit-ledger-2026-08-10.tsv` | Draft 1,374-row active-operation audit ledger, including Implemented rows |
| `audit-queue-2026-08-10.tsv` | All 937 general-SDK rows retained for semantic audit; Unreviewed is not an implementation list |
| `implementation-worklist-2026-08-10.tsv` | The 52 official `Partial`/`Planned`/`Blocked` candidate rows; not 52 independent tasks |
| `execution-checklist-2026-08-10.tsv` | The 52 rows grouped into 41 unique local-operation execution units |
| `platform-service-worklist-2026-08-10.tsv` | `platform_limited` rows derived for separate platform/protocol services |

Binance and Hyperliquid keep their `exposure` decision in the source rows so
their large lists are not duplicated. The `coverage_inventory` test verifies
every source row, lifecycle, and bridge mapping.

## Independent scope axes

This inventory does not conflate these three axes:

| Axis | Meaning |
| --- | --- |
| lifecycle | Whether the official documentation marks an operation active or deprecated |
| exposure | Whether maxt exposes it through the common `Client`/`Adapter`, an exchange-specific typed service, or a separate platform/protocol service |
| implementation | Rust official-contract preservation and verification evidence. A source row outside a bridge has not yet been audited on this axis; it is not automatically unimplemented. |

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
through its own platform/protocol service, retain it as `Blocked` because its
protocol or entitlement evidence is insufficient, or defer it from this
release.

`OPERATIONS` currently has no `OperationMapping::PlatformLimited` bridge row.
The 437 operations are therefore classified as requiring separate services;
this does not claim that those service contracts exist, that they are
implemented, or that all must be implemented immediately.

The draft ledger `audit_status` is one of `MechanicallyConnected`, `Partial`,
`Planned`, `Unreviewed`, or `Blocked`. `MechanicallyConnected` only means that
coverage, schema, and generated-contract links are present; it is not semantic
`Complete`. Request/parse behavior, response preservation, public facades, and
operation-level fixture locators must be reviewed before a row can become
`Complete`. An active operation outside a bridge remains `Unreviewed`, never
automatically unimplemented. `audit-queue` retains all 937 general-SDK rows.
The 52 `Partial`/`Planned`/`Blocked` official rows overlap across 41 local operations and
23 mapping methods, so they are not 52 independent tasks. The 41 entries in
`execution-checklist` are the current execution units.
Newly discovered items go to the next-batch backlog only.

Execution ownership is fixed to prevent duplicate shared-contract changes:
11 common-contract units, 4 Upbit units, 4 Bithumb units, 15 Binance units,
and 7 Hyperliquid units. One owner controls the 11 common units; Upbit and
Bithumb owners do not split or concurrently modify them.

The draft's mechanical status counts are below. `MechanicallyConnected` is not
semantic `Complete`, and `Unreviewed` is not a remaining-work count.

| Scope | MechanicallyConnected | Partial | Planned | Unreviewed | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| Upbit general SDK | 42 | 15 | 0 | 0 | 57 |
| Bithumb general SDK | 32 | 15 | 0 | 0 | 47 |
| Binance general SDK | 41 | 14 | 1 | 657 | 713 |
| Hyperliquid general SDK | 28 | 7 | 0 | 85 | 120 |
| General SDK total | 143 | 51 | 1 | 742 | 937 |

Current coverage bridges contain 57 Upbit, 47 Bithumb, 57 Binance, and 35
Hyperliquid local rows: 196 total. Binance `mark_price` and `mark_prices` both
bridge to the one official `premiumIndex` operation, so the bridge has 195
unique official-operation targets. Within the local bridge rows, `coverage.rs`
implementation status is 144 Implemented, 51 Partial, and 1 Planned.
Therefore `937 - 196` is only
the number of general-SDK classification rows without a bridge; it is not
remaining implementation work. A row outside a bridge is not reported as
unimplemented before an operation-by-operation audit of existing Rust behavior
and official-contract preservation.

Where the official source lacks a complete request schema or operation list,
the inventory does not guess a count or discard the scope. It retains `Blocked`
evidence and a source locator in `hyperliquid-unresolved-2026-08-10.tsv` until
the official schema permits manifest rows and an implementation status.

## Central contract decision

This inventory step adds no common `Adapter`/`Client` contract. The existing
market, book, trade, candle, order, balance, transfer, position, and funding
meanings in `src/adapter.rs` and `src/request.rs` remain the common surface. A
new common API is added centrally only after at least two exchanges verify the
same request, response, and error semantics by fixture. Until then, implement
it as a provider typed API.

The Hyperliquid rate-limit documentation names `blockList` and the Explorer
family but provides no complete request schema or operation list. Its count is
not guessed; its `Blocked` status remains in
`hyperliquid-unresolved-2026-08-10.tsv` until an official schema is available.

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
Only `--write` updates the active 1,374-row ledger, 937-row audit queue,
52-row candidate list, 41-row execution checklist, and 437-row separate-service
list. `coverage_inventory` verifies row/column counts, allowed statuses,
derivation, and the 41-operation/23-method grouping.

This test checks source row counts and fields, official bridges, regional Upbit
inventory, explicit deprecations, and the connection to current coverage rows.
It counts bridge connections and implementation status separately, and never
infers remaining implementation work from an absent bridge.
Until the fixed Rust scope is complete, work runs only each feature's minimum
unit tests. One integration stage then closes schema, code generation, all
three language bindings, documentation, and full regression together.
