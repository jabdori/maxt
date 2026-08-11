# maxt for Dart and Flutter

[English](README.md) | [한국어](README.ko.md)

One Dart and Flutter API for the same operations, models, errors, and streams
on native platforms and the Web. Native builds use Dart build hooks; Web builds
use WebAssembly.

## Support

- [x] Android
- [x] iOS
- [x] Linux
- [x] macOS
- [x] Windows
- [x] Dart Web

Dart 3.10 or a compatible Flutter SDK is required. This package does not
download prebuilt native libraries. Its build hook compiles the included Rust
source when your Dart or Flutter application is built, so Rustup and the target
platform toolchain, such as the Android NDK or Xcode, must also be installed in
development and CI environments. Web builds additionally require the Rust
nightly toolchain with `rust-src` and `wasm-pack`.

## Supported exchanges

- Upbit Spot: Korea, Singapore, Indonesia, and Thailand
- Bithumb Spot
- Binance Spot and USD-M perpetual futures
- Hyperliquid Spot and perpetual futures on mainnet and testnet

Binance testnet constructors are not exposed. Hyperliquid HIP-3 perpetual DEXs
and outcome assets are not exposed.

## Common API

`Client` provides the same method names for every built-in adapter:

- Public REST: `markets()`, `trades()`, `orderBook()`, `ticker()`, and
  `candles()`.
- Public streams: `subscribe()` and `subscribeWith()` for trades, order books,
  tickers, and candles. Bithumb does not support candle streams.
- Public funding history: `fundingRates()` on Binance USD-M and Hyperliquid
  perpetual markets.
- Private Spot: `balances()`, `openOrders()`, `placeOrder()`, `cancelOrder()`,
  and `subscribeAccount()` on every exchange.
- Private order lookup: `order()`, `orderByClientId()`, `ordersByIds()`, and
  `orderHistory()` on Upbit and Bithumb.
- Private order rules: `orderRules()` on Upbit and Bithumb.
- Private batch cancellation: `cancelOrders()` on Upbit and Bithumb.
- Private wallet lookup and cancellation: `deposit()`, `withdrawal()`, and
  `cancelWithdrawal()` on Upbit and Bithumb. Lookups require an asset and one
  exchange ID or transaction ID; cancellation must be followed by a lookup.
- Private perpetuals: `positions()`, `marginSummary()`, `setMargin()`, and
  `fundingPayments()` on Binance USD-M and Hyperliquid.

Public calls need no credentials. Private calls require both credential fields.
Use `client.supports(feature)` before optional operations when the adapter or
credential state is dynamic.

## Exchange-specific API

Exchange-specific methods remain available through `client.adapter`.

| Adapter | Construction | Additional methods |
| --- | --- | --- |
| `UpbitAdapter` | `UpbitAdapter()` or `UpbitAdapter.withRegion(...)` | `orderBooks()`, `orderBooksAtLevel()`, `tickers()`, `tickersByQuote()`, `yearCandles()`, `orderbookInstruments()`, `marketEvents()`; authenticated: `testOrder()` |
| `BithumbAdapter` | `BithumbAdapter()` | `marketWarnings()`, `marketAlerts()`, `notices()`, `transferFees()`; authenticated: `apiKeys()`, `pendingOrders()` |
| `BinanceAdapter` | `BinanceAdapter.spot()` | `spotSymbolFilters()`; authenticated: `spotOrder()` |
| `BinanceAdapter` | `BinanceAdapter.usdMFutures()` | Authenticated: `usdMCreateListenKey()`, `usdMKeepaliveListenKey()`, `usdMCloseListenKey()` |
| `HyperliquidAdapter` | `HyperliquidAdapter()` or `HyperliquidAdapter.testnet()` | `assetContext()`, `nonFundingLedger()` |

`UpbitAdapter.testOrder()` validates an order without creating it. The returned
`Order` is a dry-run result: do not query or cancel its `id`, and do not treat
its status as a live order.

## Install

```sh
dart pub add maxt
```

For a Web application, build the package's WebAssembly files into `web/pkg`
before running or building the application:

```sh
rustup toolchain install nightly --component rust-src --target wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
dart run maxt:build_web --release
flutter build web
```

Run `dart run maxt:build_web --release` from the application root. The command
uses the installed `maxt` package, so it also works when `maxt` comes from
pub.dev. Do not commit the generated `web/pkg` files unless your deployment
process requires built assets in source control.

Serve the Web build with `Cross-Origin-Opener-Policy: same-origin` and
`Cross-Origin-Embedder-Policy: require-corp` so browsers can enable the shared
memory features used by the generated WebAssembly module. Use HTTPS in
production; `http://localhost` is suitable for local development.

## Initialize and use Binance

Call `Maxt.initialize()` once in each isolate before constructing an adapter.
Call `Maxt.dispose()` before the isolate exits. A disposed isolate cannot be
initialized again.

```dart
import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();

  final client = Client(BinanceAdapter.spot());
  final market = Market.spot(Exchange.binance, 'BTC', 'USDT');

  final ticker = await client.ticker(market);
  final filters = await client.adapter.spotSymbolFilters(market);

  print(ticker.lastPrice);
  print(filters.tickSize);

  await Maxt.dispose();
}
```

`ticker()` is common. `spotSymbolFilters()` is Binance Spot-specific and is
available through `client.adapter`.

In a browser, public calls can use direct HTTP and WebSocket connections when
the exchange permits them. Set `relayUrl` when a relay is required:

```dart
await Maxt.initialize(relayUrl: 'https://relay.example');
```

Browser credentials are disabled by default because JavaScript and WebAssembly
memory are not secret storage. Credentialed browser calls require both a relay
and explicit opt-in:

```dart
await Maxt.initialize(
  relayUrl: 'https://relay.example',
  allowInsecureBrowserCredentials: true,
);
```

Use restricted exchange keys without withdrawal permission. Keep credentials
on a trusted backend when they must not be exposed to the browser.

Deploy the relay behind an authenticated, rate-limited TLS ingress on the same
site as the application. The relay does not authenticate users, and its Origin
allowlist is not authentication. See the [relay deployment and security
requirements](../../relay/README.md).

## Streams

```dart
final stream = await client.subscribe(
  Subscription(markets: [market], feeds: [Feed.trades]),
);
try {
  await for (final item in stream) {
    switch (item) {
      case StreamEvent(:final event):
        print(event);
      case StreamError(:final error):
        print(error);
    }
  }
} finally {
  await stream.close();
}
```

`StreamError` does not terminate the stream. `close()` waits for native cleanup.

## Custom adapters

Extend `AdapterBase`, implement `exchange` and `features`, then override every
advertised operation. Wrap the instance with `Client(adapter)`. Default
methods return `UnsupportedError`.

For custom streams, return `MarketStream` or `AccountStream` over a Dart
`Stream<StreamItem<T>>`. Pass `onClose` when cleanup is required.

## Contracts

- `Decimal`: exact 96-bit coefficient, scale `0..=28`.
- `Timestamp`: signed 64-bit Unix epoch nanoseconds stored as `BigInt`.
- Errors: `InvalidRequestError`, `UnsupportedError`, `AdapterError`, `AuthenticationError`, `ExchangeError`, `TransportError`, `DecodeError`.
- Credentials: omit both fields for public access; provide both for private access.

See the [common data and pagination contracts](../../docs/common-api.md) and
[provider limits and data semantics](../../docs/providers.md).

## License

MIT
