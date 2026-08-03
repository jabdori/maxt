import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

final class FullContractAdapter extends AdapterBase {
  FullContractAdapter(this.market, this.timestamp) {
    order = Order(
      id: 'order-1',
      market: market,
      side: Side.buy,
      status: OrderStatus.open,
      filledQuantity: Decimal.parse('0.125'),
      remainingQuantity: Decimal.parse('0.375'),
      price: Decimal.parse('123.4500'),
      createdAt: timestamp,
    );
  }

  final Market market;
  final Timestamp timestamp;
  late final Order order;

  MarketKind? requestedKind;
  int? orderBookDepth;
  CandleRequest? candleRequest;
  Market? openOrdersMarket;
  StreamConfig? accountConfig;
  OrderRequest? placedRequest;
  (Market, String)? cancelledOrder;
  Market? positionsMarket;
  HistoryRequest? rateRequest;
  HistoryRequest? paymentRequest;
  MarginRequest? marginRequest;
  int accountCloseCount = 0;

  @override
  Exchange get exchange => Exchange.binance;

  @override
  Set<Feature> get features => const {
    Feature.markets,
    Feature.orderBook,
    Feature.ticker,
    Feature.candles,
    Feature.balances,
    Feature.openOrders,
    Feature.accountStream,
    Feature.trading,
    Feature.positions,
    Feature.margin,
    Feature.fundingRates,
    Feature.fundingPayments,
    Feature.marginConfig,
    Feature.reduceOnlyOrders,
  };

  @override
  Future<List<MarketInfo>> markets(MarketKind kind) async {
    requestedKind = kind;
    return [
      MarketInfo(
        market: market,
        nativeSymbol: 'BTCUSDT',
        status: MarketStatus.active,
        koreanName: '비트코인',
        englishName: 'Bitcoin',
      ),
    ];
  }

  @override
  Future<OrderBook> orderBook(Market market, [int? depth]) async {
    orderBookDepth = depth;
    return OrderBook(
      market: this.market,
      timestamp: timestamp,
      bids: [
        Level(price: Decimal.parse('123.45'), quantity: Decimal.parse('1.25')),
      ],
      asks: [
        Level(price: Decimal.parse('123.46'), quantity: Decimal.parse('2.50')),
      ],
    );
  }

  @override
  Future<Ticker> ticker(Market market) async => Ticker(
    market: this.market,
    timestamp: timestamp,
    lastTradeTime: Timestamp.fromNanoseconds(
      timestamp.nanosecondsSinceEpoch - BigInt.one,
    ),
    lastPrice: Decimal.parse('123.455'),
    change: Decimal.parse('-0.005'),
    changeRate: Decimal.parse('-0.00004'),
    high: Decimal.parse('130'),
    low: Decimal.parse('120'),
    volume: Decimal.parse('42.125'),
    quoteVolume: Decimal.parse('5200.0001'),
  );

  @override
  Future<List<Candle>> candles(CandleRequest request) async {
    candleRequest = request;
    return [
      Candle(
        market: market,
        interval: Interval.min1,
        openTime: timestamp,
        open: Decimal.parse('123.40'),
        high: Decimal.parse('124.00'),
        low: Decimal.parse('123.00'),
        close: Decimal.parse('123.45'),
        volume: Decimal.parse('10.50'),
        quoteVolume: Decimal.parse('1296.225'),
        closed: true,
      ),
    ];
  }

  @override
  Future<List<Balance>> balances() async => [
    Balance(
      asset: 'usdt',
      available: Decimal.parse('1000.00000001'),
      locked: Decimal.parse('2.5'),
    ),
  ];

  @override
  Future<List<Order>> openOrders([Market? market]) async {
    openOrdersMarket = market;
    return [order];
  }

