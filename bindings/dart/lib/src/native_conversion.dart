part of 'adapters.dart';

T _enumByName<T extends Enum>(List<T> values, Enum value) =>
    values.byName(value.name);

wire.WireExchange _exchangeToWire(Exchange value) =>
    _enumByName(wire.WireExchange.values, value);
Exchange _exchangeFromWire(wire.WireExchange value) =>
    _enumByName(Exchange.values, value);
wire.WireFeature _featureToWire(Feature value) =>
    _enumByName(wire.WireFeature.values, value);
Feature _featureFromWire(wire.WireFeature value) =>
    _enumByName(Feature.values, value);
wire.WireMarketKind _marketKindToWire(MarketKind value) =>
    _enumByName(wire.WireMarketKind.values, value);
MarketKind _marketKindFromWire(wire.WireMarketKind value) =>
    _enumByName(MarketKind.values, value);
MarketStatus _marketStatusFromWire(wire.WireMarketStatus value) =>
    _enumByName(MarketStatus.values, value);
wire.WireMarketStatus _marketStatusToWire(MarketStatus value) =>
    _enumByName(wire.WireMarketStatus.values, value);
Side _sideFromWire(wire.WireSide value) => _enumByName(Side.values, value);
wire.WireSide _sideToWire(Side value) =>
    _enumByName(wire.WireSide.values, value);
Interval _intervalFromWire(wire.WireInterval value) =>
    _enumByName(Interval.values, value);
wire.WireInterval _intervalToWire(Interval value) =>
    _enumByName(wire.WireInterval.values, value);
OrderStatus _orderStatusFromWire(wire.WireOrderStatus value) =>
    _enumByName(OrderStatus.values, value);
wire.WireOrderStatus _orderStatusToWire(OrderStatus value) =>
    _enumByName(wire.WireOrderStatus.values, value);
OrderType _orderTypeFromWire(wire.WireOrderType value) =>
    _enumByName(OrderType.values, value);
wire.WireOrderType _orderTypeToWire(OrderType value) =>
    _enumByName(wire.WireOrderType.values, value);
TimeInForce _timeInForceFromWire(wire.WireTimeInForce value) =>
    _enumByName(TimeInForce.values, value);
wire.WireTimeInForce _timeInForceToWire(TimeInForce value) =>
    _enumByName(wire.WireTimeInForce.values, value);
MarginMode _marginModeFromWire(wire.WireMarginMode value) =>
    _enumByName(MarginMode.values, value);
wire.WireMarginMode _marginModeToWire(MarginMode value) =>
    _enumByName(wire.WireMarginMode.values, value);
native.WireUpbitRegion _upbitRegionToWire(UpbitRegion value) =>
    _enumByName(native.WireUpbitRegion.values, value);

native_adapter.WireFeed _feedToWire(Feed value) => switch (value.kind) {
  FeedKind.trades => const native_adapter.WireFeed.trades(),
  FeedKind.orderBook => const native_adapter.WireFeed.orderBook(),
  FeedKind.ticker => const native_adapter.WireFeed.ticker(),
  FeedKind.candles => native_adapter.WireFeed.candles(
    _intervalToWire(value.interval!),
  ),
};

Feed _feedFromWire(native_adapter.WireFeed value) => switch (value) {
  native_adapter.WireFeed_Trades() => Feed.trades,
  native_adapter.WireFeed_OrderBook() => Feed.orderBook,
  native_adapter.WireFeed_Ticker() => Feed.ticker,
  native_adapter.WireFeed_Candles(:final field0) => Feed.candles(
    _intervalFromWire(field0),
  ),
};

native_adapter.WireSubscription _subscriptionToWire(Subscription value) =>
    native_adapter.WireSubscription(
      markets: value.markets.map(_marketToWire).toList(growable: false),
      feeds: value.feeds.map(_feedToWire).toList(growable: false),
    );

Subscription _subscriptionFromWire(native_adapter.WireSubscription value) =>
    Subscription(
      markets: value.markets.map(_marketFromWire),
      feeds: value.feeds.map(_feedFromWire),
    );

