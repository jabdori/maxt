# maxt for Dart and Flutter

`maxt` exposes one native Dart API for Upbit, Bithumb, Binance, and
Hyperliquid. The package builds its Rust crate through Dart build hooks; it
does not ship prebuilt native binaries.

## Requirements

- Dart 3.10 or later, or a compatible Flutter SDK
- Rustup and the platform's native build tools
- Android, iOS, Linux, macOS, or Windows; web is not supported

## Install

Add the package to a Dart or Flutter project:

```sh
dart pub add maxt
```

## Initialize and call the common API

Initialize the native runtime once in each isolate before constructing an
adapter:

```dart
import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();

  final client = Client(BinanceAdapter.spot());
  final market = Market.spot(Exchange.binance, 'BTC', 'USDT');
  final ticker = await client.ticker(market);

  print('$market: ${ticker.lastPrice}');
  await Maxt.dispose();
}
```

Credentials are optional for public data and required for private account or
trading methods. Pass both fields together when constructing an authenticated
adapter.

## Value contracts

| Value | Contract |
| --- | --- |
| `Decimal` | `parse()` accepts only exact values with a 96-bit coefficient and scale `0..=28`; comparison, `+`, and `-` use decimal arithmetic; native validation never rounds or truncates |
| `Timestamp` | Signed 64-bit Unix epoch nanoseconds; unit constructors saturate, while smaller-unit getters truncate toward the epoch |
| `Interval` | `seconds` is `null` for `month1`; `advance()` uses UTC calendar months and returns `null` on overflow |
| Common models | `OrderBook` exposes best prices, spread, and midpoint; `Balance.total`, `Position.isFlat`, and `Page.hasMore` match the Rust helpers |
| Common enums | Exchange, feature, market-kind, side, order-status, and exchange-error helpers expose the same classifications as Rust |
| `HyperliquidLedgerKind` | Unknown provider names preserve `providerName` with `isOther == true` |

Plain and scientific decimal strings are accepted when exact. Values outside
the native range throw before crossing the Rust boundary.

## Errors

Calls throw structured exceptions such as `InvalidRequestError`,
`UnsupportedError`, `AuthenticationError`, `ExchangeError`, `TransportError`,
and `DecodeError`. `ExchangeError` preserves the provider code, status, and
retry classification.

## Streams

Streams emit `StreamEvent` or non-terminal `StreamError` values. Call
`await stream.close()` or cancel its subscription to await native cleanup;
cleanup errors are returned to the caller.

Call `await Maxt.dispose()` before an isolate exits. A disposed isolate cannot
initialize the native runtime again.

## Adapters

For a custom adapter, extend `AdapterBase`, provide `exchange` and `features`,
and override the methods for every advertised feature. Unimplemented methods
return `UnsupportedError`.

Provider-specific methods remain on the concrete adapter:

```dart
final adapter = client.adapter;
```

See the provider references for those methods and their contracts.

See the [maxt repository](https://github.com/jabdori/maxt) for the common API
and provider contracts.

## License

MIT
