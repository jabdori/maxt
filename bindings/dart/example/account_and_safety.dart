import 'package:maxt/maxt.dart';

const _accessKey = String.fromEnvironment('UPBIT_ACCESS_KEY');
const _secretKey = String.fromEnvironment('UPBIT_SECRET_KEY');

Future<void> main() async {
  final market = Market.spot(Exchange.upbit, 'BTC', 'KRW');
  final draft = OrderRequest.limit(
    market,
    Side.buy,
    Size.base(Decimal.parse('0.0001')),
    Decimal.parse('100000'),
  ).withClientId('docs-example-only');
  final history = TransferHistoryRequest(
    asset: 'BTC',
    network: Network.bitcoin,
    limit: 20,
  );
  print('Order draft only; it was not sent: ${draft.clientId}');
  print('Transfer-history request: ${history.asset}');

  if (_accessKey.isEmpty || _secretKey.isEmpty) {
    print(
      'Pass -DUPBIT_ACCESS_KEY=... -DUPBIT_SECRET_KEY=... for read-only account data.',
    );
    return;
  }

  await Maxt.initialize();
  try {
    final client = Client(
      UpbitAdapter(accessKey: _accessKey, secretKey: _secretKey),
    );
    final balances = await client.balances();
    final orders = await client.openOrders();
    print('${balances.length} balances and ${orders.length} open orders');
  } finally {
    await Maxt.dispose();
  }
}