native_adapter.WireStreamConfig _streamConfigToWire(StreamConfig value) {
  final maxReconnectAttempts = value.maxReconnectAttempts;
  if (maxReconnectAttempts != null) {
    _validateUnsigned(
      maxReconnectAttempts,
      field: 'maxReconnectAttempts',
      max: _uint32Max,
    );
  }
  _validateUnsigned(
    value.initialReconnectDelayMs,
    field: 'initialReconnectDelayMs',
    max: _uint64Max,
  );
  _validateUnsigned(
    value.maxReconnectDelayMs,
    field: 'maxReconnectDelayMs',
    max: _uint64Max,
  );
  _validateUnsigned(
    value.idleTimeoutMs,
    field: 'idleTimeoutMs',
    max: _uint64Max,
  );
  _validateUnsigned(value.bufferSize, field: 'bufferSize', max: _uint32Max);
  return native_adapter.WireStreamConfig(
    maxReconnectAttempts: maxReconnectAttempts,
    initialReconnectDelayMs: BigInt.from(value.initialReconnectDelayMs),
    maxReconnectDelayMs: BigInt.from(value.maxReconnectDelayMs),
    idleTimeoutMs: BigInt.from(value.idleTimeoutMs),
    bufferSize: BigInt.from(value.bufferSize),
    overflow: _enumByName(native_adapter.WireOverflow.values, value.overflow),
  );
}

final BigInt _uint32Max = BigInt.parse('4294967295');
final BigInt _uint64Max = BigInt.parse('18446744073709551615');

void _validateUnsigned(
  int value, {
  required String field,
  required BigInt max,
}) {
  if (value < 0) {
    throw InvalidRequestError(field: field, detail: 'must not be negative');
  }
  if (BigInt.from(value) > max) {
    throw InvalidRequestError(
      field: field,
      detail: 'is outside the supported unsigned integer range',
    );
  }
}

int? _checkedUint32(int? value, {required String field}) {
  if (value != null) {
    _validateUnsigned(value, field: field, max: _uint32Max);
  }
  return value;
}

StreamConfig _streamConfigFromWire(native_adapter.WireStreamConfig value) =>
    StreamConfig(
      maxReconnectAttempts: value.maxReconnectAttempts,
      initialReconnectDelayMs: value.initialReconnectDelayMs.toInt(),
      maxReconnectDelayMs: value.maxReconnectDelayMs.toInt(),
      idleTimeoutMs: value.idleTimeoutMs.toInt(),
      bufferSize: value.bufferSize.toInt(),
      overflow: _enumByName(Overflow.values, value.overflow),
    );

wire.WireMarket _marketToWire(Market value) => wire.WireMarket(
  exchange: _exchangeToWire(value.exchange),
  kind: _marketKindToWire(value.kind),
  base: value.base,
  quote: value.quote,
);

Market _marketFromWire(wire.WireMarket value) => Market(
  _exchangeFromWire(value.exchange),
  _marketKindFromWire(value.kind),
  value.base,
  value.quote,
);

Decimal? _decimalFromWire(String? value) =>
    value == null ? null : Decimal.parse(value);
Timestamp? _timestampFromWire(int? value) =>
    value == null ? null : Timestamp.fromNanoseconds(value);

wire.WireCandleRequest _candleRequestToWire(CandleRequest value) =>
    wire.WireCandleRequest(
      market: _marketToWire(value.market),
      interval: _intervalToWire(value.interval),
      fromNs: value.from?.nanosecondsSinceEpoch,
      toNs: value.to?.nanosecondsSinceEpoch,
      limit: _checkedUint32(value.limit, field: 'limit'),
    );

CandleRequest _candleRequestFromWire(wire.WireCandleRequest value) =>
    CandleRequest(
      _marketFromWire(value.market),
      _intervalFromWire(value.interval),
      from: _timestampFromWire(value.fromNs),
      to: _timestampFromWire(value.toNs),
      limit: value.limit,
    );

wire.WireHistoryRequest _historyRequestToWire(HistoryRequest value) =>
    wire.WireHistoryRequest(
      market: _marketToWire(value.market),
      fromNs: value.from?.nanosecondsSinceEpoch,
      toNs: value.to?.nanosecondsSinceEpoch,
      cursor: value.cursor?.value,
      limit: _checkedUint32(value.limit, field: 'limit'),
    );

