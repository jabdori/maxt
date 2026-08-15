import 'package:maxt/maxt.dart';

const _address = String.fromEnvironment('HYPERLIQUID_ADDRESS');

Future<void> main() async {
  await Maxt.initialize();
  try {
    final market = Market.perpetual(Exchange.hyperliquid, 'BTC', 'USDC');
    final adapter = HyperliquidAdapter();
    final mids = await adapter.allMids();
    final book = await adapter.l2Book(market);
    final trades = await adapter.recentTrades(market);
    print(
      '${mids.length} mid prices; ${book.bids.length + book.asks.length} levels; ${trades.length} trades',
    );

    if (_address.isEmpty) {
      print('Pass -DHYPERLIQUID_ADDRESS=0x... for address-scoped Info reads.');
    } else {
      final orders = await HyperliquidAdapter(
        address: _address,
      ).basicOpenOrders();
      print('${orders.length} address-scoped open orders');
    }
  } finally {
    await Maxt.dispose();
  }
}
