# ADR 0001: Keep provider-specific stream contracts separate

[English](0001-provider-specific-stream-boundaries.md) | [한국어](0001-provider-specific-stream-boundaries.ko.md)

- Status: Accepted
- Date: 2026-08-14

## Context

`Client::subscribe` and `Client::subscribe_account` provide the common live
stream contract. They return `MarketStream` and `AccountStream`, whose events
are deliberately normalized into `MarketEvent` and `AccountEvent`.

Some exchange streams carry additional information that has no stable common
meaning. Hyperliquid, for example, publishes trade hashes and participants,
per-level order counts, candle trade counts, provider order-status detail, and
Spot entry notional. The current common streams intentionally do not expose
those fields. See the [Hyperliquid subscription reference](../providers/hyperliquid.md#streams)
and the [official WebSocket subscription schemas](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions).

The same common event types are public in Rust, Python, Dart, and TypeScript.
Adding a provider-only optional field or a global extension object to them
would couple every provider and binding to one exchange's data model.

## Decision

1. Keep `Client`, `MarketStream`, `AccountStream`, `MarketEvent`, and
   `AccountEvent` as stable, normalized common contracts.
2. Expose full-fidelity streaming data through provider-specific adapter APIs
   and provider-specific tagged event types. A provider event may contain a
   normalized common projection plus its documented native fields.
3. Type documented provider fields. `raw_json` is permitted for
   forward-compatible retention when needed, but does not replace typed
   fields.
4. Promote a field into a common model only after an explicit compatibility
   decision establishes an exchange-independent meaning, unit, absence rule,
   and validation contract. Evidence from more than one provider is normally
   required.
5. Do not introduce a generic public `ProviderStream<T>` abstraction before a
   second concrete provider-specific stream needs the same mechanism. Reuse
   the existing connection, close, reconnect, and backpressure behaviour.

For example, a future Hyperliquid-specific trade event should conceptually
contain a normalized `Trade` and the Hyperliquid-only hash, participants, and
native trade identifier. It must not add those fields to every exchange's
`Trade`.

## Consequences

- Common-stream consumers retain one small, portable API across exchanges.
- Advanced consumers can opt into typed provider data without downcasting a
  common event or decoding untyped JSON.
- Adding another exchange's native stream affects that provider's types,
  schema, bindings, fixtures, and documentation rather than every common
  event consumer.
- A provider-specific stream remains a public API and must be implemented
  through the schema and all supported language bindings before it is marked
  complete.
- The six Hyperliquid common-stream gaps are covered by this decision. Their
  implementation is a single provider-stream batch, not six unrelated common
  event changes.

## Out of scope

Upbit `LIST_SUBSCRIPTIONS` is a connection-control operation, not a
provider-data extension. Its adapter-local handling is decided separately in
[ADR 0002](0002-upbit-connection-scoped-operations.md).

Platform-limited operations remain separate services or explicit blocked
decisions. This ADR does not change their adapter boundary.

## Alternatives rejected

### Add provider fields as `Option` fields on common models

This makes every exchange and binding carry provider-only concepts, obscures
which provider owns a field, and turns normal model evolution into a global
compatibility change.

### Add one universal provider-extension object to common events

This simply moves the same coupling into a large cross-provider union. It
would still require every binding to know every provider's native data.

### Discard the provider fields

This prevents full-fidelity clients and conflicts with the documented
provider-specific API policy.
