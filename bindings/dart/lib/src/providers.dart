import 'models.dart';

/// 한 Upbit 시장의 투자 경고·주의 정보입니다.
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

/// 한 Bithumb 시장의 원본 투자 유의 플래그입니다.
final class BithumbMarketWarning {
  const BithumbMarketWarning({required this.market, required this.warning});

  final Market market;
  final String warning;
}

/// 한 Bithumb 시장의 활성 경보입니다.
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

/// 한 Binance 현물 심볼의 가격·수량·명목가 제약입니다.
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

/// Binance 현물 주문의 공급자 전용 상세 정보입니다.
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

/// Hyperliquid 계정 전체의 비펀딩 원장 항목입니다.
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

/// Hyperliquid 시장의 현재 가격·펀딩·주문 정밀도 정보입니다.
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
