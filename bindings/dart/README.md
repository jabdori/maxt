# maxt for Dart and Flutter

`maxt` exposes one native Dart API for Upbit, Bithumb, Binance, and
Hyperliquid market data, accounts, orders, and live streams. The package builds
the bundled Rust crate through Dart build hooks; it does not ship prebuilt
native binaries.

## Requirements

- Dart 3.10 or later, or a compatible Flutter SDK
- Rustup and the platform's native build tools
- Android, iOS, Linux, macOS, or Windows; web is not supported

## Install

After the first pub.dev release, add `maxt` to your application's
`pubspec.yaml`, then run `dart pub get` or `flutter pub get`.

Before that release, use a repository checkout:

```yaml
dependencies:
  maxt:
    path: ../maxt/bindings/dart
```

## Initialize and call the common API

Initialize the native runtime once in each isolate before constructing an
adapter:

```dart
import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();

  final client = Client(UpbitAdapter());
  final markets = await client.markets(MarketKind.spot);

  print('Loaded ${markets.length} Upbit spot markets');
  await Maxt.dispose();
}
```

Credentials are optional for public data and required for private account or
trading methods. Pass both fields together when constructing an authenticated
adapter.

## API surface

`Client` provides markets, trades, order books, tickers, candles, public and
account streams, balances, orders, positions, margin, and funding history.

| Adapter | Provider-specific API |
| --- | --- |
| `UpbitAdapter` | `region`, `orderBooks`, `tickers`, `marketEvents` |
| `BithumbAdapter` | `marketWarnings`, `marketAlerts` |
| `BinanceAdapter` | `venue`, `spotSymbolFilters`, `spotOrder`, `usdMCreateListenKey`, `usdMKeepaliveListenKey`, `usdMCloseListenKey` |
| `HyperliquidAdapter` | `isTestnet`, `nonFundingLedger`, `assetContext` |

Calls throw structured `InvalidRequestError`, `UnsupportedError`,
`AdapterError`, `AuthenticationError`, `ExchangeError`, `TransportError`, or
`DecodeError` values. `ExchangeError` preserves the provider code, status, and
retry classification.

## Streams

Streams emit `StreamEvent` or non-terminal `StreamError` values. Call
`await stream.close()` or cancel its subscription to await native cleanup;
cleanup errors are returned to the caller.

Call `await Maxt.dispose()` before an isolate exits. A disposed isolate cannot
initialize the native runtime again.

See the [maxt repository](https://github.com/jabdori/maxt) for the common API
and provider contracts.

## License

MIT