HistoryRequest _historyRequestFromWire(wire.WireHistoryRequest value) =>
    HistoryRequest(
      _marketFromWire(value.market),
      from: _timestampFromWire(value.fromNs),
      to: _timestampFromWire(value.toNs),
      cursor: value.cursor == null ? null : Cursor(value.cursor!),
      limit: value.limit,
    );

wire.WireOrderRequest _orderRequestToWire(OrderRequest value) =>
    wire.WireOrderRequest(
      market: _marketToWire(value.market),
      side: _sideToWire(value.side),
      orderType: _orderTypeToWire(value.orderType),
      size: wire.WireSize(
        kind: value.size is BaseSize
            ? wire.WireSizeKind.base
            : wire.WireSizeKind.quote,
        value: value.size.value.toString(),
      ),
      price: value.price?.toString(),
      timeInForce: value.timeInForce == null
          ? null
          : _timeInForceToWire(value.timeInForce!),
      reduceOnly: value.reduceOnly,
    );

OrderRequest _orderRequestFromWire(wire.WireOrderRequest value) {
  final market = _marketFromWire(value.market);
  final side = _sideFromWire(value.side);
  final sizeValue = Decimal.parse(value.size.value);
  final size = switch (value.size.kind) {
    wire.WireSizeKind.base => Size.base(sizeValue),
    wire.WireSizeKind.quote => Size.quote(sizeValue),
  };
  var request = switch (_orderTypeFromWire(value.orderType)) {
    OrderType.market => OrderRequest.market(market, side, size),
    OrderType.limit => OrderRequest.limit(
      market,
      side,
      size,
      Decimal.parse(
        value.price ?? (throw StateError('native limit order has no price')),
      ),
    ),
  };
  if (value.timeInForce != null) {
    request = request.withTimeInForce(_timeInForceFromWire(value.timeInForce!));
  }
  return value.reduceOnly ? request.asReduceOnly() : request;
}

wire.WireMarginRequest _marginRequestToWire(MarginRequest value) =>
    wire.WireMarginRequest(
      market: _marketToWire(value.market),
      leverage: value.leverage?.toString(),
      marginMode: value.marginMode == null
          ? null
          : _marginModeToWire(value.marginMode!),
    );

MarginRequest _marginRequestFromWire(wire.WireMarginRequest value) =>
    MarginRequest(
      _marketFromWire(value.market),
      leverage: _decimalFromWire(value.leverage),
      marginMode: value.marginMode == null
          ? null
          : _marginModeFromWire(value.marginMode!),
    );

MarketInfo _marketInfoFromWire(wire.WireMarketInfo value) => MarketInfo(
  market: _marketFromWire(value.market),
  nativeSymbol: value.nativeSymbol,
  status: _marketStatusFromWire(value.status),
  koreanName: value.koreanName,
  englishName: value.englishName,
);

wire.WireMarketInfo _marketInfoToWire(MarketInfo value) => wire.WireMarketInfo(
  market: _marketToWire(value.market),
  nativeSymbol: value.nativeSymbol,
  status: _marketStatusToWire(value.status),
  koreanName: value.koreanName,
  englishName: value.englishName,
);

Trade _tradeFromWire(wire.WireTrade value) => Trade(
  market: _marketFromWire(value.market),
  timestamp: Timestamp.fromNanoseconds(value.timestampNs),
  price: Decimal.parse(value.price),
  quantity: Decimal.parse(value.quantity),
  takerSide: _sideFromWire(value.takerSide),
  id: value.id,
);

wire.WireTrade _tradeToWire(Trade value) => wire.WireTrade(
  market: _marketToWire(value.market),
  timestampNs: value.timestamp.nanosecondsSinceEpoch,
  price: value.price.toString(),
  quantity: value.quantity.toString(),
  takerSide: _sideToWire(value.takerSide),
  id: value.id,
);

Level _levelFromWire(wire.WireLevel value) => Level(
  price: Decimal.parse(value.price),
  quantity: Decimal.parse(value.quantity),
);

wire.WireLevel _levelToWire(Level value) => wire.WireLevel(
  price: value.price.toString(),
  quantity: value.quantity.toString(),
);