  @override
  Future<AccountStream> subscribeAccount(StreamConfig config) async {
    accountConfig = config;
    return AccountStream(
      Stream.fromIterable([
        StreamItem.event(
          AccountEvent.balance(
            Balance(
              asset: 'usdt',
              available: Decimal.parse('999.5'),
              locked: Decimal.parse('3.0'),
            ),
          ),
        ),
        const StreamItem<AccountEvent>.error(DecodeError('손상된 계정 프레임')),
        StreamItem.event(AccountEvent.order(order)),
      ]),
      onClose: () async => accountCloseCount++,
    );
  }

  @override
  Future<Order> placeOrder(OrderRequest request) async {
    placedRequest = request;
    return order;
  }

  @override
  Future<Order> cancelOrder(Market market, String orderId) async {
    cancelledOrder = (market, orderId);
    return order;
  }

  @override
  Future<List<Position>> positions([Market? market]) async {
    positionsMarket = market;
    return [
      Position(
        market: this.market,
        side: Side.buy,
        quantity: Decimal.parse('0.500'),
        entryPrice: Decimal.parse('120.00'),
        markPrice: Decimal.parse('123.45'),
        notional: Decimal.parse('61.725'),
        unrealizedPnl: Decimal.parse('1.725'),
        leverage: Decimal.parse('5'),
        marginMode: MarginMode.isolated,
      ),
    ];
  }

  @override
  Future<MarginSummary> marginSummary() async => MarginSummary(
    asset: 'usdt',
    equity: Decimal.parse('1001.725'),
    marginBalance: Decimal.parse('1000.5'),
    availableBalance: Decimal.parse('900.25'),
  );

  @override
  Future<Page<FundingRate>> fundingRates(HistoryRequest request) async {
    rateRequest = request;
    return Page(
      items: [
        FundingRate(
          market: market,
          timestamp: timestamp,
          rate: Decimal.parse('0.000125'),
          markPrice: Decimal.parse('123.45'),
        ),
      ],
      next: const Cursor('rate-next'),
    );
  }

  @override
  Future<Page<FundingPayment>> fundingPayments(HistoryRequest request) async {
    paymentRequest = request;
    return Page(
      items: [
        FundingPayment(
          market: market,
          timestamp: timestamp,
          amount: Decimal.parse('-0.01234567'),
          rate: Decimal.parse('0.000125'),
          id: 'funding-1',
        ),
      ],
      next: const Cursor('payment-next'),
    );
  }

  @override
  Future<void> setMargin(MarginRequest request) async {
    marginRequest = request;
  }
}

