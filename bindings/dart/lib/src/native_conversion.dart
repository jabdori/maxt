part of 'adapters.dart';

T _enumByName<T extends Enum>(List<T> values, Enum value) =>
    values.byName(value.name);

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
  validateStreamConfigIntegers(value);
  return native_adapter.WireStreamConfig(
    maxReconnectAttempts: value.maxReconnectAttempts,
    initialReconnectDelayMs: BigInt.from(value.initialReconnectDelayMs),
    maxReconnectDelayMs: BigInt.from(value.maxReconnectDelayMs),
    idleTimeoutMs: BigInt.from(value.idleTimeoutMs),
    bufferSize: BigInt.from(value.bufferSize),
    overflow: _enumByName(native_adapter.WireOverflow.values, value.overflow),
  );
}

void validateStreamConfigIntegers(StreamConfig value) {
  final maxReconnectAttempts = value.maxReconnectAttempts;
  if (maxReconnectAttempts != null) {
    validateUnsigned(
      maxReconnectAttempts,
      field: 'maxReconnectAttempts',
      max: _uint32Max,
    );
  }
  validateUnsigned(
    value.initialReconnectDelayMs,
    field: 'initialReconnectDelayMs',
    max: _uint64Max,
  );
  validateUnsigned(
    value.maxReconnectDelayMs,
    field: 'maxReconnectDelayMs',
    max: _uint64Max,
  );
  validateUnsigned(
    value.idleTimeoutMs,
    field: 'idleTimeoutMs',
    max: _uint64Max,
  );
  validateUnsigned(value.bufferSize, field: 'bufferSize', max: _uint32Max);
}

final BigInt _uint32Max = BigInt.parse('4294967295');
final BigInt _uint64Max = BigInt.parse('18446744073709551615');

void validateUnsigned(int value, {required String field, required BigInt max}) {
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

int? checkedUint32(int? value, {required String field}) {
  if (value != null) {
    validateUnsigned(value, field: field, max: _uint32Max);
  }
  return value;
}

int checkedRequiredUint32(int value, {required String field}) {
  validateUnsigned(value, field: field, max: _uint32Max);
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

Decimal? _decimalFromWire(String? value) =>
    value == null ? null : Decimal.parse(value);
Timestamp? _timestampFromWire(PlatformInt64? value) => value == null
    ? null
    : Timestamp.fromNanoseconds(platformInt64ToBigInt(value));
PlatformInt64 _timestampToWire(Timestamp value) =>
    platformInt64FromBigInt(value.nanosecondsSinceEpoch);
PlatformInt64? _optionalTimestampToWire(Timestamp? value) =>
    value == null ? null : _timestampToWire(value);

wire.WireCandleRequest _candleRequestToWire(CandleRequest value) =>
    wire.WireCandleRequest(
      market: _marketToWire(value.market),
      interval: _intervalToWire(value.interval),
      fromNs: _optionalTimestampToWire(value.from),
      toNs: _optionalTimestampToWire(value.to),
      limit: checkedUint32(value.limit, field: 'limit'),
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
      fromNs: _optionalTimestampToWire(value.from),
      toNs: _optionalTimestampToWire(value.to),
      cursor: value.cursor?.value,
      limit: checkedUint32(value.limit, field: 'limit'),
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
      clientId: value.clientId,
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
    OrderType.best => OrderRequest.best(
      market,
      side,
      size,
      _timeInForceFromWire(
        value.timeInForce ??
            (throw StateError('native best order has no time in force')),
      ),
    ),
  };
  if (value.timeInForce != null) {
    request = request.withTimeInForce(_timeInForceFromWire(value.timeInForce!));
  }
  if (value.reduceOnly) {
    request = request.asReduceOnly();
  }
  if (value.clientId != null) {
    request = request.withClientId(value.clientId!);
  }
  return request;
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

Stream<StreamItem<HyperliquidMarketEvent>> _nativeHyperliquidMarketItems(
  native_stream.NativeHyperliquidMarketSubscription subscription,
) async* {
  while (true) {
    final item = await _nativeFuture(
      () => native.nativeHyperliquidMarketSubscriptionNext(
        subscription: subscription,
      ),
    );
    switch (item) {
      case native_stream.WireHyperliquidMarketStreamItem_Event(:final field0):
        yield StreamItem.event(_hyperliquidMarketEventFromWire(field0));
      case native_stream.WireHyperliquidMarketStreamItem_Error(:final field0):
        yield StreamItem.error(_nativeError(field0));
      case native_stream.WireHyperliquidMarketStreamItem_End():
        return;
    }
  }
}

Stream<StreamItem<HyperliquidAccountEvent>> _nativeHyperliquidAccountItems(
  native_stream.NativeHyperliquidAccountSubscription subscription,
) async* {
  while (true) {
    final item = await _nativeFuture(
      () => native.nativeHyperliquidAccountSubscriptionNext(
        subscription: subscription,
      ),
    );
    switch (item) {
      case native_stream.WireHyperliquidAccountStreamItem_Event(:final field0):
        yield StreamItem.event(_hyperliquidAccountEventFromWire(field0));
      case native_stream.WireHyperliquidAccountStreamItem_Error(:final field0):
        yield StreamItem.error(_nativeError(field0));
      case native_stream.WireHyperliquidAccountStreamItem_End():
        return;
    }
  }
}

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
