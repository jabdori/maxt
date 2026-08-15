part of 'adapters.dart';

/// Package-internal contract that provides a generated native-client handle.
abstract interface class NativeHandleProvider {
  native.NativeClient get nativeHandle;
}

native.NativeClient? _nativeHandle(Adapter adapter) => switch (adapter) {
  NativeHandleProvider(:final nativeHandle) => nativeHandle,
  _ => null,
};

/// Package-internal delegate through which public [Client] sends common calls to Rust.
final class NativeClientDelegate extends GeneratedNativeDelegate {
  NativeClientDelegate._({
    required this.exchange,
    required this.features,
    required Future<Adapter> delegate,
    required bool directCustomAdapter,
  }) : _delegate = delegate,
       _directCustomAdapter = directCustomAdapter;

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
      directCustomAdapter:
          !bridgeCustomAdapters && adapter is! NativeHandleProvider,
    );
  }

  final Future<Adapter> _delegate;
  final bool _directCustomAdapter;

  @override
  final Exchange exchange;

  @override
  final Set<Feature> features;

  @override
  Future<Adapter> get delegateAdapter => _delegate;

  @override
  bool supports(Feature feature) => features.contains(feature);

  @override
  Future<MarketStream> subscribe(
    Subscription subscription,
    StreamConfig config,
  ) async {
    final stream = await (await delegateAdapter).subscribe(
      subscription,
      config,
    );
    return _directCustomAdapter
        ? _marketStreamWithCleanupError(stream)
        : stream;
  }

  @override
  Future<AccountStream> subscribeAccount(StreamConfig config) async {
    final stream = await (await delegateAdapter).subscribeAccount(config);
    return _directCustomAdapter
        ? _accountStreamWithCleanupError(stream)
        : stream;
  }

  Future<TransferPlan> prepareTransferTo(
    NativeClientDelegate destination,
    ExchangeTransferRequest request,
  ) async {
    final source = await delegateAdapter;
    final target = await destination.delegateAdapter;
    final sourceHandle = _nativeHandle(source);
    final targetHandle = _nativeHandle(target);
    if (sourceHandle != null && targetHandle != null) {
      return _nativeFuture(
        () => sourceHandle.prepareTransferTo(
          destination: targetHandle,
          request: _exchangeTransferRequestToWire(request),
        ),
      ).then(_transferPlanFromWire);
    }
    return _prepareExchangeTransfer(source, target, request);
  }

  Future<TransferPlan> prepareTransferToChain(
    ChainTransferRequest request,
  ) async {
    final source = await delegateAdapter;
    final sourceHandle = _nativeHandle(source);
    if (sourceHandle != null) {
      return _nativeFuture(
        () => sourceHandle.prepareTransferToChain(
          request: _chainTransferRequestToWire(request),
        ),
      ).then(_transferPlanFromWire);
    }
    return _prepareChainTransfer(source, request);
  }

  Future<Withdrawal> executeTransfer(TransferPlan plan) async {
    final source = await delegateAdapter;
    final sourceHandle = _nativeHandle(source);
    if (sourceHandle != null) {
      return _nativeFuture(
        () => sourceHandle.executeTransfer(plan: _transferPlanToWire(plan)),
      ).then(_withdrawalFromWire);
    }
    return _executeTransfer(source, plan);
  }
}

const int _defaultTransferPlanLifetimeNanoseconds = 60000000000;

