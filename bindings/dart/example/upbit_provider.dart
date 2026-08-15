import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();
  try {
    final adapter = UpbitAdapter();
    final market = Market.spot(Exchange.upbit, 'BTC', 'KRW');
    final tickers = await adapter.tickers([market]);
    final instruments = await adapter.orderbookInstruments([market]);
    print('region=${adapter.region}; ticker rows=${tickers.length}');
    print('order-book instrument rows=${instruments.length}');
  } finally {
    await Maxt.dispose();
  }
}
