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

Add `maxt` to your application's `pubspec.yaml`, then run `dart pub get` or
`flutter pub get`.

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
}
```

Credentials are optional for public data and required for private account or
trading methods. Pass both fields together when constructing an authenticated
adapter.

See the [maxt repository](https://github.com/jabdori/maxt) for the common API
and provider contracts.

## License

MIT
