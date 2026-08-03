part of 'adapters.dart';

/// 생성된 native client handle을 제공하는 package 내부 계약입니다.
abstract interface class NativeHandleProvider {
  native.NativeClient get nativeHandle;
}

/// 공개 [Client]가 모든 공통 호출을 Rust로 보내는 package 내부 delegate입니다.
final class NativeClientDelegate extends AdapterBase {
  NativeClientDelegate._({
    required this.exchange,
    required this.features,
    required Future<Adapter> delegate,
  }) : _delegate = delegate;

  factory NativeClientDelegate.fromAdapter(Adapter adapter) {
    if (!Maxt.isInitialized) {
      throw StateError(
        'Call and await Maxt.initialize() before constructing a Client.',
      );
    }
    final Future<Adapter> delegate = switch (adapter) {
      NativeHandleProvider(:final nativeHandle) => Future<Adapter>.value(
        _NativeDelegateAdapter(nativeHandle),
      ),
      _ when bridgeCustomAdapters => DartAdapterBridge(
        adapter,
      ).register().then(_NativeDelegateAdapter.new),
      _ => Future<Adapter>.value(adapter),
    };
    return NativeClientDelegate._(
      exchange: adapter.exchange,
      features: Set.unmodifiable(adapter.features),
      delegate: delegate,
    );
  }

  final Future<Adapter> _delegate;

  @override
  final Exchange exchange;

  @override
  final Set<Feature> features;

  Future<Adapter> get _native => _delegate;

  @override
  Future<List<MarketInfo>> markets(MarketKind kind) async =>
      (await _native).markets(kind);

  @override
  Future<List<Trade>> trades(Market market, [int? limit]) async =>
      (await _native).trades(market, limit);

  @override
  Future<OrderBook> orderBook(Market market, [int? depth]) async =>
      (await _native).orderBook(market, depth);

  @override
  Future<Ticker> ticker(Market market) async => (await _native).ticker(market);

  @override
  Future<List<Candle>> candles(CandleRequest request) async =>
      (await _native).candles(request);

  @override
  Future<MarketStream> subscribe(
    Subscription subscription,
    StreamConfig config,
  ) async => (await _native).subscribe(subscription, config);

  @override
  Future<List<Balance>> balances() async => (await _native).balances();

  @override
  Future<List<Order>> openOrders([Market? market]) async =>
      (await _native).openOrders(market);

  @override
  Future<AccountStream> subscribeAccount(StreamConfig config) async =>
      (await _native).subscribeAccount(config);

  @override
  Future<Order> placeOrder(OrderRequest request) async =>
      (await _native).placeOrder(request);

  @override
  Future<Order> cancelOrder(Market market, String orderId) async =>
      (await _native).cancelOrder(market, orderId);

  @override
  Future<List<Position>> positions([Market? market]) async =>
      (await _native).positions(market);

  @override
  Future<MarginSummary> marginSummary() async =>
      (await _native).marginSummary();

  @override
  Future<Page<FundingRate>> fundingRates(HistoryRequest request) async =>
      (await _native).fundingRates(request);

  @override
  Future<Page<FundingPayment>> fundingPayments(HistoryRequest request) async =>
      (await _native).fundingPayments(request);

  @override
  Future<void> setMargin(MarginRequest request) async =>
      (await _native).setMargin(request);
}

final class _NativeDelegateAdapter extends _NativeAdapterBase {
  _NativeDelegateAdapter(super.handle);
}

/// Dart [Adapter]를 Rust ForeignAdapter로 등록하는 package 내부 브리지입니다.
final class DartAdapterBridge {
  DartAdapterBridge(this.adapter);

  final Adapter adapter;
  final DartStreamRegistry streams = DartStreamRegistry();

