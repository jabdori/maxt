@TestOn('browser')
library;

import 'dart:async';

import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

final class _ConfigAdapter extends AdapterBase {
  StreamConfig? received;
  int tradeCalls = 0;

  @override
  Exchange get exchange => Exchange.binance;

  @override
  Set<Feature> get features => const {Feature.tradeStream};

  @override
  Future<List<Trade>> trades(Market market, [int? limit]) async {
    tradeCalls++;
    return const [];
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
}
