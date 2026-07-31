import 'adapter.dart';
import 'adapters.dart' show NativeClientDelegate;
import 'models.dart';
import 'stream.dart';

/// 하나의 거래소 어댑터를 공통 API로 노출합니다.
///
/// 생성하기 전에 `await Maxt.initialize()`로 native 런타임을
/// 초기화해야 합니다.
final class Client<A extends Adapter> {
  Client(this.adapter) : _native = NativeClientDelegate.fromAdapter(adapter);

  /// 공급자 전용 기능에 접근할 때 쓰는 원래 어댑터 객체입니다.
  final A adapter;
  final NativeClientDelegate _native;

  Exchange get exchange => _native.exchange;

  bool supports(Feature feature) => _native.supports(feature);

  Future<List<MarketInfo>> markets(MarketKind kind) => _native.markets(kind);

  Future<List<Trade>> trades(Market market, [int? limit]) =>
      _native.trades(market, limit);

  Future<OrderBook> orderBook(Market market, [int? depth]) =>
      _native.orderBook(market, depth);

  Future<Ticker> ticker(Market market) => _native.ticker(market);

  Future<List<Candle>> candles(CandleRequest request) =>
      _native.candles(request);

  Future<MarketStream> subscribe(Subscription subscription) =>
      subscribeWith(subscription, const StreamConfig());

  Future<MarketStream> subscribeWith(
    Subscription subscription,
    StreamConfig config,
  ) => _native.subscribe(subscription, config);

  Future<List<Balance>> balances() => _native.balances();

  Future<List<Order>> openOrders() => _native.openOrders();

  Future<List<Order>> openOrdersOn(Market market) => _native.openOrders(market);

  Future<AccountStream> subscribeAccount() =>
      subscribeAccountWith(const StreamConfig());

  Future<AccountStream> subscribeAccountWith(StreamConfig config) =>
      _native.subscribeAccount(config);

  Future<Order> placeOrder(OrderRequest request) => _native.placeOrder(request);

  Future<Order> cancelOrder(Market market, String orderId) =>
      _native.cancelOrder(market, orderId);

  Future<List<Position>> positions() async =>
      _openPositions(await _native.positions());

  Future<List<Position>> positionsOn(Market market) async =>
      _openPositions(await _native.positions(market));

  Future<MarginSummary> marginSummary() => _native.marginSummary();

  Future<Page<FundingRate>> fundingRates(HistoryRequest request) =>
      _native.fundingRates(request);

  Future<Page<FundingPayment>> fundingPayments(HistoryRequest request) =>
      _native.fundingPayments(request);

  Future<void> setMargin(MarginRequest request) => _native.setMargin(request);

  static List<Position> _openPositions(List<Position> positions) =>
      positions.where((position) => !position.isFlat).toList(growable: false);
}
