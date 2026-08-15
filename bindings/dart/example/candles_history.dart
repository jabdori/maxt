import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();
  try {
    final market = Market.spot(Exchange.binance, 'BTC', 'USDT');
    final client = Client(BinanceAdapter.spot());
    final candles = await client.candles(
      CandleRequest(market, Interval.min1, limit: 5),
    );
    for (final candle in candles) {
      print(
        '${candle.openTime}: close=${candle.close} volume=${candle.volume}',
      );
    }

    // Reuse a previous page cursor only when the previous response has one.
    final history = HistoryRequest(market, limit: 100);
    print('Private history request prepared for ${history.market}');
  } finally {
    await Maxt.dispose();
  }
}
