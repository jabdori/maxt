import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

void main() {
  setUpAll(Maxt.initialize);
  tearDownAll(Maxt.dispose);

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

  test('공급자 전용 u32 인자를 native 호출 전에 범위 검증한다', () async {
    final upbit = UpbitAdapter();
    final hyperliquid = HyperliquidAdapter();
    final cases =
        <({String field, Future<Object?> Function(int value) invoke})>[
          (
            field: 'depth',
            invoke: (value) => upbit.orderBooks(const [], value),
          ),
          (
            field: 'limit',
            invoke: (value) => hyperliquid.nonFundingLedger(limit: value),
          ),
        ];

    for (final testCase in cases) {
      for (final value in [-1, 4294967296]) {
        await expectLater(
          testCase.invoke(value),
          throwsA(
            isA<InvalidRequestError>().having(
              (error) => error.field,
              'field',
              testCase.field,
            ),
          ),
        );
      }
    }
  });

  test('built-in Adapter는 빈 구독을 native 호출 전에 거절한다', () async {
    final adapter = UpbitAdapter();
    final market = Market.spot(Exchange.upbit, 'BTC', 'KRW');

    await expectLater(
      adapter.subscribe(Subscription(), const StreamConfig()),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'markets',
        ),
      ),
    );
    await expectLater(
      adapter.subscribe(Subscription(markets: [market]), const StreamConfig()),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'feeds',
        ),
      ),
    );
  });
}
