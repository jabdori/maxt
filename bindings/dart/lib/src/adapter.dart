import 'errors.dart';
import 'models.dart';
import 'stream.dart';

/// 사용자 정의 거래소 어댑터의 공개 계약입니다.
///
/// 일반적인 구현은 [AdapterBase]를 확장하고 [exchange]와 [features]만
/// 필수로 제공합니다. 선택 기능의 기본 메서드는 [UnsupportedError]를
/// 반환합니다.
abstract interface class Adapter {
  Exchange get exchange;

  Set<Feature> get features;

  bool supports(Feature feature);

  Future<List<MarketInfo>> markets(MarketKind kind);
  Future<List<Trade>> trades(Market market, [int? limit]);
  Future<OrderBook> orderBook(Market market, [int? depth]);
  Future<Ticker> ticker(Market market);
  Future<List<Candle>> candles(CandleRequest request);
  Future<MarketStream> subscribe(
    Subscription subscription,
    StreamConfig config,
  );
  Future<List<Balance>> balances();
  Future<List<Order>> openOrders([Market? market]);
  Future<AccountStream> subscribeAccount(StreamConfig config);
  Future<Order> placeOrder(OrderRequest request);
  Future<Order> cancelOrder(Market market, String orderId);
  Future<List<Position>> positions([Market? market]);
  Future<MarginSummary> marginSummary();
  Future<Page<FundingRate>> fundingRates(HistoryRequest request);
  Future<Page<FundingPayment>> fundingPayments(HistoryRequest request);
  Future<void> setMargin(MarginRequest request);
}

/// 선택 기능에 정직한 기본 실패 동작을 제공하는 어댑터 기반 클래스입니다.
abstract base class AdapterBase implements Adapter {
  Future<T> _unsupported<T>(Feature feature) => Future<T>.error(
    UnsupportedError(
      feature: feature,
      exchange: exchange,
      detail: '${exchange.id} has no endpoint for ${feature.wireName}',
    ),
  );

  @override
  bool supports(Feature feature) => features.contains(feature);

  @override
  Future<List<MarketInfo>> markets(MarketKind kind) =>
      _unsupported(Feature.markets);

  @override
  Future<List<Trade>> trades(Market market, [int? limit]) =>
      _unsupported(Feature.trades);

  @override
  Future<OrderBook> orderBook(Market market, [int? depth]) =>
      _unsupported(Feature.orderBook);

  @override
  Future<Ticker> ticker(Market market) => _unsupported(Feature.ticker);

  @override
  Future<List<Candle>> candles(CandleRequest request) =>
      _unsupported(Feature.candles);

  @override
  Future<MarketStream> subscribe(
    Subscription subscription,
    StreamConfig config,
  ) {
    if (subscription.markets.isEmpty) {
      return Future.error(
        const InvalidRequestError(
          field: 'markets',
          detail: 'a subscription needs at least one market',
        ),
      );
    }
    if (subscription.feeds.isEmpty) {
      return Future.error(
        const InvalidRequestError(
          field: 'feeds',
          detail: 'a subscription needs at least one feed',
        ),
      );
    }
    return _unsupported(switch (subscription.feeds.first.kind) {
      FeedKind.orderBook => Feature.orderBookStream,
      FeedKind.ticker => Feature.tickerStream,
      FeedKind.candles => Feature.candleStream,
      FeedKind.trades => Feature.tradeStream,
    });
  }

  @override
  Future<List<Balance>> balances() => _unsupported(Feature.balances);

  @override
  Future<List<Order>> openOrders([Market? market]) =>
      _unsupported(Feature.openOrders);

  @override
  Future<AccountStream> subscribeAccount(StreamConfig config) =>
      _unsupported(Feature.accountStream);

  @override
  Future<Order> placeOrder(OrderRequest request) =>
      _unsupported(Feature.trading);

  @override
  Future<Order> cancelOrder(Market market, String orderId) =>
      _unsupported(Feature.trading);

  @override
  Future<List<Position>> positions([Market? market]) =>
      _unsupported(Feature.positions);

  @override
  Future<MarginSummary> marginSummary() => _unsupported(Feature.margin);

  @override
  Future<Page<FundingRate>> fundingRates(HistoryRequest request) =>
      _unsupported(Feature.fundingRates);

  @override
  Future<Page<FundingPayment>> fundingPayments(HistoryRequest request) =>
      _unsupported(Feature.fundingPayments);

  @override
  Future<void> setMargin(MarginRequest request) =>
      _unsupported(Feature.marginConfig);
}
