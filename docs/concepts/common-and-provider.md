# Concept: common APIs and provider APIs

[English](common-and-provider.md) | [한국어](common-and-provider.ko.md)

`maxt` has two intentionally different API layers.

## Common API: portable behavior

`Client` exposes operations such as `ticker`, `candles`, `balances`, and
`openOrders`. They use common models and a common stream/error contract, so an
application can switch among adapters that support the feature with a small
change.

Common does not mean every exchange endpoint is forced into one model. A field
that cannot be represented honestly stays absent instead of being guessed.

## Provider API: exchange-specific fidelity

Concrete adapters expose provider methods when an exchange has meaningful data
or behavior outside the common contract. Examples include Binance mark-price
context, Upbit Korea pockets, Bithumb TWAP, and Hyperliquid address-scoped
Info responses.

Use `client.adapter` in Rust/Python/Dart, or the concrete adapter instance in
TypeScript. This is composition: an application keeps a portable `Client` and
can opt into an exchange-specific operation where it matters.

## Choose deliberately

Choose a common operation when the application needs comparable behavior
across exchanges. Choose a provider operation when the extra fields, regional
rules, or response semantics affect the application decision. The generated
[API-to-scenario map](../examples.md#api-to-scenario-map) names the runnable
task for every public method; the generated [binding contract](../../bindings/common/generated/api.md)
records its exact language name.
