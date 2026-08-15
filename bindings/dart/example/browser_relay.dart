import 'package:maxt/maxt.dart';

const _relayUrl = String.fromEnvironment('MAXT_RELAY_URL');

Future<void> main() async {
  // A public read works directly in a browser. A relay URL is only needed when
  // configuring credentials for signed requests.
  await Maxt.initialize(relayUrl: _relayUrl.isEmpty ? null : _relayUrl);
  try {
    final market = Market.spot(Exchange.binance, 'BTC', 'USDT');
    final ticker = await Client(BinanceAdapter.spot()).ticker(market);
    print('${ticker.market}: ${ticker.lastPrice}');
    print(
      'For signed browser calls, use a trusted relay and explicit opt-in; see relay/README.md.',
    );
  } finally {
    await Maxt.dispose();
  }
}
