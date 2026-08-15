# Guide: account reads and safe financial requests

[English](account-safety.md) | [한국어](account-safety.ko.md)

Use this guide for account data, order preparation, and transfer workflows.

## Start with a read-only credential

Use a key pair with only the permissions the example needs. The runnable
[account and safety examples](../examples.md#account-and-assets) read balances
and open orders only when their environment or compile-time credentials are
present. Without them, they print setup instructions and exit.

Do not put a secret in source code, a checked-in configuration file, or a
browser build. Hyperliquid address-scoped Info reads are different: they use a
public address and need no private key. Signed Hyperliquid actions need a
signer.

## Build before submitting

`OrderRequest`, `WithdrawRequest`, history requests, and provider request types
validate their local shape before a network request. Build and log-safe inspect
the request first. The supplied safety examples deliberately do not call
`placeOrder`, `withdraw`, cancellation, or provider financial-write methods.

When a provider has a genuine validation endpoint, prefer it to a live order:

- Upbit: `testOrder`
- Binance: `testOrder`

Their responses are dry-run results, not live orders. Do not query or cancel a
dry-run ID.

## Treat writes as an explicit application decision

Submitting an order, withdrawal, transfer, margin change, or cancellation is a
financial write. Put the confirmation policy, idempotency policy, audit trail,
and retry decision in your application rather than a generic example. The
generated [API-to-scenario map](../examples.md#api-to-scenario-map) marks the
task where each public method belongs; provider pages explain exchange-specific
limits and race conditions.
