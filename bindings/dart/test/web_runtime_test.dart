@TestOn('browser')
library;

import 'dart:async';

import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

final class _ConfigAdapter extends AdapterBase {
  StreamConfig? received;
  int tradeCalls = 0;
  int depositCalls = 0;
  int withdrawalCalls = 0;

  @override
  Exchange get exchange => Exchange.binance;

  @override
  Set<Feature> get features => const {
    Feature.tradeStream,
    Feature.depositLookup,
    Feature.withdrawalLookup,
  };

  @override
  Future<List<Trade>> trades(Market market, [int? limit]) async {
    tradeCalls++;
    return const [];
  }

  @override
  Future<Deposit> deposit(TransferLookupRequest request) async {
    depositCalls++;
    throw StateError('invalid deposit lookup reached the custom adapter');
  }

  @override
  Future<Withdrawal> withdrawal(TransferLookupRequest request) async {
    withdrawalCalls++;
    throw StateError('invalid withdrawal lookup reached the custom adapter');
  }

  @override
  Future<MarketStream> subscribe(
    Subscription subscription,
    StreamConfig config,
  ) async {
    received = config;
    return MarketStream(const Stream<StreamItem<MarketEvent>>.empty());
  }
}

void main() {
  setUpAll(Maxt.initialize);
  tearDownAll(Maxt.dispose);

  test('WebAssembly 런타임은 공개 Adapter와 정확한 Timestamp를 제공한다', () {
    final timestamp = Timestamp.fromNanoseconds(
      BigInt.parse('1700000000123456789'),
    );
    expect(timestamp.nanosecondsSinceEpoch.toString(), '1700000000123456789');
    expect(BinanceAdapter.spot().exchange, Exchange.binance);
    expect(
      HyperliquidAdapter(
        address: '0x14791697260e4c9a71f18484c9f997b308e59325',
      ).exchange,
      Exchange.hyperliquid,
    );
  });

  test('명시적으로 허용하지 않은 브라우저 인증 정보를 거부한다', () {
    expect(
      () => BinanceAdapter.spot(apiKey: 'key', secretKey: 'secret'),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'allowInsecureBrowserCredentials',
        ),
      ),
    );
    expect(
      () => HyperliquidAdapter(privateKey: 'signer'),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'allowInsecureBrowserCredentials',
        ),
      ),
    );
  });

  test('WebAssembly도 Upbit 호가 묶음 단위를 native와 동일하게 검증한다', () async {
    final upbit = UpbitAdapter();
    final market = Market.spot(Exchange.upbit, 'BTC', 'KRW');

    await expectLater(
      upbit.orderBooksAtLevel([market], Decimal.parse('-1')),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'level',
        ),
      ),
    );
  });

  test('WebAssembly도 Bithumb 공지 개수를 검증한다', () async {
    await expectLater(
      BithumbAdapter().notices(0),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'count',
        ),
      ),
    );
  });

  test('WebAssembly도 Bithumb 수수료 조회의 빈 자산 코드를 거절한다', () async {
    await expectLater(
      BithumbAdapter().transferFees(' '),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'currency',
        ),
      ),
    );
  });

  test('WebAssembly도 자격증명 없는 Bithumb API 키 조회를 거절한다', () async {
    await expectLater(
      BithumbAdapter().apiKeys(),
      throwsA(isA<AuthenticationError>()),
    );
  });

  test('WebAssembly도 자격증명 없는 Bithumb 대기 주문 조회를 거절한다', () async {
    await expectLater(
      BithumbAdapter().pendingOrders(const BithumbPendingOrdersRequest()),
      throwsA(isA<AuthenticationError>()),
    );
  });

  test('WebAssembly도 자격증명 없는 Bithumb TWAP 조회를 거절한다', () async {
    await expectLater(
      BithumbAdapter().twapOrders(const BithumbTwapOrdersRequest()),
      throwsA(isA<AuthenticationError>()),
    );
  });

  test('WebAssembly도 자격증명 없는 Upbit 테스트 주문을 거절한다', () async {
    final market = Market.spot(Exchange.upbit, 'BTC', 'KRW');

    await expectLater(
      UpbitAdapter().testOrder(
        OrderRequest.limit(
          market,
          Side.buy,
          Size.base(Decimal.parse('0.01')),
          Decimal.parse('100000000'),
        ),
      ),
      throwsA(isA<AuthenticationError>()),
    );
  });

  test('WebAssembly도 자격증명 없는 Upbit 입금 가능 정보 조회를 거절한다', () async {
    await expectLater(
      UpbitAdapter().depositInfo('BTC', Network.bitcoin),
      throwsA(isA<AuthenticationError>()),
    );
  });

  test('WebAssembly도 자격증명 없는 Upbit 조건부 일괄 취소를 거절한다', () async {
    await expectLater(
      UpbitAdapter().batchCancelOpenOrders(
        const UpbitBatchCancelRequest(scope: UpbitBatchCancelScope.all()),
      ),
      throwsA(isA<AuthenticationError>()),
    );
  });

  test('기본 Web 스트림은 네트워크 backpressure를 사용하지 않는다', () async {
    final adapter = _ConfigAdapter();
    final client = Client(adapter);
    final stream = await client.subscribe(
      Subscription(
        markets: [Market.spot(Exchange.binance, 'BTC', 'USDT')],
        feeds: [Feed.trades],
      ),
    );
    expect(adapter.received?.overflow, Overflow.dropNewest);
    await stream.close();
  });

  test('Web 사용자 정의 Adapter 호출 전 unsigned 범위를 검사한다', () async {
    final adapter = _ConfigAdapter();
    final client = Client(adapter);
    final market = Market.spot(Exchange.binance, 'BTC', 'USDT');

    await expectLater(
      client.trades(market, -1),
      throwsA(isA<InvalidRequestError>()),
    );
    await expectLater(
      client.subscribeWith(
        Subscription(markets: [market], feeds: [Feed.trades]),
        const StreamConfig(bufferSize: -1),
      ),
      throwsA(isA<InvalidRequestError>()),
    );
    expect(adapter.tradeCalls, 0);
    expect(adapter.received, isNull);
  });

  test('Web 사용자 정의 Adapter에 유효하지 않은 입출금 조회를 전달하지 않는다', () async {
    final adapter = _ConfigAdapter();
    final client = Client(adapter);

    await expectLater(
      client.deposit(TransferLookupRequest(asset: 'BTC')),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'reference',
        ),
      ),
    );
    await expectLater(
      client.withdrawal(
        TransferLookupRequest(asset: 'BTC', id: 'withdrawal-1', txId: 'tx-1'),
      ),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'reference',
        ),
      ),
    );
    await expectLater(
      client.withdrawal(TransferLookupRequest(asset: 'BTC', txId: '  ')),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'tx_id',
        ),
      ),
    );

    expect(adapter.depositCalls, 0);
    expect(adapter.withdrawalCalls, 0);
  });
}
