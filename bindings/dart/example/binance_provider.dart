import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();
  try {
    final market = Market.spot(Exchange.binance, 'BTC', 'USDT');
    final spot = BinanceAdapter.spot();
    final average = await spot.spotAveragePrice(market);
    final filters = await spot.spotSymbolFilters(market);
    final exchange = await spot.spotExchangeInfo();
    print(
      '${average.minutes}-minute average=${average.price}; tick=${filters.tickSize}',
    );
    print('Spot symbols: ${exchange.symbols.length}');

    final futures = BinanceAdapter.usdMFutures();
    final metadata = await futures.usdMExchangeInfo();
    print('USD-M symbols: ${metadata.symbols.length}');
  } finally {
    await Maxt.dispose();
  }
}
