import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();
  try {
    final adapter = BithumbAdapter();
    final warnings = await adapter.marketWarnings();
    final notices = await adapter.notices(5);
    final fees = await adapter.transferFees('BTC');
    print(
      '${warnings.length} warning rows, ${notices.length} notices, ${fees.length} BTC fee rows',
    );
  } finally {
    await Maxt.dispose();
  }
}
