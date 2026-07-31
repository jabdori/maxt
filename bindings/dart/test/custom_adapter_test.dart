import 'dart:async';

import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

final class ReplayAdapter extends AdapterBase {
  ReplayAdapter(this.replayedTrades);

  final List<Trade> replayedTrades;
  int closeCount = 0;

  @override
  Exchange get exchange => Exchange.upbit;

  @override
  Set<Feature> get features => const {
    Feature.trades,
    Feature.balances,
    Feature.tradeStream,
  };

  @override
  Future<List<Trade>> trades(Market market, [int? limit]) async =>
      limit == null ? replayedTrades : replayedTrades.take(limit).toList();

  @override
  Future<List<Balance>> balances() async => [
    Balance(
      asset: 'krw',
      available: Decimal.parse('100000'),
      locked: Decimal.parse('2500'),
    ),
  ];

  @override
  Future<MarketStream> subscribe(
    Subscription subscription,
    StreamConfig config,
  ) async => MarketStream(
    Stream.fromIterable([
      StreamItem.event(MarketEvent.trade(replayedTrades[0])),
      const StreamItem<MarketEvent>.error(DecodeError('손상된 재생 프레임')),
      StreamItem.event(MarketEvent.trade(replayedTrades[1])),
    ]),
    onClose: () async => closeCount++,
  );
}

final class FailingNaturalCloseAdapter extends AdapterBase {
  int closeCount = 0;
  int accountCloseCount = 0;

  @override
  Exchange get exchange => Exchange.upbit;

  @override
  Set<Feature> get features => const {
    Feature.tradeStream,
    Feature.accountStream,
  };

  @override
  Future<MarketStream> subscribe(
    Subscription subscription,
    StreamConfig config,
  ) async => MarketStream(
    const Stream.empty(),
    onClose: () async {
      closeCount++;
      throw StateError('custom close failed');
    },
  );

  @override
  Future<AccountStream> subscribeAccount(StreamConfig config) async =>
      AccountStream(
        const Stream.empty(),
        onClose: () async {
          accountCloseCount++;
          throw StateError('custom account close failed');
        },
      );
}

void main() {
  setUpAll(Maxt.initialize);

  test('ReplayAdapter는 REST·비공개 조회와 비종료 오류 스트림을 제공한다', () async {
    final market = Market.spot(Exchange.upbit, 'BTC', 'KRW');
    final trades = [
      Trade(
        market: market,
        timestamp: Timestamp.fromNanoseconds(1700000000123456789),
        price: Decimal.parse('50000000.01'),
        quantity: Decimal.parse('0.001'),
        takerSide: Side.buy,
        id: 'first',
      ),
      Trade(
        market: market,
        timestamp: Timestamp.fromNanoseconds(1700000000123456790),
        price: Decimal.parse('50000000.02'),
        quantity: Decimal.parse('0.002'),
        takerSide: Side.sell,
        id: 'second',
      ),
    ];
    final adapter = ReplayAdapter(trades);
    final client = Client(adapter);

    final replayed = (await client.trades(market, 1)).single;
    expect(replayed.market, market);
    expect(replayed.timestamp, trades.first.timestamp);
    expect(replayed.price, trades.first.price);
    expect(replayed.quantity, trades.first.quantity);
    expect(replayed.takerSide, trades.first.takerSide);
    expect(replayed.id, trades.first.id);
    expect((await client.balances()).single.asset, 'KRW');

    final stream = await client.subscribe(
      Subscription(markets: [market], feeds: [Feed.trades]),
    );
    final items = await stream.toList();

    expect(items, hasLength(3));
    expect(
      ((items[0] as StreamEvent<MarketEvent>).event as TradeMarketEvent)
          .value
          .id,
      'first',
    );
    expect((items[1] as StreamError<MarketEvent>).error, isA<DecodeError>());
    expect(
      ((items[2] as StreamEvent<MarketEvent>).event as TradeMarketEvent)
          .value
          .id,
      'second',
    );

    await stream.close();
    await stream.close();
    expect(adapter.closeCount, 1);
  });

  test('custom stream 자연 종료의 close 오류를 tagged error로 전달한다', () async {
    final adapter = FailingNaturalCloseAdapter();
    final client = Client(adapter);
    final zoneErrors = <Object>[];
    List<StreamItem<MarketEvent>>? items;

    await runZonedGuarded(() async {
      final stream = await client.subscribe(
        Subscription(
          markets: [Market.spot(Exchange.upbit, 'BTC', 'KRW')],
          feeds: [Feed.trades],
        ),
      );
      items = await stream.toList();
      await Future<void>.delayed(Duration.zero);
      await stream.close();
    }, (error, _) => zoneErrors.add(error));

    expect(zoneErrors, isEmpty);
    expect(items, hasLength(1));
    expect(items?.single, isA<StreamError<MarketEvent>>());
    expect(
      (items?.single as StreamError<MarketEvent>).error,
      isA<AdapterError>(),
    );
    expect(adapter.closeCount, 1);
  });

  test('custom account stream 자연 종료의 close 오류도 tagged error로 전달한다', () async {
    final adapter = FailingNaturalCloseAdapter();
    final client = Client(adapter);
    final zoneErrors = <Object>[];
    List<StreamItem<AccountEvent>>? items;

    await runZonedGuarded(() async {
      final stream = await client.subscribeAccount();
      items = await stream.toList();
      await Future<void>.delayed(Duration.zero);
      await stream.close();
    }, (error, _) => zoneErrors.add(error));

    expect(zoneErrors, isEmpty);
    expect(items, hasLength(1));
    expect(items?.single, isA<StreamError<AccountEvent>>());
    expect(
      (items?.single as StreamError<AccountEvent>).error,
      isA<AdapterError>(),
    );
    expect(adapter.accountCloseCount, 1);
  });
}