Future<TransferPlan> _prepareExchangeTransfer(
  Adapter source,
  Adapter destination,
  ExchangeTransferRequest request,
) async {
  final asset = request.asset;
  if (asset.isEmpty) {
    throw const InvalidRequestError(
      field: 'asset',
      detail: 'asset must not be empty',
    );
  }
  _validatePositiveAmount(request.amount);
  final requestedSource = request.sourceNetwork;
  final requestedDestination = request.destinationNetwork;
  if (requestedSource != null &&
      requestedDestination != null &&
      !_sameChain(requestedSource, requestedDestination)) {
    throw TransferError(
      kind: TransferErrorKind.networkMismatch,
      detail:
          'source network $requestedSource differs from destination network '
          '$requestedDestination',
    );
  }

  final sourceNetworks = await source.assetNetworks(asset);
  final destinationNetworks = await destination.assetNetworks(asset);
  final (sourceNetwork, destinationNetwork) = _selectNetworks(
    sourceNetworks,
    requestedSource,
    destinationNetworks,
    requestedDestination,
  );
  _validateTransferAmount(
    request.amount,
    sourceNetwork.minimumWithdrawal,
    sourceNetwork.maximumWithdrawal,
  );

  final address = await destination.depositAddress(
    DepositAddressRequest(asset: asset, network: destinationNetwork.network),
  );
  if (address.exchange != destination.exchange ||
      address.asset != asset ||
      !_sameChain(address.network, sourceNetwork.network)) {
    throw AdapterError(
      '${destination.exchange.displayName} returned a deposit destination '
      'that does not match $asset on ${sourceNetwork.network}',
    );
  }
  if (destinationNetwork.memoRequired && address.memo == null) {
    throw TransferError(
      kind: TransferErrorKind.memoRequired,
      detail:
          '${destination.exchange.displayName} $asset deposits on '
          '${destinationNetwork.network} require a memo or tag',
    );
  }
  final destinationAddress = address.address;
  if (destinationAddress == null) {
    throw TransferError(
      kind: TransferErrorKind.destinationUnavailable,
      detail:
          '${destination.exchange.displayName} has not issued a $asset '
          'deposit address on ${destinationNetwork.network} yet',
    );
  }

  final withdrawal = WithdrawRequest(
    asset: asset,
    network: sourceNetwork.network,
    amount: request.amount,
    destination: TransferDestination.exchange(
      ExchangeDestination(
        exchange: address.exchange,
        asset: address.asset,
        network: address.network,
        address: destinationAddress,
        memo: address.memo,
      ),
    ),
    clientId: _uuidV4(),
  );
  final quote = await source.prepareWithdrawal(withdrawal);
  _validateTransferQuote(request.amount, quote);
  return _transferPlan(
    source.exchange,
    destination.exchange,
    withdrawal,
    quote,
  );
}

Future<TransferPlan> _prepareChainTransfer(
  Adapter source,
  ChainTransferRequest request,
) async {
  final asset = request.asset;
  if (asset.isEmpty) {
    throw const InvalidRequestError(
      field: 'asset',
      detail: 'asset must not be empty',
    );
  }
  if (asset != request.destination.asset) {
    throw TransferError(
      kind: TransferErrorKind.assetMismatch,
      detail:
          'source asset $asset differs from destination asset '
          '${request.destination.asset}',
    );
  }
  final requestedNetwork = request.sourceNetwork;
  if (requestedNetwork != null &&
      !_sameChain(requestedNetwork, request.destination.network)) {
    throw TransferError(
      kind: TransferErrorKind.networkMismatch,
      detail:
          'source network $requestedNetwork differs from destination network '
          '${request.destination.network}',
    );
  }
  _validatePositiveAmount(request.amount);

  final networks = await source.assetNetworks(asset);
  final sourceNetwork = networks
      .where(
        (candidate) =>
            candidate.withdrawalEnabled &&
            _sameChain(candidate.network, request.destination.network),
      )
      .firstOrNull;
  if (sourceNetwork == null) {
    throw TransferError(
      kind: TransferErrorKind.networkUnavailable,
      detail:
          '${request.destination.network} is not enabled for $asset withdrawal',
    );
  }
  _validateTransferAmount(
    request.amount,
    sourceNetwork.minimumWithdrawal,
    sourceNetwork.maximumWithdrawal,
  );
  final withdrawal = WithdrawRequest(
    asset: asset,
    network: sourceNetwork.network,
    amount: request.amount,
    destination: TransferDestination.chain(request.destination),
    clientId: _uuidV4(),
  );
  final quote = await source.prepareWithdrawal(withdrawal);
  _validateTransferQuote(request.amount, quote);
  return _transferPlan(source.exchange, null, withdrawal, quote);
}

Future<Withdrawal> _executeTransfer(Adapter source, TransferPlan plan) {
  if (source.exchange != plan.source) {
    throw InvalidRequestError(
      field: 'plan.source',
      detail:
          'plan belongs to ${plan.source.displayName}, not '
          '${source.exchange.displayName}',
    );
  }
  if (Timestamp.now().compareTo(plan.expiresAt) >= 0) {
    throw TransferError(
      kind: TransferErrorKind.planExpired,
      detail: 'plan expired at ${plan.expiresAt.nanosecondsSinceEpoch}',
    );
  }
  return source.withdraw(plan.request);
}