OrderBook _orderBookFromWire(wire.WireOrderBook value) => OrderBook(
  market: _marketFromWire(value.market),
  timestamp: Timestamp.fromNanoseconds(value.timestampNs),
  bids: value.bids.map(_levelFromWire),
  asks: value.asks.map(_levelFromWire),
);

wire.WireOrderBook _orderBookToWire(OrderBook value) => wire.WireOrderBook(
  market: _marketToWire(value.market),
  timestampNs: value.timestamp.nanosecondsSinceEpoch,
  bids: value.bids.map(_levelToWire).toList(growable: false),
  asks: value.asks.map(_levelToWire).toList(growable: false),
);

Ticker _tickerFromWire(wire.WireTicker value) => Ticker(
  market: _marketFromWire(value.market),
  timestamp: Timestamp.fromNanoseconds(value.timestampNs),
  lastTradeTime: _timestampFromWire(value.lastTradeTimeNs),
  lastPrice: Decimal.parse(value.lastPrice),
  change: _decimalFromWire(value.change),
  changeRate: _decimalFromWire(value.changeRate),
  high: _decimalFromWire(value.high),
  low: _decimalFromWire(value.low),
  volume: _decimalFromWire(value.volume),
  quoteVolume: _decimalFromWire(value.quoteVolume),
);

wire.WireTicker _tickerToWire(Ticker value) => wire.WireTicker(
  market: _marketToWire(value.market),
  timestampNs: value.timestamp.nanosecondsSinceEpoch,
  lastTradeTimeNs: value.lastTradeTime?.nanosecondsSinceEpoch,
  lastPrice: value.lastPrice.toString(),
  change: value.change?.toString(),
  changeRate: value.changeRate?.toString(),
  high: value.high?.toString(),
  low: value.low?.toString(),
  volume: value.volume?.toString(),
  quoteVolume: value.quoteVolume?.toString(),
);

Candle _candleFromWire(wire.WireCandle value) => Candle(
  market: _marketFromWire(value.market),
  interval: _intervalFromWire(value.interval),
  openTime: Timestamp.fromNanoseconds(value.openTimeNs),
  open: Decimal.parse(value.open),
  high: Decimal.parse(value.high),
  low: Decimal.parse(value.low),
  close: Decimal.parse(value.close),
  volume: Decimal.parse(value.volume),
  quoteVolume: _decimalFromWire(value.quoteVolume),
  closed: value.closed,
);

wire.WireCandle _candleToWire(Candle value) => wire.WireCandle(
  market: _marketToWire(value.market),
  interval: _intervalToWire(value.interval),
  openTimeNs: value.openTime.nanosecondsSinceEpoch,
  open: value.open.toString(),
  high: value.high.toString(),
  low: value.low.toString(),
  close: value.close.toString(),
  volume: value.volume.toString(),
  quoteVolume: value.quoteVolume?.toString(),
  closed: value.closed,
);

Balance _balanceFromWire(wire.WireBalance value) => Balance(
  asset: value.asset,
  available: Decimal.parse(value.available),
  locked: Decimal.parse(value.locked),
);

wire.WireBalance _balanceToWire(Balance value) => wire.WireBalance(
  asset: value.asset,
  available: value.available.toString(),
  locked: value.locked.toString(),
);

Order _orderFromWire(wire.WireOrder value) => Order(
  id: value.id,
  market: _marketFromWire(value.market),
  side: _sideFromWire(value.side),
  status: _orderStatusFromWire(value.status),
  filledQuantity: Decimal.parse(value.filledQuantity),
  remainingQuantity: Decimal.parse(value.remainingQuantity),
  price: _decimalFromWire(value.price),
  createdAt: _timestampFromWire(value.createdAtNs),
);

wire.WireOrder _orderToWire(Order value) => wire.WireOrder(
  id: value.id,
  market: _marketToWire(value.market),
  side: _sideToWire(value.side),
  status: _orderStatusToWire(value.status),
  filledQuantity: value.filledQuantity.toString(),
  remainingQuantity: value.remainingQuantity.toString(),
  price: value.price?.toString(),
  createdAtNs: value.createdAt?.nanosecondsSinceEpoch,
);