  Future<native.NativeClient> register() async {
    final dartAdapter = await _nativeFuture(
      () => native.registerDartAdapter(
        exchange: _exchangeToWire(adapter.exchange),
        features: adapter.features.map(_featureToWire).toList(growable: false),
        dispatcher: dispatch,
      ),
    );
    return _nativeSync(
      () => native.NativeClient.fromDartAdapter(adapter: dartAdapter),
    );
  }

  Future<native_adapter.AdapterResult> dispatch(
    native_adapter.AdapterCall call,
  ) async {
    try {
      return native_adapter.AdapterResult.success(await _reply(call));
    } catch (error) {
      return native_adapter.AdapterResult.error(
        _errorToWire(
          error,
          feature: _featureForCall(call),
          exchange: adapter.exchange,
        ),
      );
    }
  }

  Future<native_adapter.AdapterReply> _reply(
    native_adapter.AdapterCall call,
  ) => switch (call) {
    native_adapter.AdapterCall_Markets(:final kind) =>
      adapter
          .markets(_marketKindFromWire(kind))
          .then(
            (values) => native_adapter.AdapterReply.markets(
              values.map(_marketInfoToWire).toList(growable: false),
            ),
          ),
    native_adapter.AdapterCall_Trades(:final market, :final limit) =>
      adapter
          .trades(_marketFromWire(market), limit)
          .then(
            (values) => native_adapter.AdapterReply.trades(
              values.map(_tradeToWire).toList(growable: false),
            ),
          ),
    native_adapter.AdapterCall_OrderBook(:final market, :final depth) =>
      adapter
          .orderBook(_marketFromWire(market), depth)
          .then(
            (value) =>
                native_adapter.AdapterReply.orderBook(_orderBookToWire(value)),
          ),
    native_adapter.AdapterCall_Ticker(:final market) =>
      adapter
          .ticker(_marketFromWire(market))
          .then(
            (value) => native_adapter.AdapterReply.ticker(_tickerToWire(value)),
          ),
    native_adapter.AdapterCall_Candles(:final request) =>
      adapter
          .candles(_candleRequestFromWire(request))
          .then(
            (values) => native_adapter.AdapterReply.candles(
              values.map(_candleToWire).toList(growable: false),
            ),
          ),
    native_adapter.AdapterCall_Balances() => adapter.balances().then(
      (values) => native_adapter.AdapterReply.balances(
        values.map(_balanceToWire).toList(growable: false),
      ),
    ),
    native_adapter.AdapterCall_OpenOrders(:final market) =>
      adapter
          .openOrders(market == null ? null : _marketFromWire(market))
          .then(
            (values) => native_adapter.AdapterReply.openOrders(
              values.map(_orderToWire).toList(growable: false),
            ),
          ),
    native_adapter.AdapterCall_PlaceOrder(:final request) =>
      adapter
          .placeOrder(_orderRequestFromWire(request))
          .then(
            (value) =>
                native_adapter.AdapterReply.placeOrder(_orderToWire(value)),
          ),
    native_adapter.AdapterCall_CancelOrder(:final market, :final orderId) =>
      adapter
          .cancelOrder(_marketFromWire(market), orderId)
          .then(
            (value) =>
                native_adapter.AdapterReply.cancelOrder(_orderToWire(value)),
          ),
    native_adapter.AdapterCall_Positions(:final market) =>
      adapter
          .positions(market == null ? null : _marketFromWire(market))
          .then(
            (values) => native_adapter.AdapterReply.positions(
              values.map(_positionToWire).toList(growable: false),
            ),
          ),
    native_adapter.AdapterCall_MarginSummary() => adapter.marginSummary().then(
      (value) => native_adapter.AdapterReply.marginSummary(
        _marginSummaryToWire(value),
      ),
    ),
    native_adapter.AdapterCall_FundingRates(:final request) =>
      adapter
          .fundingRates(_historyRequestFromWire(request))
          .then(
            (value) => native_adapter.AdapterReply.fundingRates(
              _fundingRatePageToWire(value),
            ),
          ),
    native_adapter.AdapterCall_FundingPayments(:final request) =>
      adapter
          .fundingPayments(_historyRequestFromWire(request))
          .then(
            (value) => native_adapter.AdapterReply.fundingPayments(
              _fundingPaymentPageToWire(value),
            ),
          ),
    native_adapter.AdapterCall_SetMargin(:final request) =>
      adapter
          .setMargin(_marginRequestFromWire(request))
          .then((_) => const native_adapter.AdapterReply.unit()),
    native_adapter.AdapterCall_Subscribe() => _subscribeMarket(call),
    native_adapter.AdapterCall_SubscribeAccount() => _subscribeAccount(call),
    native_adapter.AdapterCall_CancelStream(:final streamId) =>
      streams
          .cancel(streamId)
          .then((_) => const native_adapter.AdapterReply.unit()),
  };