(AssetNetwork, AssetNetwork) _selectNetworks(
  List<AssetNetwork> source,
  Network? requestedSource,
  List<AssetNetwork> destination,
  Network? requestedDestination,
) {
  final requested = requestedSource ?? requestedDestination;
  if (requested != null) {
    final sourceNetwork = source
        .where(
          (candidate) =>
              candidate.withdrawalEnabled &&
              _sameChain(candidate.network, requested),
        )
        .firstOrNull;
    final destinationNetwork = destination
        .where(
          (candidate) =>
              candidate.depositEnabled &&
              _sameChain(candidate.network, requested),
        )
        .firstOrNull;
    if (sourceNetwork != null && destinationNetwork != null) {
      return (sourceNetwork, destinationNetwork);
    }
    throw TransferError(
      kind: TransferErrorKind.networkUnavailable,
      detail: '$requested is not enabled for both withdrawal and deposit',
    );
  }

  final matches = <(AssetNetwork, AssetNetwork)>[];
  for (final sourceNetwork in source.where(
    (candidate) => candidate.withdrawalEnabled,
  )) {
    final destinationNetwork = destination
        .where(
          (candidate) =>
              candidate.depositEnabled &&
              _sameChain(candidate.network, sourceNetwork.network),
        )
        .firstOrNull;
    if (destinationNetwork != null &&
        !matches.any(
          (match) => _sameChain(match.$1.network, sourceNetwork.network),
        )) {
      matches.add((sourceNetwork, destinationNetwork));
    }
  }
  return switch (matches.length) {
    0 => throw const TransferError(
      kind: TransferErrorKind.networkMismatch,
      detail: 'source and destination have no enabled network in common',
    ),
    1 => matches.single,
    _ => throw const TransferError(
      kind: TransferErrorKind.ambiguousNetwork,
      detail: 'more than one enabled network is shared; select one explicitly',
    ),
  };
}

void _validatePositiveAmount(Decimal amount) {
  if (amount <= Decimal.zero) {
    throw const TransferError(
      kind: TransferErrorKind.amountOutOfRange,
      detail: 'amount must be greater than zero',
    );
  }
}

void _validateTransferAmount(
  Decimal amount,
  Decimal? minimum,
  Decimal? maximum,
) {
  if (minimum != null && amount < minimum) {
    throw TransferError(
      kind: TransferErrorKind.amountOutOfRange,
      detail: 'amount $amount is below minimum $minimum',
    );
  }
  if (maximum != null && amount > maximum) {
    throw TransferError(
      kind: TransferErrorKind.amountOutOfRange,
      detail: 'amount $amount exceeds maximum $maximum',
    );
  }
}

void _validateTransferQuote(Decimal amount, WithdrawalQuote quote) {
  _validateTransferAmount(amount, quote.minimumAmount, quote.maximumAmount);
  if (quote.addressAllowed == false) {
    throw const TransferError(
      kind: TransferErrorKind.addressNotAllowed,
      detail: 'destination address is not allowed by the source account',
    );
  }
  if (quote.travelRule is TravelRuleRequirementRequired) {
    throw const TransferError(
      kind: TransferErrorKind.travelRuleRequired,
      detail: 'provider-specific Travel Rule data or consent is required',
    );
  }
  final fee = quote.fee;
  if (fee != null && fee >= amount) {
    throw const TransferError(
      kind: TransferErrorKind.amountOutOfRange,
      detail: 'withdrawal fee must be smaller than the amount',
    );
  }
}

TransferPlan _transferPlan(
  Exchange source,
  Exchange? destination,
  WithdrawRequest request,
  WithdrawalQuote quote,
) {
  final createdAt = Timestamp.now();
  final expiresAt =
      quote.expiresAt ??
      Timestamp.fromNanoseconds(
        createdAt.nanosecondsSinceEpoch +
            BigInt.from(_defaultTransferPlanLifetimeNanoseconds),
      );
  return TransferPlan(
    source: source,
    destination: destination,
    request: request,
    quote: quote,
    createdAt: createdAt,
    expiresAt: expiresAt,
  );
}

bool _sameChain(Network left, Network right) =>
    !left.isOther && !right.isOther && left.providerName == right.providerName;

