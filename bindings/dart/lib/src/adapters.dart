import 'dart:async';
import 'dart:math';
import 'dart:typed_data';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart'
    show PlatformInt64;

import 'adapter.dart';
import 'errors.dart';
import 'models.dart';
import 'platform_int64.dart';
import 'providers.dart';
import 'runtime.dart';
import 'rust/api.dart' as native;
import 'rust/adapter.dart' as native_adapter;
import 'rust/wire.dart' as wire;
import 'rust/stream.dart' as native_stream;
import 'stream.dart';

part 'dart_adapter_bridge.dart';
part 'generated_delegate.dart';
part 'generated_provider_guard.dart';
part 'generated_provider_methods.dart';
part 'generated_wire_converters.dart';
part 'native_conversion.dart';

/// Package-internal registry that combines a Dart Adapter stream with Rust cancellation.
final class DartStreamRegistry {
  final Map<String, _DartStreamCloser> _active = {};
  final Set<String> _pendingCancellation = {};

  int get activeStreamCount => _active.length;
  int get pendingCancellationCount => _pendingCancellation.length;

  Future<bool> register(String id, Future<void> Function() close) async {
    if (_active.containsKey(id)) {
      throw StateError('stream $id is already registered');
    }
    final entry = _DartStreamCloser(close);
    _active[id] = entry;
    if (!_pendingCancellation.remove(id)) return true;

    try {
      await entry.close();
      return false;
    } finally {
      if (identical(_active[id], entry)) _active.remove(id);
      _pendingCancellation.remove(id);
    }
  }

  Future<void> cancel(String id) async {
    final entry = _active[id];
    if (entry == null) {
      _pendingCancellation.add(id);
      return;
    }
    try {
      await entry.close();
    } finally {
      if (identical(_active[id], entry)) _active.remove(id);
      _pendingCancellation.remove(id);
    }
  }

  Future<void> finish(String id) async {
    final entry = _active.remove(id);
    _pendingCancellation.remove(id);
    if (entry == null) {
      return;
    }
    if (!entry.isClosing) await entry.close();
  }

  void forgetPending(String id) => _pendingCancellation.remove(id);
}

final class _DartStreamCloser {
  _DartStreamCloser(this._close);

  final Future<void> Function() _close;
  Future<void>? _closing;

  bool get isClosing => _closing != null;

  Future<void> close() {
    final closing = _closing;
    if (closing != null) return closing;

    final completer = Completer<void>();
    _closing = completer.future;
    Future.sync(
      _close,
    ).then(completer.complete, onError: completer.completeError);
    return completer.future;
  }
}