  Future<native_adapter.AdapterReply> _subscribeMarket(
    native_adapter.AdapterCall_Subscribe call,
  ) async {
    try {
      final stream = await adapter.subscribe(
        _subscriptionFromWire(call.subscription),
        _streamConfigFromWire(call.config),
      );
      final accepted = await streams.register(call.streamId, stream.close);
      if (accepted) {
        unawaited(
          _pumpMarket(
            call.streamId,
            stream,
            call.sink,
            _streamFeature(call.subscription),
          ),
        );
      } else {
        await _endMarket(call.sink);
      }
      return const native_adapter.AdapterReply.unit();
    } catch (error, stackTrace) {
      try {
        await _endMarket(call.sink);
      } catch (_) {
        // 초기 오류를 sink 정리 오류로 대체하지 않습니다.
      } finally {
        streams.forgetPending(call.streamId);
      }
      Error.throwWithStackTrace(error, stackTrace);
    }
  }

  Future<native_adapter.AdapterReply> _subscribeAccount(
    native_adapter.AdapterCall_SubscribeAccount call,
  ) async {
    try {
      final stream = await adapter.subscribeAccount(
        _streamConfigFromWire(call.config),
      );
      final accepted = await streams.register(call.streamId, stream.close);
      if (accepted) {
        unawaited(_pumpAccount(call.streamId, stream, call.sink));
      } else {
        await _endAccount(call.sink);
      }
      return const native_adapter.AdapterReply.unit();
    } catch (error, stackTrace) {
      try {
        await _endAccount(call.sink);
      } catch (_) {
        // 초기 오류를 sink 정리 오류로 대체하지 않습니다.
      } finally {
        streams.forgetPending(call.streamId);
      }
      Error.throwWithStackTrace(error, stackTrace);
    }
  }

  Future<void> _pumpMarket(
    String id,
    MarketStream stream,
    native_stream.MarketStreamSink sink,
    Feature feature,
  ) async {
    try {
      await for (final item in stream) {
        final wireItem = switch (item) {
          StreamEvent<MarketEvent>(:final event) =>
            native_stream.MarketStreamItem.event(_marketEventToWire(event)),
          StreamError<MarketEvent>(:final error) =>
            native_stream.MarketStreamItem.error(
              _errorToWire(error, feature: feature, exchange: adapter.exchange),
            ),
        };
        if (!await native.marketStreamSinkAdd(sink: sink, item: wireItem)) {
          break;
        }
      }
    } catch (error) {
      await _sendMarketError(sink, error, feature);
    }

    try {
      await streams.finish(id);
    } catch (error) {
      await _sendMarketError(sink, error, feature);
    }
    try {
      await _endMarket(sink);
    } catch (_) {
      // 닫힌 native sink에서는 End를 전달할 수 없습니다.
    }
  }

