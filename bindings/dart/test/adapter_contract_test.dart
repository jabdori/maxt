import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

final class MinimalAdapter extends AdapterBase {
  @override
  Exchange get exchange => Exchange.upbit;

  @override
  Set<Feature> get features => const {};
}

void main() {
  test('모든 선택 Adapter 메서드는 해당 기능의 UnsupportedError를 반환한다', () async {
    final adapter = MinimalAdapter();
    final market = Market.spot(Exchange.upbit, 'BTC', 'KRW');
    final decimal = Decimal.parse('1');
    final subscription = Subscription(markets: [market], feeds: [Feed.trades]);
    final order = OrderRequest.market(market, Side.buy, Size.quote(decimal));
    final history = HistoryRequest(market);
    final calls = <(Feature, Future<Object?> Function())>[
      (Feature.markets, () => adapter.markets(MarketKind.spot)),
      (Feature.trades, () => adapter.trades(market)),
      (Feature.orderBook, () => adapter.orderBook(market)),
      (Feature.ticker, () => adapter.ticker(market)),
      (
        Feature.candles,
        () => adapter.candles(CandleRequest(market, Interval.min1)),
      ),
      (
        Feature.tradeStream,
        () => adapter.subscribe(subscription, const StreamConfig()),
      ),
      (Feature.balances, adapter.balances),
      (Feature.openOrders, adapter.openOrders),
      (
        Feature.accountStream,
        () => adapter.subscribeAccount(const StreamConfig()),
      ),
      (Feature.trading, () => adapter.placeOrder(order)),
      (Feature.trading, () => adapter.cancelOrder(market, 'order-id')),
      (Feature.positions, adapter.positions),
      (Feature.margin, adapter.marginSummary),
      (Feature.fundingRates, () => adapter.fundingRates(history)),
      (Feature.fundingPayments, () => adapter.fundingPayments(history)),
      (
        Feature.marginConfig,
        () => adapter.setMargin(MarginRequest(market, leverage: decimal)),
      ),
    ];

    for (final (feature, call) in calls) {
      await expectLater(
        call(),
        throwsA(
          isA<UnsupportedError>().having(
            (error) => error.message,
            'message',
            'upbit has no endpoint for ${feature.wireName}',
          ),
        ),
        reason: feature.name,
      );
    }
  });

  test('subscribe 기본 오류는 첫 피드에 대응하는 스트림 기능을 지목한다', () async {
    final adapter = MinimalAdapter();
    final market = Market.spot(Exchange.upbit, 'BTC', 'KRW');
    final cases = <(Feed, Feature)>[
      (Feed.trades, Feature.tradeStream),
      (Feed.orderBook, Feature.orderBookStream),
      (Feed.ticker, Feature.tickerStream),
      (const Feed.candles(Interval.min1), Feature.candleStream),
    ];

    for (final (feed, feature) in cases) {
      final subscription = Subscription(markets: [market], feeds: [feed]);
      await expectLater(
        adapter.subscribe(subscription, const StreamConfig()),
        throwsA(
          isA<UnsupportedError>().having(
            (error) => error.message,
            'message',
            'upbit has no endpoint for ${feature.wireName}',
          ),
        ),
      );
    }
  });

  test('subscribe는 빈 시장과 빈 피드를 구조화된 요청 오류로 거절한다', () async {
    final adapter = MinimalAdapter();
    final market = Market.spot(Exchange.upbit, 'BTC', 'KRW');

    await expectLater(
      adapter.subscribe(
        Subscription(feeds: [Feed.trades]),
        const StreamConfig(),
      ),
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
