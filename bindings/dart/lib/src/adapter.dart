import 'errors.dart';
import 'generated_adapter.dart';
import 'models.dart';
import 'stream.dart';

/// 사용자 정의 거래소 어댑터의 공개 계약입니다.
///
/// 일반적인 구현은 [AdapterBase]를 확장하고 [exchange]와 [features]만
/// 필수로 제공합니다. 선택 기능의 기본 메서드는 [UnsupportedError]를
/// 반환합니다.
abstract interface class Adapter implements GeneratedAdapterContract {}

/// 구현하지 않은 메서드가 [UnsupportedError]를 반환하는 기본 클래스입니다.
abstract base class AdapterBase extends GeneratedAdapterDefaults
    implements Adapter {
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