Position _positionFromWire(wire.WirePosition value) => Position(
  market: _marketFromWire(value.market),
  side: value.side == null ? null : _sideFromWire(value.side!),
  quantity: Decimal.parse(value.quantity),
  entryPrice: _decimalFromWire(value.entryPrice),
  markPrice: _decimalFromWire(value.markPrice),
  notional: _decimalFromWire(value.notional),
  unrealizedPnl: _decimalFromWire(value.unrealizedPnl),
  leverage: _decimalFromWire(value.leverage),
  marginMode: value.marginMode == null
      ? null
      : _marginModeFromWire(value.marginMode!),
);

wire.WirePosition _positionToWire(Position value) => wire.WirePosition(
  market: _marketToWire(value.market),
  side: value.side == null ? null : _sideToWire(value.side!),
  quantity: value.quantity.toString(),
  entryPrice: value.entryPrice?.toString(),
  markPrice: value.markPrice?.toString(),
  notional: value.notional?.toString(),
  unrealizedPnl: value.unrealizedPnl?.toString(),
  leverage: value.leverage?.toString(),
  marginMode: value.marginMode == null
      ? null
      : _marginModeToWire(value.marginMode!),
);

MarginSummary _marginSummaryFromWire(wire.WireMarginSummary value) =>
    MarginSummary(
      asset: value.asset,
      equity: _decimalFromWire(value.equity),
      marginBalance: _decimalFromWire(value.marginBalance),
      availableBalance: _decimalFromWire(value.availableBalance),
    );

wire.WireMarginSummary _marginSummaryToWire(MarginSummary value) =>
    wire.WireMarginSummary(
      asset: value.asset,
      equity: value.equity?.toString(),
      marginBalance: value.marginBalance?.toString(),
      availableBalance: value.availableBalance?.toString(),
    );

FundingRate _fundingRateFromWire(wire.WireFundingRate value) => FundingRate(
  market: _marketFromWire(value.market),
  timestamp: Timestamp.fromNanoseconds(value.timestampNs),
  rate: Decimal.parse(value.rate),
  markPrice: _decimalFromWire(value.markPrice),
);

wire.WireFundingRate _fundingRateToWire(FundingRate value) =>
    wire.WireFundingRate(
      market: _marketToWire(value.market),
      timestampNs: value.timestamp.nanosecondsSinceEpoch,
      rate: value.rate.toString(),
      markPrice: value.markPrice?.toString(),
    );

FundingPayment _fundingPaymentFromWire(wire.WireFundingPayment value) =>
    FundingPayment(
      market: _marketFromWire(value.market),
      timestamp: Timestamp.fromNanoseconds(value.timestampNs),
      amount: Decimal.parse(value.amount),
      rate: _decimalFromWire(value.rate),
      id: value.id,
    );

wire.WireFundingPayment _fundingPaymentToWire(FundingPayment value) =>
    wire.WireFundingPayment(
      market: _marketToWire(value.market),
      timestampNs: value.timestamp.nanosecondsSinceEpoch,
      amount: value.amount.toString(),
      rate: value.rate?.toString(),
      id: value.id,
    );

Page<FundingRate> _fundingRatePageFromWire(wire.WireFundingRatePage value) =>
    Page(
      items: value.items.map(_fundingRateFromWire),
      next: value.next == null ? null : Cursor(value.next!),
    );

wire.WireFundingRatePage _fundingRatePageToWire(Page<FundingRate> value) =>
    wire.WireFundingRatePage(
      items: value.items.map(_fundingRateToWire).toList(growable: false),
      next: value.next?.value,
    );

Page<FundingPayment> _fundingPaymentPageFromWire(
  wire.WireFundingPaymentPage value,
) => Page(
  items: value.items.map(_fundingPaymentFromWire),
  next: value.next == null ? null : Cursor(value.next!),
);

wire.WireFundingPaymentPage _fundingPaymentPageToWire(
  Page<FundingPayment> value,
) => wire.WireFundingPaymentPage(
  items: value.items.map(_fundingPaymentToWire).toList(growable: false),
  next: value.next?.value,
);

