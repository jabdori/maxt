import 'models.dart';

/// Investment warning and caution information for one Upbit market.
final class UpbitMarketEvent {
  UpbitMarketEvent({
    required this.market,
    this.warning = false,
    Iterable<String> cautions = const [],
  }) : cautions = List.unmodifiable(cautions);

  final Market market;
  final bool warning;
  final List<String> cautions;
}

/// Native investment-caution flag for one Bithumb market.
final class BithumbMarketWarning {
  const BithumbMarketWarning({required this.market, required this.warning});

  final Market market;
  final String warning;
}

/// Active alert for one Bithumb market.
final class BithumbMarketAlert {
  const BithumbMarketAlert({
    required this.market,
    required this.kind,
    required this.step,
    required this.endsAt,
  });

  final Market market;
  final String kind;
  final BithumbAlertStep step;
  final Timestamp endsAt;
}

/// Price, quantity, and notional constraints for one Binance Spot symbol.
final class BinanceSymbolFilters {
  const BinanceSymbolFilters({
    required this.symbol,
    this.tickSize,
    this.minPrice,
    this.maxPrice,
    this.stepSize,
    this.minQuantity,
    this.maxQuantity,
    this.minNotional,
  });

  final String symbol;
  final Decimal? tickSize;
  final Decimal? minPrice;
  final Decimal? maxPrice;
  final Decimal? stepSize;
  final Decimal? minQuantity;
  final Decimal? maxQuantity;
  final Decimal? minNotional;
}

/// Provider-specific details for a Binance Spot order.
final class BinanceSpotOrderDetail {
  const BinanceSpotOrderDetail({
    required this.order,
    required this.clientOrderId,
    required this.orderType,
    required this.timeInForce,
    required this.filledQuoteQuantity,
    this.updatedAt,
  });

  final Order order;
  final String clientOrderId;
  final String orderType;
  final String timeInForce;
  final Decimal filledQuoteQuantity;
  final Timestamp? updatedAt;
}

/// Non-funding ledger entry for an entire Hyperliquid account.
final class HyperliquidLedgerEntry {
  const HyperliquidLedgerEntry({
    required this.kind,
    required this.time,
    required this.hash,
    this.asset,
    this.amount,
    this.fee,
    this.counterparty,
  });

  final HyperliquidLedgerKind kind;
  final Timestamp time;
  final String hash;
  final String? asset;
  final Decimal? amount;
  final Decimal? fee;
  final String? counterparty;
}

/// Current price, funding, and order-precision information for a Hyperliquid market.
final class HyperliquidAssetContext {
  const HyperliquidAssetContext({
    required this.sizeDecimals,
    required this.priceDecimals,
    this.midPrice,
    this.markPrice,
    this.oraclePrice,
    this.fundingRate,
    this.openInterest,
  });

  final Decimal? midPrice;
  final Decimal? markPrice;
  final Decimal? oraclePrice;
  final Decimal? fundingRate;
  final Decimal? openInterest;
  final int sizeDecimals;
  final int priceDecimals;
}
