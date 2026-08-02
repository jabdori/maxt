# maxt for Dart and Flutter

[English](README.md) | [한국어](README.ko.md)

One native Dart API for the same operations, models, errors, and streams as
the Rust contract. Dart build hooks compile the Rust crate for the target, and
generated contracts are checked against the native API.

## Support

- [x] Android
- [x] iOS
- [x] Linux
- [x] macOS
- [x] Windows
- [ ] Dart Web

Dart 3.10 or a compatible Flutter SDK, Rustup, and target build tools are required.

## Install

```sh
dart pub add maxt
```

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
- `Timestamp`: signed 64-bit Unix epoch nanoseconds.
- Errors: `InvalidRequestError`, `UnsupportedError`, `AdapterError`, `AuthenticationError`, `ExchangeError`, `TransportError`, `DecodeError`.
- Credentials: omit both fields for public access; provide both for private access.

See the [common API](../../docs/common-api.md) and [provider support](../../docs/providers.md).

## License

MIT