String _uuidV4() {
  final bytes = List<int>.generate(16, (_) => Random.secure().nextInt(256));
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  final hex = bytes
      .map((byte) => byte.toRadixString(16).padLeft(2, '0'))
      .join();
  return '${hex.substring(0, 8)}-${hex.substring(8, 12)}-'
      '${hex.substring(12, 16)}-${hex.substring(16, 20)}-'
      '${hex.substring(20)}';
}

MarketStream _marketStreamWithCleanupError(MarketStream stream) {
  final wrapped = _streamWithCleanupError(stream);
  return MarketStream(wrapped.source, onClose: wrapped.close);
}

AccountStream _accountStreamWithCleanupError(AccountStream stream) {
  final wrapped = _streamWithCleanupError(stream);
  return AccountStream(wrapped.source, onClose: wrapped.close);
}

({Stream<StreamItem<T>> source, Future<void> Function() close})
_streamWithCleanupError<T>(CloseableStream<T> stream) {
  var cleanupReported = false;
  final source = () async* {
    await for (final item in stream) {
      yield item;
    }
    try {
      await stream.close();
    } catch (error) {
      cleanupReported = true;
      yield StreamItem<T>.error(_adapterCleanupError(error));
      return;
    }
    cleanupReported = true;
  }();
  return (
    source: source,
    close: () => cleanupReported ? Future<void>.value() : stream.close(),
  );
}

Object _adapterCleanupError(Object error) =>
    error is MaxtError ? error : AdapterError(error.toString());

final class _NativeDelegateAdapter extends _NativeAdapterBase {
  _NativeDelegateAdapter(super.handle);
}

