import 'package:maxt/maxt.dart';

/// Reads one public Binance Spot quote only when this value is `true`.
///
/// The default run does not initialize a native library or connect to the
/// network. Enable the live public read explicitly:
///
/// ```sh
/// dart run -DMAXT_RUN_PUBLIC_READ=true example/main.dart
/// ```
const _runPublicRead = bool.fromEnvironment('MAXT_RUN_PUBLIC_READ');

Future<void> main() async {
  final market = Market.spot(Exchange.binance, 'BTC', 'USDT');
  print('Prepared public market: $market');

  if (!_runPublicRead) {
    print('Set MAXT_RUN_PUBLIC_READ=true to perform the public read.');
    return;
  }

  try {
    await Maxt.initialize();

    // Reads public market data without credentials, orders, or transfers.
    final client = Client(BinanceAdapter.spot());
    final ticker = await client.ticker(market);
    final average = await client.adapter.spotAveragePrice(market);
    print('${ticker.market}: ${ticker.lastPrice}');
    print('Binance ${average.minutes}-minute average: ${average.price}');
  } on MaxtError catch (error) {
    print('Could not read the public quote: $error');
  } catch (error) {
    print('Unexpected error: $error');
  } finally {
    if (Maxt.isInitialized) await Maxt.dispose();
  }
}
