import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

final class RecordingAdapter extends AdapterBase {
  final List<Object> calls = [];

  @override
  Exchange get exchange => Exchange.binance;

  @override
  Set<Feature> get features => const {
    Feature.trades,
    Feature.orderBook,
    Feature.candles,
    Feature.tradeStream,
    Feature.openOrders,
    Feature.positions,
    Feature.fundingRates,
  };

  @override
  Future<List<Trade>> trades(Market market, [int? limit]) async {
    calls.add(('trades', market, limit));
    return [];
  }

  @override
  Future<OrderBook> orderBook(Market market, [int? depth]) async {
    calls.add(('orderBook', market, depth));
    throw StateError('unexpected orderBook call');
  }

  @override
  Future<List<Candle>> candles(CandleRequest request) async {
    calls.add(('candles', request));
    return [];
  }

  @override
  Future<Page<FundingRate>> fundingRates(HistoryRequest request) async {
    calls.add(('fundingRates', request));
    return Page(items: const []);
  }

  @override
  Future<MarketStream> subscribe(
    Subscription subscription,
    StreamConfig config,
  ) async {
    calls.add(('subscribe', subscription, config));
    return MarketStream(const Stream.empty());
  }

  @override
  Future<List<Order>> openOrders([Market? market]) async {
    calls.add(('openOrders', market));
    return [];
  }

  @override
  Future<List<Position>> positions([Market? market]) async {
    calls.add(('positions', market));
    final selected =
        market ?? Market.perpetual(Exchange.binance, 'BTC', 'USDT');
    return [
      Position(market: selected, quantity: Decimal.parse('0.000')),
      Position(
        market: selected,
        side: Side.buy,
        quantity: Decimal.parse('0.125'),
      ),
    ];
  }
}

void main() {
  setUpAll(Maxt.initialize);
  tearDownAll(Maxt.dispose);

  test('Client는 선택 인자와 기본 StreamConfig를 그대로 전달한다', () async {
    final adapter = RecordingAdapter();
    final client = Client(adapter);
    final market = Market.perpetual(Exchange.binance, 'BTC', 'USDT');
    final subscription = Subscription(markets: [market], feeds: [Feed.trades]);
    const explicit = StreamConfig(bufferSize: 8);

    await client.trades(market);
    final defaultStream = await client.subscribe(subscription);
    final explicitStream = await client.subscribeWith(subscription, explicit);
    await client.openOrders();
    await client.openOrdersOn(market);

    expect(adapter.calls[0], ('trades', market, null));
    final defaultCall =
        adapter.calls[1] as (String, Subscription, StreamConfig);
    final explicitCall =
        adapter.calls[2] as (String, Subscription, StreamConfig);
    expect(defaultCall.$1, 'subscribe');
    expect(defaultCall.$2.markets, subscription.markets);
    expect(defaultCall.$2.feeds, subscription.feeds);
    expect(defaultCall.$3, const StreamConfig());
    expect(explicitCall.$1, 'subscribe');
    expect(explicitCall.$2.markets, subscription.markets);
    expect(explicitCall.$2.feeds, subscription.feeds);
    expect(explicitCall.$3, explicit);
    expect(adapter.calls[3], ('openOrders', null));
    expect(adapter.calls[4], ('openOrders', market));
    await defaultStream.close();
    await explicitStream.close();
  });

  test('Client positions는 수량이 0인 행만 전체·시장별 결과에서 제거한다', () async {
    final adapter = RecordingAdapter();
    final client = Client(adapter);
    final market = Market.perpetual(Exchange.binance, 'ETH', 'USDT');

    final all = await client.positions();
    final onMarket = await client.positionsOn(market);

    expect(all.map((position) => position.quantity.toString()), ['0.125']);
    expect(onMarket.map((position) => position.quantity.toString()), ['0.125']);
    expect(adapter.calls, [('positions', null), ('positions', market)]);
  });

  test('unsigned native StreamConfig 필드를 변환하기 전에 범위 검증한다', () async {
    final adapter = RecordingAdapter();
    final client = Client(adapter);
    final subscription = Subscription(
      markets: [Market.perpetual(Exchange.binance, 'BTC', 'USDT')],
      feeds: [Feed.trades],
    );
    final cases = <({StreamConfig config, String field})>[
      (
        config: const StreamConfig(maxReconnectAttempts: -1),
        field: 'maxReconnectAttempts',
      ),
      (
        config: const StreamConfig(maxReconnectAttempts: 4294967296),
        field: 'maxReconnectAttempts',
      ),
      (
        config: const StreamConfig(initialReconnectDelayMs: -1),
        field: 'initialReconnectDelayMs',
      ),
      (
        config: const StreamConfig(maxReconnectDelayMs: -1),
        field: 'maxReconnectDelayMs',
      ),
      (config: const StreamConfig(idleTimeoutMs: -1), field: 'idleTimeoutMs'),
      (config: const StreamConfig(bufferSize: -1), field: 'bufferSize'),
    ];

    for (final testCase in cases) {
      await expectLater(
        client.subscribeWith(subscription, testCase.config),
        throwsA(
          isA<InvalidRequestError>().having(
            (error) => error.field,
            'field',
            testCase.field,
          ),
        ),
      );
    }

    expect(adapter.calls, isEmpty);
  });

  test('u32 공개 요청 인자를 사용자 정의 Adapter 호출 전에 범위 검증한다', () async {
    final adapter = RecordingAdapter();
    final client = Client(adapter);
    final market = Market.perpetual(Exchange.binance, 'BTC', 'USDT');
    final cases =
        <({String field, Future<Object?> Function(int value) invoke})>[
          (field: 'limit', invoke: (value) => client.trades(market, value)),
          (field: 'depth', invoke: (value) => client.orderBook(market, value)),
          (
            field: 'limit',
            invoke: (value) => client.candles(
              CandleRequest(market, Interval.min1, limit: value),
            ),
          ),
          (
            field: 'limit',
            invoke: (value) =>
                client.fundingRates(HistoryRequest(market, limit: value)),
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

    expect(adapter.calls, isEmpty);
  });
}