native_stream.WireMarketEvent _marketEventToWire(MarketEvent value) =>
    switch (value) {
      TradeMarketEvent(:final value) => native_stream.WireMarketEvent.trade(
        _tradeToWire(value),
      ),
      OrderBookMarketEvent(:final value) =>
        native_stream.WireMarketEvent.orderBook(_orderBookToWire(value)),
      TickerMarketEvent(:final value) => native_stream.WireMarketEvent.ticker(
        _tickerToWire(value),
      ),
      CandleMarketEvent(:final value) => native_stream.WireMarketEvent.candle(
        _candleToWire(value),
      ),
      ReconnectedMarketEvent() =>
        const native_stream.WireMarketEvent.reconnected(),
    };

MarketEvent _marketEventFromWire(native_stream.WireMarketEvent value) =>
    switch (value) {
      native_stream.WireMarketEvent_Trade(:final field0) => MarketEvent.trade(
        _tradeFromWire(field0),
      ),
      native_stream.WireMarketEvent_OrderBook(:final field0) =>
        MarketEvent.orderBook(_orderBookFromWire(field0)),
      native_stream.WireMarketEvent_Ticker(:final field0) => MarketEvent.ticker(
        _tickerFromWire(field0),
      ),
      native_stream.WireMarketEvent_Candle(:final field0) => MarketEvent.candle(
        _candleFromWire(field0),
      ),
      native_stream.WireMarketEvent_Reconnected() =>
        const MarketEvent.reconnected(),
    };

native_stream.WireAccountEvent _accountEventToWire(AccountEvent value) =>
    switch (value) {
      BalanceAccountEvent(:final value) =>
        native_stream.WireAccountEvent.balance(_balanceToWire(value)),
      OrderAccountEvent(:final value) => native_stream.WireAccountEvent.order(
        _orderToWire(value),
      ),
      ReconnectedAccountEvent() =>
        const native_stream.WireAccountEvent.reconnected(),
    };

AccountEvent _accountEventFromWire(native_stream.WireAccountEvent value) =>
    switch (value) {
      native_stream.WireAccountEvent_Balance(:final field0) =>
        AccountEvent.balance(_balanceFromWire(field0)),
      native_stream.WireAccountEvent_Order(:final field0) => AccountEvent.order(
        _orderFromWire(field0),
      ),
      native_stream.WireAccountEvent_Reconnected() =>
        const AccountEvent.reconnected(),
    };

Stream<StreamItem<MarketEvent>> _nativeMarketItems(
  native_stream.NativeMarketSubscription subscription,
) async* {
  while (true) {
    final item = await _nativeFuture(
      () => native.nativeMarketSubscriptionNext(subscription: subscription),
    );
    switch (item) {
      case native_stream.WireMarketStreamItem_Event(:final field0):
        yield StreamItem.event(_marketEventFromWire(field0));
      case native_stream.WireMarketStreamItem_Error(:final field0):
        yield StreamItem.error(_nativeError(field0));
      case native_stream.WireMarketStreamItem_End():
        return;
    }
  }
}

Stream<StreamItem<AccountEvent>> _nativeAccountItems(
  native_stream.NativeAccountSubscription subscription,
) async* {
  while (true) {
    final item = await _nativeFuture(
      () => native.nativeAccountSubscriptionNext(subscription: subscription),
    );
    switch (item) {
      case native_stream.WireAccountStreamItem_Event(:final field0):
        yield StreamItem.event(_accountEventFromWire(field0));
      case native_stream.WireAccountStreamItem_Error(:final field0):
        yield StreamItem.error(_nativeError(field0));
      case native_stream.WireAccountStreamItem_End():
        return;
    }
  }
}

UpbitMarketEvent _upbitMarketEventFromWire(wire.WireUpbitMarketEvent value) =>
    UpbitMarketEvent(
      market: _marketFromWire(value.market),
      warning: value.warning,
      cautions: value.cautions,
    );

BithumbMarketWarning _bithumbMarketWarningFromWire(
  wire.WireBithumbMarketWarning value,
) => BithumbMarketWarning(
  market: _marketFromWire(value.market),
  warning: value.warning,
);

BithumbMarketAlert _bithumbMarketAlertFromWire(
  wire.WireBithumbMarketAlert value,
) => BithumbMarketAlert(
  market: _marketFromWire(value.market),
  kind: value.kind,
  step: _enumByName(BithumbAlertStep.values, value.step),
  endsAt: Timestamp.fromNanoseconds(value.endsAtNs),
);

