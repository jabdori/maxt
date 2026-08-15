import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();
  try {
    final market = Market.spot(Exchange.binance, 'BTC', 'USDT');
    final client = Client(BinanceAdapter.spot());
    final stream = await client.subscribe(
      Subscription(markets: [market], feeds: [Feed.trades]),
    );

    var remaining = 3;
    await for (final item in stream) {
      print(item);
      if (--remaining == 0) break;
    }
  } finally {
    await Maxt.dispose();
  }
}