/// Package-internal bridge that registers a Dart [Adapter] as a Rust ForeignAdapter.
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
    native_adapter.AdapterCall_OrderRules(:final market) =>
      adapter
          .orderRules(_marketFromWire(market))
          .then(
            (value) => native_adapter.AdapterReply.orderRules(
              _orderRulesToWire(value),
            ),
          ),
    native_adapter.AdapterCall_AssetNetworks(:final asset) =>
      adapter
          .assetNetworks(asset)
          .then(
            (values) => native_adapter.AdapterReply.assetNetworks(
              values.map(_assetNetworkToWire).toList(growable: false),
            ),
          ),
    native_adapter.AdapterCall_DepositAddresses() =>
      adapter.depositAddresses().then(
        (values) => native_adapter.AdapterReply.depositAddresses(
          values.map(_depositAddressEntryToWire).toList(growable: false),
        ),
      ),
    native_adapter.AdapterCall_DepositAddress(:final request) =>
      adapter
          .depositAddress(_depositAddressRequestFromWire(request))
          .then(
            (value) => native_adapter.AdapterReply.depositAddress(
              _depositAddressToWire(value),
            ),
          ),
    native_adapter.AdapterCall_CreateDepositAddress(:final request) =>
      adapter
          .createDepositAddress(_depositAddressRequestFromWire(request))
          .then(
            (value) => native_adapter.AdapterReply.createDepositAddress(
              _depositAddressToWire(value),
            ),
          ),
    native_adapter.AdapterCall_PrepareWithdrawal(:final request) =>
      adapter
          .prepareWithdrawal(_withdrawRequestFromWire(request))
          .then(
            (value) => native_adapter.AdapterReply.prepareWithdrawal(
              _withdrawalQuoteToWire(value),
            ),
          ),
    native_adapter.AdapterCall_Withdraw(:final request) =>
      adapter
          .withdraw(_withdrawRequestFromWire(request))
          .then(
            (value) =>
                native_adapter.AdapterReply.withdraw(_withdrawalToWire(value)),
          ),
    native_adapter.AdapterCall_Deposit(:final request) =>
      adapter
          .deposit(_transferLookupRequestFromWire(request))
          .then(
            (value) =>
                native_adapter.AdapterReply.deposit(_depositToWire(value)),
          ),
    native_adapter.AdapterCall_Withdrawal(:final request) =>
      adapter
          .withdrawal(_transferLookupRequestFromWire(request))
          .then(
            (value) => native_adapter.AdapterReply.withdrawal(
              _withdrawalToWire(value),
            ),
          ),
    native_adapter.AdapterCall_CancelWithdrawal(:final withdrawalId) =>
      adapter
          .cancelWithdrawal(withdrawalId)
          .then((_) => const native_adapter.AdapterReply.unit()),
    native_adapter.AdapterCall_Deposits(:final request) =>
      adapter
          .deposits(_transferHistoryRequestFromWire(request))
          .then(
            (value) =>
                native_adapter.AdapterReply.deposits(_depositPageToWire(value)),
          ),
    native_adapter.AdapterCall_Withdrawals(:final request) =>
      adapter
          .withdrawals(_transferHistoryRequestFromWire(request))
          .then(
            (value) => native_adapter.AdapterReply.withdrawals(
              _withdrawalPageToWire(value),
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
    native_adapter.AdapterCall_Order(:final market, :final orderId) =>
      adapter
          .order(_marketFromWire(market), orderId)
          .then(
            (value) => native_adapter.AdapterReply.order(_orderToWire(value)),
          ),
    native_adapter.AdapterCall_OrderByClientId(
      :final market,
      :final clientId,
    ) =>
      adapter
          .orderByClientId(_marketFromWire(market), clientId)
          .then(
            (value) => native_adapter.AdapterReply.order(_orderToWire(value)),
          ),
    native_adapter.AdapterCall_OrdersByIds(:final request) =>
      adapter
          .ordersByIds(_orderLookupRequestFromWire(request))
          .then(
            (values) => native_adapter.AdapterReply.ordersByIds(
              values.map(_orderToWire).toList(growable: false),
            ),
          ),
    native_adapter.AdapterCall_OrderHistory(:final request) =>
      adapter
          .orderHistory(_orderHistoryRequestFromWire(request))
          .then(
            (value) => native_adapter.AdapterReply.orderHistory(
              _orderPageToWire(value),
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
          .then((_) => const native_adapter.AdapterReply.unit()),
    native_adapter.AdapterCall_CancelOrderByClientId(
      :final market,
      :final clientId,
    ) =>
      adapter
          .cancelOrderByClientId(_marketFromWire(market), clientId)
          .then((_) => const native_adapter.AdapterReply.unit()),
    native_adapter.AdapterCall_CancelOrders(:final request) =>
      adapter
          .cancelOrders(_cancelOrdersRequestFromWire(request))
          .then(
            (value) => native_adapter.AdapterReply.cancelOrders(
              _cancelOrdersResultToWire(value),
            ),
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
        // Do not replace the original error with a sink-cleanup error.
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
        // Do not replace the original error with a sink-cleanup error.
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
      // A closed native sink cannot receive End.
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
      // A closed native sink cannot receive End.
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
      // A closed native sink cannot receive another error.
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
      // A closed native sink cannot receive another error.
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
    native_adapter.AdapterCall_OrderRules() => Feature.trading,
    native_adapter.AdapterCall_AssetNetworks() => Feature.assetNetworks,
    native_adapter.AdapterCall_DepositAddresses() => Feature.depositAddresses,
    native_adapter.AdapterCall_DepositAddress() => Feature.depositAddresses,
    native_adapter.AdapterCall_CreateDepositAddress() =>
      Feature.depositAddresses,
    native_adapter.AdapterCall_PrepareWithdrawal() => Feature.withdrawalQuotes,
    native_adapter.AdapterCall_Withdraw() => Feature.withdrawals,
    native_adapter.AdapterCall_Deposit() => Feature.depositLookup,
    native_adapter.AdapterCall_Withdrawal() => Feature.withdrawalLookup,
    native_adapter.AdapterCall_CancelWithdrawal() =>
      Feature.withdrawalCancellation,
    native_adapter.AdapterCall_Deposits() => Feature.depositHistory,
    native_adapter.AdapterCall_Withdrawals() => Feature.withdrawalHistory,
    native_adapter.AdapterCall_OpenOrders() => Feature.openOrders,
    native_adapter.AdapterCall_Order() ||
    native_adapter.AdapterCall_OrderByClientId() ||
    native_adapter.AdapterCall_OrdersByIds() ||
    native_adapter.AdapterCall_OrderHistory() => Feature.orderHistory,
    native_adapter.AdapterCall_PlaceOrder() ||
    native_adapter.AdapterCall_CancelOrder() ||
    native_adapter.AdapterCall_CancelOrderByClientId() ||
    native_adapter.AdapterCall_CancelOrders() => Feature.trading,
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