BinanceSymbolFilters _binanceSymbolFiltersFromWire(
  wire.WireBinanceSymbolFilters value,
) => BinanceSymbolFilters(
  symbol: value.symbol,
  tickSize: _decimalFromWire(value.tickSize),
  minPrice: _decimalFromWire(value.minPrice),
  maxPrice: _decimalFromWire(value.maxPrice),
  stepSize: _decimalFromWire(value.stepSize),
  minQuantity: _decimalFromWire(value.minQuantity),
  maxQuantity: _decimalFromWire(value.maxQuantity),
  minNotional: _decimalFromWire(value.minNotional),
);

BinanceSpotOrderDetail _binanceSpotOrderFromWire(
  wire.WireBinanceSpotOrderDetail value,
) => BinanceSpotOrderDetail(
  order: _orderFromWire(value.order),
  clientOrderId: value.clientOrderId,
  orderType: value.orderType,
  timeInForce: value.timeInForce,
  filledQuoteQuantity: Decimal.parse(value.filledQuoteQuantity),
  updatedAt: _timestampFromWire(value.updatedAtNs),
);

HyperliquidLedgerKind _hyperliquidLedgerKindFromWire(
  wire.WireHyperliquidLedgerEntry value,
) => value.kind == wire.WireHyperliquidLedgerKind.other
    ? HyperliquidLedgerKind.other(value.providerKind ?? 'other')
    : switch (value.kind) {
        wire.WireHyperliquidLedgerKind.deposit => HyperliquidLedgerKind.deposit,
        wire.WireHyperliquidLedgerKind.withdraw =>
          HyperliquidLedgerKind.withdraw,
        wire.WireHyperliquidLedgerKind.internalTransfer =>
          HyperliquidLedgerKind.internalTransfer,
        wire.WireHyperliquidLedgerKind.subAccountTransfer =>
          HyperliquidLedgerKind.subAccountTransfer,
        wire.WireHyperliquidLedgerKind.spotTransfer =>
          HyperliquidLedgerKind.spotTransfer,
        wire.WireHyperliquidLedgerKind.accountClassTransfer =>
          HyperliquidLedgerKind.accountClassTransfer,
        wire.WireHyperliquidLedgerKind.vaultDeposit =>
          HyperliquidLedgerKind.vaultDeposit,
        wire.WireHyperliquidLedgerKind.vaultWithdraw =>
          HyperliquidLedgerKind.vaultWithdraw,
        wire.WireHyperliquidLedgerKind.vaultDistribution =>
          HyperliquidLedgerKind.vaultDistribution,
        wire.WireHyperliquidLedgerKind.liquidation =>
          HyperliquidLedgerKind.liquidation,
        wire.WireHyperliquidLedgerKind.other => throw StateError('unreachable'),
      };

HyperliquidLedgerEntry _hyperliquidLedgerEntryFromWire(
  wire.WireHyperliquidLedgerEntry value,
) => HyperliquidLedgerEntry(
  kind: _hyperliquidLedgerKindFromWire(value),
  time: Timestamp.fromNanoseconds(value.timeNs),
  hash: value.hash,
  asset: value.asset,
  amount: _decimalFromWire(value.amount),
  fee: _decimalFromWire(value.fee),
  counterparty: value.counterparty,
);

Page<HyperliquidLedgerEntry> _hyperliquidLedgerPageFromWire(
  wire.WireHyperliquidLedgerPage value,
) => Page(
  items: value.items.map(_hyperliquidLedgerEntryFromWire),
  next: value.next == null ? null : Cursor(value.next!),
);

HyperliquidAssetContext _hyperliquidAssetContextFromWire(
  wire.WireHyperliquidAssetContext value,
) => HyperliquidAssetContext(
  midPrice: _decimalFromWire(value.midPrice),
  markPrice: _decimalFromWire(value.markPrice),
  oraclePrice: _decimalFromWire(value.oraclePrice),
  fundingRate: _decimalFromWire(value.fundingRate),
  openInterest: _decimalFromWire(value.openInterest),
  sizeDecimals: value.sizeDecimals,
  priceDecimals: value.priceDecimals,
);