void main() {
  setUpAll(Maxt.initialize);
  tearDownAll(Maxt.dispose);

  test('Dart Adapter의 공개·비공개·주문·증거금·이력을 Rust Client로 왕복한다', () async {
    final market = Market.perpetual(Exchange.binance, 'BTC', 'USDT');
    final timestamp = Timestamp.fromNanoseconds(
      BigInt.parse("1700000000123456789"),
    );
    final adapter = FullContractAdapter(market, timestamp);
    final client = Client(adapter);

    final marketInfo = (await client.markets(MarketKind.perpetual)).single;
    expect(marketInfo.market, market);
    expect(marketInfo.nativeSymbol, 'BTCUSDT');
    expect(marketInfo.koreanName, '비트코인');
    expect(adapter.requestedKind, MarketKind.perpetual);

    final book = await client.orderBook(market, 7);
    expect(book.timestamp, timestamp);
    expect(book.bestBid?.price, Decimal.parse('123.45'));
    expect(book.bestAsk?.quantity, Decimal.parse('2.50'));
    expect(adapter.orderBookDepth, 7);

    final ticker = await client.ticker(market);
    expect(
      ticker.lastTradeTime?.nanosecondsSinceEpoch,
      BigInt.parse('1700000000123456788'),
    );
    expect(ticker.change, Decimal.parse('-0.005'));
    expect(ticker.quoteVolume, Decimal.parse('5200.0001'));

    final candleQuery = CandleRequest(
      market,
      Interval.min1,
      from: Timestamp.fromNanoseconds(BigInt.parse("1700000000000000000")),
      to: timestamp,
      limit: 3,
    );
    final candle = (await client.candles(candleQuery)).single;
    expect(candle.openTime, timestamp);
    expect(candle.quoteVolume, Decimal.parse('1296.225'));
    expect(candle.closed, isTrue);
    expect(adapter.candleRequest?.from, candleQuery.from);
    expect(adapter.candleRequest?.to, candleQuery.to);
    expect(adapter.candleRequest?.limit, 3);

    final balance = (await client.balances()).single;
    expect(balance.asset, 'USDT');
    expect(balance.available, Decimal.parse('1000.00000001'));

    final openOrder = (await client.openOrdersOn(market)).single;
    expect(openOrder.id, 'order-1');
    expect(openOrder.createdAt, timestamp);
    expect(openOrder.price, Decimal.parse('123.4500'));
    expect(adapter.openOrdersMarket, market);

    final orderRequest = OrderRequest.limit(
      market,
      Side.buy,
      Size.base(Decimal.parse('0.50')),
      Decimal.parse('123.4500'),
    ).withTimeInForce(TimeInForce.postOnly).asReduceOnly();
    expect((await client.placeOrder(orderRequest)).id, 'order-1');
    expect(adapter.placedRequest?.market, market);
    expect(adapter.placedRequest?.size, isA<BaseSize>());
    expect(adapter.placedRequest?.size.value, Decimal.parse('0.50'));
    expect(adapter.placedRequest?.price, Decimal.parse('123.4500'));
    expect(adapter.placedRequest?.timeInForce, TimeInForce.postOnly);
    expect(adapter.placedRequest?.reduceOnly, isTrue);

    expect((await client.cancelOrder(market, 'order-1')).id, 'order-1');
    expect(adapter.cancelledOrder, (market, 'order-1'));

    final position = (await client.positionsOn(market)).single;
    expect(position.quantity, Decimal.parse('0.500'));
    expect(position.unrealizedPnl, Decimal.parse('1.725'));
    expect(position.marginMode, MarginMode.isolated);
    expect(adapter.positionsMarket, market);

    final margin = await client.marginSummary();
    expect(margin.asset, 'USDT');
    expect(margin.equity, Decimal.parse('1001.725'));

    final history = HistoryRequest(
      market,
      from: Timestamp.fromNanoseconds(BigInt.parse("1700000000000000000")),
      to: timestamp,
      cursor: const Cursor('cursor-in'),
      limit: 2,
    );
    final rates = await client.fundingRates(history);
    final payments = await client.fundingPayments(history);
    expect(rates.items.single.rate, Decimal.parse('0.000125'));
    expect(rates.next, const Cursor('rate-next'));
    expect(payments.items.single.amount, Decimal.parse('-0.01234567'));
    expect(payments.next, const Cursor('payment-next'));
    expect(adapter.rateRequest?.cursor, const Cursor('cursor-in'));
    expect(adapter.paymentRequest?.from, history.from);

    final marginRequest = MarginRequest(
      market,
      leverage: Decimal.parse('5'),
      marginMode: MarginMode.isolated,
    );
    await client.setMargin(marginRequest);
    expect(adapter.marginRequest?.leverage, Decimal.parse('5'));
    expect(adapter.marginRequest?.marginMode, MarginMode.isolated);

    final account = await client.subscribeAccountWith(
      const StreamConfig(bufferSize: 16),
    );
    final accountItems = await account.toList();
    expect(accountItems, hasLength(3));
    expect(
      (accountItems[0] as StreamEvent<AccountEvent>).event,
      isA<BalanceAccountEvent>(),
    );
    expect(
      (accountItems[1] as StreamError<AccountEvent>).error,
      isA<DecodeError>(),
    );
    expect(
      (accountItems[2] as StreamEvent<AccountEvent>).event,
      isA<OrderAccountEvent>(),
    );
    await account.close();
    expect(adapter.accountConfig, const StreamConfig(bufferSize: 16));
    expect(adapter.accountCloseCount, 1);
  });
}
