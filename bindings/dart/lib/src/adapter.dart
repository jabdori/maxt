import 'errors.dart';
import 'generated_adapter.dart';
import 'models.dart';
import 'stream.dart';

/// Public contract for a custom exchange adapter.
///
/// A typical implementation extends [AdapterBase] and only must provide
/// [exchange] and [features]. Default optional methods return [UnsupportedError].
abstract interface class Adapter implements GeneratedAdapterContract {}

/// Base class whose unimplemented methods return [UnsupportedError].
abstract base class AdapterBase extends GeneratedAdapterDefaults
    implements Adapter {
  /// Validates an unsupported base subscription call before completing with an error.
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
    final feature = switch (subscription.feeds.first.kind) {
      FeedKind.orderBook => Feature.orderBookStream,
      FeedKind.ticker => Feature.tickerStream,
      FeedKind.candles => Feature.candleStream,
      FeedKind.trades => Feature.tradeStream,
    };
    return Future.error(
      UnsupportedError(
        feature: feature,
        exchange: exchange,
        detail: '${exchange.id} has no endpoint for ${feature.wireName}',
      ),
    );
  }
}
