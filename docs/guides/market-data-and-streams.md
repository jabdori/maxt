# Guide: market data and streams

[English](market-data-and-streams.md) | [한국어](market-data-and-streams.ko.md)

Use this guide when an application needs public prices, candles, order books,
or a live market feed.

## Read a snapshot first

Use `Client` for the portable snapshot calls:

- market catalogue: `markets`
- latest price: `ticker`
- top-of-book or depth snapshot: `orderBook`
- recent individual trades: `trades`
- oldest-first candle range: `candles`

The [public market-data example](../examples.md#market-data) shows a market
list, ticker, book, and trades. The [candle example](../examples.md#candles-and-history)
uses `CandleRequest`; its lower time bound is inclusive and its upper time
bound is exclusive.

## Open a stream after the snapshot

Subscribe with a `Subscription` that names one or more markets and feeds. A
snapshot before the stream gives the application an initial state. Treat a
reconnect event as a possible gap: fetch a fresh snapshot before treating the
stream as synchronized again.

The runnable [stream examples](../examples.md#streams) read a few events and
then close. Production applications normally keep the stream alive, surface
per-item stream errors, and decide how to rebuild their local state after a
reconnect.

## Select the right contract

The common stream is intentionally normalized. When an exchange-specific
event contains material fields that the common event does not carry, use the
provider's `subscribeDetailed` method instead. It is a different contract, not
an upgrade toggle. The [provider pages](../providers.md) record the supported
detailed streams.

## Do not infer unsupported products

`Client.supports(feature)` tells you whether the configured adapter supports a
common feature. It does not mean a similarly named endpoint on every exchange
has the same data model. Use the generated [API-to-scenario map](../examples.md#api-to-scenario-map)
and [endpoint reference](../../bindings/common/generated/api.md) for the
current public surface.