abstract base class _NativeAdapterBase
    implements Adapter, NativeHandleProvider {
  _NativeAdapterBase(native.NativeClient handle)
    : _handle = handle,
      exchange = _exchangeFromWire(handle.exchange()),
      features = Set.unmodifiable(
        Feature.values.where(
          (feature) => handle.supports(feature: _featureToWire(feature)),
        ),
      );

  final native.NativeClient _handle;

  @override
  native.NativeClient get nativeHandle => _handle;

  @override
  final Exchange exchange;

  @override
  final Set<Feature> features;

  @override
  bool supports(Feature feature) =>
      _handle.supports(feature: _featureToWire(feature));

  @override
  Future<List<MarketInfo>> markets(MarketKind kind) => _nativeFuture(
    () => _handle.markets(kind: _marketKindToWire(kind)),
  ).then((values) => values.map(_marketInfoFromWire).toList(growable: false));

  @override
  Future<List<Trade>> trades(Market market, [int? limit]) => _nativeFuture(
    () => _handle.trades(
      market: _marketToWire(market),
      limit: checkedUint32(limit, field: 'limit'),
    ),
  ).then((values) => values.map(_tradeFromWire).toList(growable: false));

  @override
  Future<OrderBook> orderBook(Market market, [int? depth]) => _nativeFuture(
    () => _handle.orderBook(
      market: _marketToWire(market),
      depth: checkedUint32(depth, field: 'depth'),
    ),
  ).then(_orderBookFromWire);

  @override
  Future<Ticker> ticker(Market market) => _nativeFuture(
    () => _handle.ticker(market: _marketToWire(market)),
  ).then(_tickerFromWire);

  @override
  Future<List<Candle>> candles(CandleRequest request) => _nativeFuture(
    () => _handle.candles(request: _candleRequestToWire(request)),
  ).then((values) => values.map(_candleFromWire).toList(growable: false));

  @override
  Future<MarketStream> subscribe(
    Subscription subscription,
    StreamConfig config,
  ) async {
    if (subscription.markets.isEmpty) {
      throw const InvalidRequestError(
        field: 'markets',
        detail: 'a subscription needs at least one market',
      );
    }
    if (subscription.feeds.isEmpty) {
      throw const InvalidRequestError(
        field: 'feeds',
        detail: 'a subscription needs at least one feed',
      );
    }
    final handle = await _nativeFuture(
      () => _handle.subscribe(
        subscription: _subscriptionToWire(subscription),
        config: _streamConfigToWire(config),
      ),
    );
    return MarketStream(
      _nativeMarketItems(handle),
      onClose: () => _nativeFuture(
        () => native.nativeMarketSubscriptionClose(subscription: handle),
      ),
    );
  }

  @override
  Future<List<Balance>> balances() => _nativeFuture(
    _handle.balances,
  ).then((values) => values.map(_balanceFromWire).toList(growable: false));

  @override
  Future<OrderRules> orderRules(Market market) => _nativeFuture(
    () => _handle.orderRules(market: _marketToWire(market)),
  ).then(_orderRulesFromWire);

  @override
  Future<List<AssetNetwork>> assetNetworks(String asset) => _nativeFuture(
    () => _handle.assetNetworks(asset: asset),
  ).then((values) => values.map(_assetNetworkFromWire).toList(growable: false));

  @override
  Future<List<DepositAddressEntry>> depositAddresses() =>
      _nativeFuture(_handle.depositAddresses).then(
        (values) =>
            values.map(_depositAddressEntryFromWire).toList(growable: false),
      );

  @override
  Future<DepositAddress> depositAddress(DepositAddressRequest request) =>
      _nativeFuture(
        () => _handle.depositAddress(
          request: _depositAddressRequestToWire(request),
        ),
      ).then(_depositAddressFromWire);

  @override
  Future<DepositAddress> createDepositAddress(DepositAddressRequest request) =>
      _nativeFuture(
        () => _handle.createDepositAddress(
          request: _depositAddressRequestToWire(request),
        ),
      ).then(_depositAddressFromWire);

  @override
  Future<WithdrawalQuote> prepareWithdrawal(WithdrawRequest request) =>
      _nativeFuture(
        () =>
            _handle.prepareWithdrawal(request: _withdrawRequestToWire(request)),
      ).then(_withdrawalQuoteFromWire);

  @override
  Future<Withdrawal> withdraw(WithdrawRequest request) => _nativeFuture(
    () => _handle.withdraw(request: _withdrawRequestToWire(request)),
  ).then(_withdrawalFromWire);

  @override
  Future<Deposit> deposit(TransferLookupRequest request) => _nativeFuture(
    () => _handle.deposit(request: _transferLookupRequestToWire(request)),
  ).then(_depositFromWire);

  @override
  Future<Withdrawal> withdrawal(TransferLookupRequest request) => _nativeFuture(
    () => _handle.withdrawal(request: _transferLookupRequestToWire(request)),
  ).then(_withdrawalFromWire);

  @override
  Future<void> cancelWithdrawal(String withdrawalId) =>
      _nativeFuture(() => _handle.cancelWithdrawal(withdrawalId: withdrawalId));

  @override
  Future<Page<Deposit>> deposits(TransferHistoryRequest request) =>
      _nativeFuture(
        () => _handle.deposits(request: _transferHistoryRequestToWire(request)),
      ).then(_depositPageFromWire);

  @override
  Future<Page<Withdrawal>> withdrawals(TransferHistoryRequest request) =>
      _nativeFuture(
        () => _handle.withdrawals(
          request: _transferHistoryRequestToWire(request),
        ),
      ).then(_withdrawalPageFromWire);

  @override
  Future<AccountStream> subscribeAccount(StreamConfig config) async {
    final handle = await _nativeFuture(
      () => _handle.subscribeAccount(config: _streamConfigToWire(config)),
    );
    return AccountStream(
      _nativeAccountItems(handle),
      onClose: () => _nativeFuture(
        () => native.nativeAccountSubscriptionClose(subscription: handle),
      ),
    );
  }

  @override
  Future<List<Order>> openOrders([Market? market]) => _nativeFuture(
    () => _handle.openOrders(
      market: market == null ? null : _marketToWire(market),
    ),
  ).then((values) => values.map(_orderFromWire).toList(growable: false));

  @override
  Future<Order> order(Market market, String orderId) => _nativeFuture(
    () => _handle.order(market: _marketToWire(market), orderId: orderId),
  ).then(_orderFromWire);

  @override
  Future<Order> orderByClientId(Market market, String clientId) =>
      _nativeFuture(
        () => _handle.orderByClientId(
          market: _marketToWire(market),
          clientId: clientId,
        ),
      ).then(_orderFromWire);

  @override
  Future<List<Order>> ordersByIds(OrderLookupRequest request) => _nativeFuture(
    () => _handle.ordersByIds(request: _orderLookupRequestToWire(request)),
  ).then((values) => values.map(_orderFromWire).toList(growable: false));

  @override
  Future<Page<Order>> orderHistory(OrderHistoryRequest request) =>
      _nativeFuture(
        () =>
            _handle.orderHistory(request: _orderHistoryRequestToWire(request)),
      ).then(_orderPageFromWire);

  @override
  Future<Order> placeOrder(OrderRequest request) => _nativeFuture(
    () => _handle.placeOrder(request: _orderRequestToWire(request)),
  ).then(_orderFromWire);

  @override
  Future<void> cancelOrder(Market market, String orderId) => _nativeFuture(
    () => _handle.cancelOrder(market: _marketToWire(market), orderId: orderId),
  );

  @override
  Future<void> cancelOrderByClientId(Market market, String clientId) =>
      _nativeFuture(
        () => _handle.cancelOrderByClientId(
          market: _marketToWire(market),
          clientId: clientId,
        ),
      );

  @override
  Future<CancelOrdersResult> cancelOrders(CancelOrdersRequest request) =>
      _nativeFuture(
        () =>
            _handle.cancelOrders(request: _cancelOrdersRequestToWire(request)),
      ).then(_cancelOrdersResultFromWire);

  @override
  Future<List<Position>> positions([Market? market]) => _nativeFuture(
    () => _handle.positions(
      market: market == null ? null : _marketToWire(market),
    ),
  ).then((values) => values.map(_positionFromWire).toList(growable: false));

  @override
  Future<MarginSummary> marginSummary() =>
      _nativeFuture(_handle.marginSummary).then(_marginSummaryFromWire);

  @override
  Future<Page<FundingRate>> fundingRates(HistoryRequest request) =>
      _nativeFuture(
        () => _handle.fundingRates(request: _historyRequestToWire(request)),
      ).then(_fundingRatePageFromWire);

  @override
  Future<Page<FundingPayment>> fundingPayments(HistoryRequest request) =>
      _nativeFuture(
        () => _handle.fundingPayments(request: _historyRequestToWire(request)),
      ).then(_fundingPaymentPageFromWire);

  @override
  Future<void> setMargin(MarginRequest request) => _nativeFuture(
    () => _handle.setMargin(request: _marginRequestToWire(request)),
  );
}