  Future<void> _pumpAccount(
    String id,
    AccountStream stream,
    native_stream.AccountStreamSink sink,
  ) async {
    try {
      await for (final item in stream) {
        final wireItem = switch (item) {
          StreamEvent<AccountEvent>(:final event) =>
            native_stream.AccountStreamItem.event(_accountEventToWire(event)),
          StreamError<AccountEvent>(:final error) =>
            native_stream.AccountStreamItem.error(
              _errorToWire(
                error,
                feature: Feature.accountStream,
                exchange: adapter.exchange,
              ),
            ),
        };
        if (!await native.accountStreamSinkAdd(sink: sink, item: wireItem)) {
          break;
        }
      }
    } catch (error) {
      await _sendAccountError(sink, error);
    }

    try {
      await streams.finish(id);
    } catch (error) {
      await _sendAccountError(sink, error);
    }
    try {
      await _endAccount(sink);
    } catch (_) {
      // 닫힌 native sink에서는 End를 전달할 수 없습니다.
    }
  }

  Future<void> _sendMarketError(
    native_stream.MarketStreamSink sink,
    Object error,
    Feature feature,
  ) async {
    try {
      await native.marketStreamSinkAdd(
        sink: sink,
        item: native_stream.MarketStreamItem.error(
          _errorToWire(error, feature: feature, exchange: adapter.exchange),
        ),
      );
    } catch (_) {
      // 닫힌 native sink에서는 추가 오류를 전달할 수 없습니다.
    }
  }

  Future<void> _sendAccountError(
    native_stream.AccountStreamSink sink,
    Object error,
  ) async {
    try {
      await native.accountStreamSinkAdd(
        sink: sink,
        item: native_stream.AccountStreamItem.error(
          _errorToWire(
            error,
            feature: Feature.accountStream,
            exchange: adapter.exchange,
          ),
        ),
      );
    } catch (_) {
      // 닫힌 native sink에서는 추가 오류를 전달할 수 없습니다.
    }
  }

  Future<bool> _endMarket(native_stream.MarketStreamSink sink) =>
      native.marketStreamSinkAdd(
        sink: sink,
        item: const native_stream.MarketStreamItem.end(),
      );

  Future<bool> _endAccount(native_stream.AccountStreamSink sink) =>
      native.accountStreamSinkAdd(
        sink: sink,
        item: const native_stream.AccountStreamItem.end(),
      );

  Feature _streamFeature(native_adapter.WireSubscription subscription) =>
      subscription.feeds.isEmpty
      ? Feature.tradeStream
      : switch (subscription.feeds.first) {
          native_adapter.WireFeed_Trades() => Feature.tradeStream,
          native_adapter.WireFeed_OrderBook() => Feature.orderBookStream,
          native_adapter.WireFeed_Ticker() => Feature.tickerStream,
          native_adapter.WireFeed_Candles() => Feature.candleStream,
        };

  Feature _featureForCall(native_adapter.AdapterCall call) => switch (call) {
    native_adapter.AdapterCall_Markets() => Feature.markets,
    native_adapter.AdapterCall_Trades() => Feature.trades,
    native_adapter.AdapterCall_OrderBook() => Feature.orderBook,
    native_adapter.AdapterCall_Ticker() => Feature.ticker,
    native_adapter.AdapterCall_Candles() => Feature.candles,
    native_adapter.AdapterCall_Balances() => Feature.balances,
    native_adapter.AdapterCall_OpenOrders() => Feature.openOrders,
    native_adapter.AdapterCall_PlaceOrder() ||
    native_adapter.AdapterCall_CancelOrder() => Feature.trading,
    native_adapter.AdapterCall_Positions() => Feature.positions,
    native_adapter.AdapterCall_MarginSummary() => Feature.margin,
    native_adapter.AdapterCall_FundingRates() => Feature.fundingRates,
    native_adapter.AdapterCall_FundingPayments() => Feature.fundingPayments,
    native_adapter.AdapterCall_SetMargin() => Feature.marginConfig,
    native_adapter.AdapterCall_Subscribe(:final subscription) => _streamFeature(
      subscription,
    ),
    native_adapter.AdapterCall_SubscribeAccount() => Feature.accountStream,
    native_adapter.AdapterCall_CancelStream() => Feature.accountStream,
  };
}
