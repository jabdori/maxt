# ADR 0002: Keep Upbit connection control adapter-local

[English](0002-upbit-connection-scoped-operations.md) | [한국어](0002-upbit-connection-scoped-operations.ko.md)

- Status: Accepted
- Date: 2026-08-14

## Context

Upbit's `LIST_SUBSCRIPTIONS` is an operation on one already-open WebSocket
connection. Opening a second socket and recreating a subscription can verify a
request shape, but cannot answer what the caller's live connection currently
receives.

`MarketStream` deliberately remains an inbound, portable stream. Making it a
public bidirectional control abstraction for this single provider operation
would broaden every adapter and binding without another concrete user.

## Decision

1. Keep `UpbitAdapter::list_subscriptions(subscription)` as the public
   provider operation.
2. Register each active Upbit market connection under its immutable
   `Subscription` selector. The adapter sends `LIST_SUBSCRIPTIONS` through the
   matching session's internal write handle and consumes its response before
   market-event decoding.
   The returned `MarketStream` remains the connection's dispatcher and must
   keep running while the operation awaits its reply.
3. Require exactly one matching active connection. No match or multiple
   matches returns a local `InvalidRequest` error; the SDK never guesses which
   socket the caller meant.
4. Keep the internal socket write handle crate-private. It is not a public
   common-stream API, and response parsing remains Upbit-specific.
5. An operation waiting for a response fails if that session reconnects,
   because the request belonged to the prior socket.

## Consequences

- Rust, Python, Dart, and TypeScript keep the existing
  `list_subscriptions(subscription)` signature.
- The operation now has the same connection scope as Upbit's documented API.
- Multiple identical live subscriptions are explicit ambiguity, not silently
  selected state.
- If another provider requires richer connection control, reconsider a
  provider-specific stream API first. Do not add a universal mutable stream
  abstraction solely for this operation.

## Alternatives rejected

### Open a temporary socket

It reports the temporary socket's subscriptions, not the caller's connection.

### Add a public control method to `MarketStream`

That would imply every market stream supports provider-specific operation
frames and response correlation, which is not true.

### Pick the most recent matching connection

The result would depend on timing when multiple identical connections exist
and could inspect the wrong session.