/// Upbit Spot exchange adapter.
final class UpbitAdapter extends _NativeAdapterBase {
  /// Creates an Upbit Spot adapter for the default Korea region.
  factory UpbitAdapter({String? accessKey, String? secretKey}) =>
      UpbitAdapter.withRegion(
        UpbitRegion.korea,
        accessKey: accessKey,
        secretKey: secretKey,
      );

  /// Creates an Upbit Spot adapter for [region].
  factory UpbitAdapter.withRegion(
    UpbitRegion region, {
    String? accessKey,
    String? secretKey,
  }) {
    validateBrowserCredentials(accessKey, secretKey);
    return UpbitAdapter._(
      _nativeSync(
        () => native.NativeClient.upbit(
          region: _upbitRegionToWire(region),
          accessKey: accessKey,
          secretKey: secretKey,
        ),
      ),
      region,
    );
  }

  UpbitAdapter._(super.handle, this.region);

  /// Upbit region used by this adapter.
  final UpbitRegion region;

  /// Creates a new adapter with credentials for the same region.
  UpbitAdapter withCredentials(String accessKey, String secretKey) =>
      UpbitAdapter.withRegion(
        region,
        accessKey: accessKey,
        secretKey: secretKey,
      );
}

/// Bithumb Spot exchange adapter.
final class BithumbAdapter extends _NativeAdapterBase {
  /// Creates a Bithumb Spot adapter.
  factory BithumbAdapter({String? accessKey, String? secretKey}) {
    validateBrowserCredentials(accessKey, secretKey);
    return BithumbAdapter._(
      _nativeSync(
        () => native.NativeClient.bithumb(
          accessKey: accessKey,
          secretKey: secretKey,
        ),
      ),
    );
  }