T _nativeSync<T>(T Function() call) {
  try {
    return call();
  } on wire.NativeError catch (error, stackTrace) {
    Error.throwWithStackTrace(_nativeError(error), stackTrace);
  }
}

Future<T> _nativeFuture<T>(Future<T> Function() call) async {
  try {
    return await call();
  } on wire.NativeError catch (error, stackTrace) {
    Error.throwWithStackTrace(_nativeError(error), stackTrace);
  }
}

Object _nativeError(wire.NativeError value) => switch (value.kind) {
  wire.NativeErrorKind.invalidRequest => InvalidRequestError(
    field: value.field ?? 'request',
    detail: value.detail ?? value.message,
  ),
  wire.NativeErrorKind.unsupported => _unsupportedErrorFromWire(value),
  wire.NativeErrorKind.adapter => AdapterError(value.detail ?? value.message),
  wire.NativeErrorKind.auth => AuthenticationError(
    value.detail ?? value.message,
  ),
  wire.NativeErrorKind.exchange => _exchangeErrorFromWire(value),
  wire.NativeErrorKind.transport => TransportError(
    value.detail ?? value.message,
  ),
  wire.NativeErrorKind.decode => DecodeError(value.detail ?? value.message),
};

wire.NativeError _errorToWire(
  Object value, {
  Feature? feature,
  Exchange? exchange,
}) {
  final message = value.toString();
  return switch (value) {
    InvalidRequestError error => wire.NativeError(
      kind: wire.NativeErrorKind.invalidRequest,
      message: message,
      detail: error.detail,
      field: error.field,
      retryable: false,
      rateLimited: false,
    ),
    UnsupportedError error => wire.NativeError(
      kind: wire.NativeErrorKind.unsupported,
      message: message,
      detail: error.detail,
      feature: _featureToWire(error.feature),
      exchange: error.exchange.id,
      retryable: false,
      rateLimited: false,
    ),
    AuthenticationError error => wire.NativeError(
      kind: wire.NativeErrorKind.auth,
      message: message,
      detail: error.detail,
      retryable: false,
      rateLimited: false,
    ),
    ExchangeError error => wire.NativeError(
      kind: wire.NativeErrorKind.exchange,
      message: message,
      detail: error.message,
      exchange: error.exchange.id,
      code: error.code,
      status: error.status,
      exchangeKind: _enumByName(wire.WireExchangeErrorKind.values, error.kind),
      retryable: error.isRetryable,
      rateLimited: error.isRateLimited,
    ),
    TransportError error => wire.NativeError(
      kind: wire.NativeErrorKind.transport,
      message: message,
      detail: error.detail,
      retryable: true,
      rateLimited: false,
    ),
    DecodeError error => wire.NativeError(
      kind: wire.NativeErrorKind.decode,
      message: message,
      detail: error.detail,
      retryable: false,
      rateLimited: false,
    ),
    AdapterError error => wire.NativeError(
      kind: wire.NativeErrorKind.adapter,
      message: message,
      detail: error.detail,
      retryable: false,
      rateLimited: false,
    ),
    _ => wire.NativeError(
      kind: wire.NativeErrorKind.adapter,
      message: message,
      detail: message,
      retryable: false,
      rateLimited: false,
    ),
  };
}

MaxtError _unsupportedErrorFromWire(wire.NativeError value) {
  final feature = value.feature;
  final exchange = switch (value.exchange) {
    'upbit' => Exchange.upbit,
    'bithumb' => Exchange.bithumb,
    'binance' => Exchange.binance,
    'hyperliquid' => Exchange.hyperliquid,
    _ => null,
  };
  if (feature == null || exchange == null) {
    return const AdapterError(
      'native unsupported error has no known feature or exchange',
    );
  }
  return UnsupportedError(
    feature: _featureFromWire(feature),
    exchange: exchange,
    detail: value.detail ?? value.message,
  );
}

ExchangeError _exchangeErrorFromWire(wire.NativeError value) => ExchangeError(
  exchange: Exchange.values.byName(value.exchange!),
  code: value.code ?? '',
  message: value.detail ?? value.message,
  status: value.status,
  kind: value.exchangeKind == null
      ? ExchangeErrorKind.unknown
      : _enumByName(ExchangeErrorKind.values, value.exchangeKind!),
);
