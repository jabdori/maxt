# Provider support

[English](providers.md) | [한국어](providers.ko.md)

All built-in adapters expose public market data without credentials. Use
common operations through `Client` and exchange-specific operations through
`Client::adapter()`.

## Status

- [x] Upbit Spot: Korea, Singapore, Indonesia, Thailand
- [x] Bithumb Spot
- [x] Binance Spot
- [x] Binance USD-M perpetual futures
- [x] Hyperliquid Spot and perpetual futures

Binance testnet constructors are not exposed.

## Constructors

- Upbit Korea: `UpbitAdapter::new()`
- Upbit regional: `UpbitAdapter::with_region(region)`
- Bithumb: `BithumbAdapter::new()`
- Binance Spot: `BinanceAdapter::spot()`
- Binance USD-M: `BinanceAdapter::usd_m_futures()`
- Hyperliquid mainnet: `HyperliquidAdapter::new()`
- Hyperliquid testnet: `HyperliquidAdapter::testnet()`

Add credentials before `Client::new(adapter)`:

- Upbit and Bithumb: `.with_credentials(access_key, secret_key)`
- Binance: `.with_credentials(api_key, secret_key)`
- Hyperliquid: `.with_wallet(address, private_key)`

`supports(feature) == false` means credentials are absent or the operation is
structurally unsupported. The call returns `Error::Auth` or
`Error::Unsupported`, respectively.

## Boundaries

- Upbit regions use separate hosts, listings, books, and credentials.
- Bithumb has no `Feature::CandleStream`; `open_orders()` reads one provider page.
- Binance Spot and USD-M use separate adapters and state.
- One Binance USD-M `Subscription` may merge multiple WebSockets.
- Hyperliquid `positions_on(spot)` returns `Ok(vec![])`.

## References

- [Upbit](providers/upbit.md) ([한국어](providers/upbit.ko.md))
- [Bithumb](providers/bithumb.md) ([한국어](providers/bithumb.ko.md))
- [Binance](providers/binance.md) ([한국어](providers/binance.ko.md))
- [Hyperliquid](providers/hyperliquid.md) ([한국어](providers/hyperliquid.ko.md))
