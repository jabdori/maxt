import 'package:maxt/maxt.dart';
import 'package:maxt/src/adapters.dart';
import 'package:maxt/src/rust/adapter.dart' as native_adapter;
import 'package:maxt/src/rust/convert.dart' as wire;
import 'package:test/test.dart';

final class EmptyFeedAdapter extends AdapterBase {
  @override
  Exchange get exchange => Exchange.binance;

  @override
  Set<Feature> get features => const {Feature.tradeStream};
}

void main() {
  setUpAll(Maxt.initialize);

  test('Subscribe 초기 실패는 원래 오류를 보존하고 registry를 비운다', () async {
    final adapter = EmptyFeedAdapter();
    final bridge = DartAdapterBridge(adapter);
    final nativeClient = await bridge.register();
    final market = const wire.WireMarket(
      exchange: wire.WireExchange.binance,
      kind: wire.WireMarketKind.perpetual,
      base: 'BTC',
      quote: 'USDT',
    );
    final subscription = native_adapter.WireSubscription(
      markets: [market],
      feeds: const [],
    );
    final config = native_adapter.WireStreamConfig(
      initialReconnectDelayMs: BigInt.from(1000),
      maxReconnectDelayMs: BigInt.from(30000),
      idleTimeoutMs: BigInt.from(30000),
      bufferSize: BigInt.from(4096),
      overflow: native_adapter.WireOverflow.backpressure,
    );

    for (var attempt = 0; attempt < 20; attempt++) {
      await expectLater(
        nativeClient.subscribe(subscription: subscription, config: config),
        throwsA(
          isA<wire.NativeError>()
              .having(
                (error) => error.kind,
                'kind',
                wire.NativeErrorKind.invalidRequest,
              )
              .having((error) => error.field, 'field', 'feeds'),
        ),
      );
    }

    expect(bridge.streams.activeStreamCount, 0);
    expect(bridge.streams.pendingCancellationCount, 0);

    final client = Client(adapter);
    await expectLater(
      client.subscribe(
        Subscription(
          markets: [Market.perpetual(Exchange.binance, 'BTC', 'USDT')],
        ),
      ),
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
