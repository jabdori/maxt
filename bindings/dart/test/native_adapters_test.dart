import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

void main() {
  setUpAll(Maxt.initialize);

  test('built-in Adapter는 private Rust handle의 설정과 기능을 노출한다', () {
    final upbit = UpbitAdapter.withRegion(UpbitRegion.singapore);
    final authenticated = UpbitAdapter(accessKey: 'key', secretKey: 'secret');
    final binance = BinanceAdapter.usdMFutures();
    final hyperliquid = HyperliquidAdapter.testnet();

    expect(upbit.exchange, Exchange.upbit);
    expect(upbit.region, UpbitRegion.singapore);
    expect(upbit.supports(Feature.markets), isTrue);
    expect(upbit.supports(Feature.trading), isFalse);
    expect(authenticated.supports(Feature.trading), isTrue);
    expect(binance.venue, BinanceMarket.usdMFutures);
    expect(hyperliquid.isTestnet, isTrue);
    expect(Client(upbit).adapter, same(upbit));
  });

  test('built-in Adapter는 반쪽 자격증명을 구조화된 오류로 바꾼다', () {
    expect(
      () => BithumbAdapter(accessKey: 'key'),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'credentials',
        ),
      ),
    );
  });
}