  BithumbAdapter._(super.handle);

  /// Creates a new Bithumb adapter with credentials.
  BithumbAdapter withCredentials(String accessKey, String secretKey) =>
      BithumbAdapter(accessKey: accessKey, secretKey: secretKey);
}

/// Binance Spot or USD-M perpetual-futures exchange adapter.
final class BinanceAdapter extends _NativeAdapterBase {
  /// Creates a Binance Spot adapter.
  factory BinanceAdapter.spot({String? apiKey, String? secretKey}) {
    validateBrowserCredentials(apiKey, secretKey);
    return BinanceAdapter._(
      _nativeSync(
        () => native.NativeClient.binanceSpot(
          apiKey: apiKey,
          secretKey: secretKey,
        ),
      ),
      BinanceMarket.spot,
    );
  }

  /// Creates a Binance USD-M perpetual-futures adapter.
  factory BinanceAdapter.usdMFutures({String? apiKey, String? secretKey}) {
    validateBrowserCredentials(apiKey, secretKey);
    return BinanceAdapter._(
      _nativeSync(
        () => native.NativeClient.binanceUsdMFutures(
          apiKey: apiKey,
          secretKey: secretKey,
        ),
      ),
      BinanceMarket.usdMFutures,
    );
  }

  BinanceAdapter._(super.handle, this.venue);

  /// Binance venue used by this adapter.
  final BinanceMarket venue;

  /// Creates a new adapter with credentials for the same venue.
  BinanceAdapter withCredentials(String apiKey, String secretKey) =>
      switch (venue) {
        BinanceMarket.spot => BinanceAdapter.spot(
          apiKey: apiKey,
          secretKey: secretKey,
        ),
        BinanceMarket.usdMFutures => BinanceAdapter.usdMFutures(
          apiKey: apiKey,
          secretKey: secretKey,
        ),
      };
}

/// Binance USD-M listen key that cannot be created through a constructor.
final class BinanceListenKey {
  /// Wraps an internally issued listen key.
  const BinanceListenKey._(this._handle);

  final native.WireBinanceListenKey _handle;

  /// Secret listen-key value used only for WebSocket lifecycle calls.
  String get value => _handle.value;
}

/// Hyperliquid mainnet or testnet adapter.
final class HyperliquidAdapter extends _NativeAdapterBase {
  /// Creates a Hyperliquid mainnet adapter.
  factory HyperliquidAdapter({String? address, String? privateKey}) {
    validateBrowserCredentials(null, privateKey);
    return HyperliquidAdapter._(
      _nativeSync(
        () => native.NativeClient.hyperliquid(
          testnet: false,
          address: address,
          privateKey: privateKey,
        ),
      ),
      false,
    );
  }

  /// Creates a Hyperliquid testnet adapter.
  factory HyperliquidAdapter.testnet({String? address, String? privateKey}) {
    validateBrowserCredentials(null, privateKey);
    return HyperliquidAdapter._(
      _nativeSync(
        () => native.NativeClient.hyperliquid(
          testnet: true,
          address: address,
          privateKey: privateKey,
        ),
      ),
      true,
    );
  }

  HyperliquidAdapter._(super.handle, this.isTestnet);

  /// Whether this adapter uses testnet.
  final bool isTestnet;

  /// Creates a new adapter with a wallet address and private key for the same network.
  HyperliquidAdapter withWallet(String address, String privateKey) => isTestnet
      ? HyperliquidAdapter.testnet(address: address, privateKey: privateKey)
      : HyperliquidAdapter(address: address, privateKey: privateKey);
}
