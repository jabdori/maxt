import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();
  try {
    final market = Market.perpetual(Exchange.binance, 'BTC', 'USDT');
    final adapter = BinanceAdapter.usdMFutures();
    final mark = await adapter.markPrice(market);
    final interest = await adapter.openInterest(market);
    print('mark=${mark.markPrice}; open interest=${interest.openInterest}');

    final funding = await Client(
      adapter,
    ).fundingRates(HistoryRequest(market, limit: 5));
    print('${funding.items.length} funding rows; next=${funding.next}');
  } finally {
    await Maxt.dispose();
  }
}
