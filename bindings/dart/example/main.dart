import 'package:maxt/maxt.dart';

Future<void> main() async {
  final market = Market.spot(Exchange.binance, 'BTC', 'USDT');
  print('Prepared public market: $market');

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
