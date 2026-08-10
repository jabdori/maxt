import 'dart:async';
import 'dart:math';

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

/// Dart Adapter stream과 Rust cancellation을 결합하는 package 내부 등록소입니다.
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
  Future<List<AssetNetwork>> assetNetworks(String asset) => _nativeFuture(
    () => _handle.assetNetworks(asset: asset),
  ).then((values) => values.map(_assetNetworkFromWire).toList(growable: false));

  @override
  Future<DepositAddress> depositAddress(DepositAddressRequest request) =>
      _nativeFuture(
        () => _handle.depositAddress(
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

/// Upbit 현물 거래소 어댑터입니다.
final class UpbitAdapter extends _NativeAdapterBase {
  factory UpbitAdapter({String? accessKey, String? secretKey}) =>
      UpbitAdapter.withRegion(
        UpbitRegion.korea,
        accessKey: accessKey,
        secretKey: secretKey,
      );

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

  final UpbitRegion region;

  UpbitAdapter withCredentials(String accessKey, String secretKey) =>
      UpbitAdapter.withRegion(
        region,
        accessKey: accessKey,
        secretKey: secretKey,
      );
}

/// Bithumb 현물 거래소 어댑터입니다.
final class BithumbAdapter extends _NativeAdapterBase {
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

  BithumbAdapter withCredentials(String accessKey, String secretKey) =>
      BithumbAdapter(accessKey: accessKey, secretKey: secretKey);
}

/// Binance 현물 또는 USD-M 무기한 선물 어댑터입니다.
final class BinanceAdapter extends _NativeAdapterBase {
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

  final BinanceMarket venue;

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

/// 생성자로 만들 수 없는 Binance USD-M listen key입니다.
final class BinanceListenKey {
  const BinanceListenKey._(this._handle);

  final native.WireBinanceListenKey _handle;

  String get value => _handle.value;
}

/// Hyperliquid mainnet 또는 testnet 어댑터입니다.
final class HyperliquidAdapter extends _NativeAdapterBase {
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

  final bool isTestnet;

  HyperliquidAdapter withWallet(String address, String privateKey) => isTestnet
      ? HyperliquidAdapter.testnet(address: address, privateKey: privateKey)
      : HyperliquidAdapter(address: address, privateKey: privateKey);
}
