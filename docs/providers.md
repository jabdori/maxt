# Provider support

[English](providers.md) | [한국어](providers.ko.md)

All built-in adapters expose public market data and market streams without
account configuration. Use common operations through `Client` and
exchange-specific operations through `Client::adapter()`.

Start with the [task-oriented examples](examples.md), then use the provider
pages below for exchange limits and official references.

## Status

- [x] Binance Spot
- [x] Binance USD-M perpetual futures
- [x] Upbit Spot: Korea, Singapore, Indonesia, Thailand
- [x] Bithumb Spot
- [x] Hyperliquid Spot and perpetual futures

Binance testnet constructors are not exposed.

## Constructors

- Binance Spot: `BinanceAdapter::spot()`
- Binance USD-M: `BinanceAdapter::usd_m_futures()`
- Upbit Korea: `UpbitAdapter::new()`
- Upbit regional: `UpbitAdapter::with_region(region)`
- Bithumb: `BithumbAdapter::new()`
- Hyperliquid mainnet: `HyperliquidAdapter::new()`
- Hyperliquid testnet: `HyperliquidAdapter::testnet()`

Configure account access before `Client::new(adapter)`:

- Binance: `.with_credentials(api_key, secret_key)`
- Upbit and Bithumb: `.with_credentials(access_key, secret_key)`
- Hyperliquid account reads: `.with_query_address(address)`; no local
  signature is used
- Hyperliquid signed actions: `.with_signer(private_key)`
- Hyperliquid convenience configuration: `.with_wallet(address, private_key)`
  configures both

`supports(feature) == false` means required adapter configuration is absent or
the operation is structurally unsupported. The corresponding call returns
`Error::Auth` or `Error::Unsupported`, respectively. Provider permissions,
market selection, and request validation remain separate checks.

## Boundaries

- Binance Spot and USD-M use separate adapters and state.
- One Binance USD-M `Subscription` may merge multiple WebSockets.
- Upbit regions use separate hosts, listings, books, and credentials.
- Bithumb has no `Feature::CandleStream`; `open_orders()` reads one provider page.
- Hyperliquid `positions_on(spot)` returns `Ok(vec![])`.

## References

Use the generated [endpoint coverage reference](../bindings/common/generated/api.md)
for recorded operation mapping plus implementation and validation status. Use
the provider pages for constructors, market/region limits, and official links.

- [Binance](providers/binance.md) ([한국어](providers/binance.ko.md))
- [Upbit](providers/upbit.md) ([한국어](providers/upbit.ko.md))
- [Bithumb](providers/bithumb.md) ([한국어](providers/bithumb.ko.md))
- [Hyperliquid](providers/hyperliquid.md) ([한국어](providers/hyperliquid.ko.md))
