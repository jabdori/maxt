String _asciiUpper(String value) => String.fromCharCodes(
  value.codeUnits.map(
    (code) => code >= 0x61 && code <= 0x7a ? code - 0x20 : code,
  ),
);

/// maxt가 지원하는 거래소입니다.
enum Exchange { upbit, bithumb, binance, hyperliquid }

extension ExchangeName on Exchange {
  /// 로그와 오류에 쓰는 안정적인 소문자 식별자입니다.
  String get id => name;

  /// 사용자에게 표시할 거래소 이름입니다.
  String get displayName => switch (this) {
    Exchange.upbit => 'Upbit',
    Exchange.bithumb => 'Bithumb',
    Exchange.binance => 'Binance',
    Exchange.hyperliquid => 'Hyperliquid',
  };
}

/// 어댑터가 제공할 수 있는 기능입니다.
enum Feature {
  markets,
  trades,
  orderBook,
  ticker,
  candles,
  tradeStream,
  orderBookStream,
  tickerStream,
  candleStream,
  balances,
  openOrders,
  accountStream,
  trading,
  positions,
  margin,
  fundingRates,
  fundingPayments,
  marginConfig,
  reduceOnlyOrders,
}

extension FeatureProperties on Feature {
  /// Dart/Rust 경계에서 쓰는 snake_case 이름입니다.
  String get wireName => switch (this) {
    Feature.orderBook => 'order_book',
    Feature.tradeStream => 'trade_stream',
    Feature.orderBookStream => 'order_book_stream',
    Feature.tickerStream => 'ticker_stream',
    Feature.candleStream => 'candle_stream',
    Feature.openOrders => 'open_orders',
    Feature.accountStream => 'account_stream',
    Feature.fundingRates => 'funding_rates',
    Feature.fundingPayments => 'funding_payments',
    Feature.marginConfig => 'margin_config',
    Feature.reduceOnlyOrders => 'reduce_only_orders',
    _ => name,
  };

  /// 이 기능이 인증 정보를 요구하는지 여부입니다.
  bool get needsCredentials => switch (this) {
    Feature.markets ||
    Feature.trades ||
    Feature.orderBook ||
    Feature.ticker ||
    Feature.candles ||
    Feature.tradeStream ||
    Feature.orderBookStream ||
    Feature.tickerStream ||
    Feature.candleStream ||
    Feature.fundingRates => false,
    _ => true,
  };

  /// 이 기능이 파생상품 시장에서만 의미가 있는지 여부입니다.
  bool get isDerivativesOnly => switch (this) {
    Feature.positions ||
    Feature.margin ||
    Feature.fundingRates ||
    Feature.fundingPayments ||
    Feature.marginConfig ||
    Feature.reduceOnlyOrders => true,
    _ => false,
  };
}

/// 부동소수점 변환 없이 문자열을 보존하는 정확한 소수입니다.
final class Decimal implements Comparable<Decimal> {
  Decimal._(this._value, this._coefficient, this._scale);

  /// 일반 표기법이나 과학 표기법의 유한 소수를 읽습니다.
  factory Decimal.parse(String value) {
    final match = _pattern.firstMatch(value);
    if (match == null) {
      throw FormatException('Invalid decimal', value);
    }

    final integer = match.group(2) ?? '';
    final fraction = match.group(3) ?? match.group(4) ?? '';
    final exponentText = match.group(5);
    final exponent = exponentText == null
        ? BigInt.zero
        : BigInt.parse(exponentText);
    if (exponentText != null) {
      final point = exponent + BigInt.from(integer.length);
      if (point < -_maxPointShift || point > _maxPointShift) {
        throw FormatException(
          'Decimal scientific notation is too large',
          value,
        );
      }
    }

    final digits = '$integer$fraction';
    var coefficient = BigInt.parse(digits);
    if (match.group(1) == '-') coefficient = -coefficient;
    final expandedScale = BigInt.from(fraction.length) - exponent;
    if (!_isRepresentable(coefficient, expandedScale)) {
      throw FormatException('Decimal is outside the Rust Decimal range', value);
    }

    var scale = expandedScale.toInt();

    if (coefficient == BigInt.zero) {
      scale = 0;
    } else {
      while (coefficient.remainder(BigInt.from(10)) == BigInt.zero) {
        coefficient ~/= BigInt.from(10);
        scale--;
      }
    }
    return Decimal._(value, coefficient, scale);
  }

  static final RegExp _pattern = RegExp(
    r'^([+-]?)(?:(\d+)(?:\.(\d*))?|\.(\d+))(?:[eE]([+-]?\d+))?$',
  );

  static final Decimal zero = Decimal.parse('0');
  static final Decimal one = Decimal.parse('1');
  static final BigInt _maxCoefficient = BigInt.parse(
    '79228162514264337593543950335',
  );
  static final BigInt _ten = BigInt.from(10);
  static final BigInt _maxPointShift = BigInt.from(64);
  static final BigInt _maxScaleBigInt = BigInt.from(_maxScale);
  static const int _maxScale = 28;

  final String _value;
  final BigInt _coefficient;
  final int _scale;

  bool get isZero => _coefficient == BigInt.zero;

  Decimal operator +(Decimal other) => _addOrSubtract(other, subtract: false);

  Decimal operator -(Decimal other) => _addOrSubtract(other, subtract: true);

  bool operator <(Decimal other) => compareTo(other) < 0;

  bool operator <=(Decimal other) => compareTo(other) <= 0;

  bool operator >(Decimal other) => compareTo(other) > 0;

  bool operator >=(Decimal other) => compareTo(other) >= 0;

  @override
  int compareTo(Decimal other) {
    final scale = _scale > other._scale ? _scale : other._scale;
    final left = _coefficient * _pow10(scale - _scale);
    final right = other._coefficient * _pow10(scale - other._scale);
    return left.compareTo(right);
  }

  @override
  String toString() => _value;

  @override
  bool operator ==(Object other) =>
      other is Decimal &&
      _coefficient == other._coefficient &&
      _scale == other._scale;

  @override
  int get hashCode => Object.hash(_coefficient, _scale);

  Decimal _addOrSubtract(Decimal other, {required bool subtract}) {
    final scale = _scale > other._scale ? _scale : other._scale;
    final left = _coefficient * _pow10(scale - _scale);
    final right = other._coefficient * _pow10(scale - other._scale);
    return _fromArithmetic(subtract ? left - right : left + right, scale);
  }

  Decimal _half() {
    if (_coefficient.isEven) {
      return _fromArithmetic(_coefficient ~/ BigInt.two, _scale);
    }
    return _fromArithmetic(_coefficient * BigInt.from(5), _scale + 1);
  }

  static bool _isRepresentable(BigInt coefficient, BigInt scale) {
    if (scale > _maxScaleBigInt) return false;
    if (scale >= BigInt.zero) return coefficient.abs() <= _maxCoefficient;
    if (coefficient == BigInt.zero) return true;

    final shift = -scale;
    if (shift > _maxScaleBigInt) return false;
    return coefficient.abs() <= _maxCoefficient ~/ _pow10(shift.toInt());
  }

  static Decimal _fromArithmetic(BigInt coefficient, int scale) {
    if (coefficient == BigInt.zero) return zero;

    if (scale < 0) {
      final shift = -scale;
      if (shift > _maxScale ||
          coefficient.abs() > _maxCoefficient ~/ _pow10(shift)) {
        throw RangeError('Decimal arithmetic overflow');
      }
      coefficient *= _pow10(shift);
      scale = 0;
    }

    final minimumDrop = scale > _maxScale ? scale - _maxScale : 0;
    for (var drop = minimumDrop; drop <= scale; drop++) {
      final rounded = _roundHalfEven(coefficient, drop);
      if (rounded.abs() <= _maxCoefficient) {
        return _canonical(rounded, scale - drop);
      }
    }
    throw RangeError('Decimal arithmetic overflow');
  }

  static BigInt _roundHalfEven(BigInt coefficient, int digits) {
    if (digits == 0) return coefficient;

    final divisor = _pow10(digits);
    final negative = coefficient.isNegative;
    final absolute = coefficient.abs();
    var quotient = absolute ~/ divisor;
    final remainder = absolute.remainder(divisor);
    final doubled = remainder * BigInt.two;
    if (doubled > divisor || doubled == divisor && quotient.isOdd) {
      quotient += BigInt.one;
    }
    return negative ? -quotient : quotient;
  }

  static Decimal _canonical(BigInt coefficient, int scale) {
    if (coefficient == BigInt.zero) return zero;
    while (coefficient.remainder(_ten) == BigInt.zero) {
      coefficient ~/= _ten;
      scale--;
    }
    return Decimal._(_plain(coefficient, scale), coefficient, scale);
  }

  static String _plain(BigInt coefficient, int scale) {
    final sign = coefficient.isNegative ? '-' : '';
    final digits = coefficient.abs().toString();
    if (scale <= 0) {
      return '$sign${digits.padRight(digits.length - scale, '0')}';
    }
    if (digits.length <= scale) {
      return '${sign}0.${digits.padLeft(scale, '0')}';
    }
    final point = digits.length - scale;
    return '$sign${digits.substring(0, point)}.${digits.substring(point)}';
  }

  static BigInt _pow10(int exponent) => _ten.pow(exponent);
}

/// Unix epoch 기준 UTC 나노초로 표현한 시각입니다.
final class Timestamp implements Comparable<Timestamp> {
  const Timestamp._(this.nanosecondsSinceEpoch);

  /// Unix epoch 기준 나노초에서 시각을 만듭니다.
  factory Timestamp.fromNanoseconds(int nanoseconds) {
    if (nanoseconds < _min || nanoseconds > _max) {
      throw RangeError.range(nanoseconds, _min, _max, 'nanoseconds');
    }
    return Timestamp._(nanoseconds);
  }

  /// Unix epoch 기준 마이크로초에서 시각을 만듭니다.
  factory Timestamp.fromMicroseconds(int microseconds) =>
      _fromScaled(microseconds, 1000);

  /// Unix epoch 기준 밀리초에서 시각을 만듭니다.
  factory Timestamp.fromMilliseconds(int milliseconds) =>
      _fromScaled(milliseconds, 1000000);

  /// Unix epoch 기준 초에서 시각을 만듭니다.
  factory Timestamp.fromSeconds(int seconds) =>
      _fromScaled(seconds, 1000000000);

  /// 현재 시스템 시각입니다.
  factory Timestamp.now() {
    final microseconds = DateTime.now().microsecondsSinceEpoch;
    return microseconds <= 0 ? zero : _fromScaled(microseconds, 1000);
  }

  static const int _min = -9223372036854775808;
  static const int _max = 9223372036854775807;
  static final BigInt _minBigInt = BigInt.from(_min);
  static final BigInt _maxBigInt = BigInt.from(_max);
  static const Timestamp zero = Timestamp._(0);

  final int nanosecondsSinceEpoch;

  /// Unix epoch 기준 밀리초를 epoch(0) 방향으로 절삭한 값입니다.
  int get millisecondsSinceEpoch => nanosecondsSinceEpoch ~/ 1000000;

  /// Unix epoch 기준 초를 epoch(0) 방향으로 절삭한 값입니다.
  int get secondsSinceEpoch => nanosecondsSinceEpoch ~/ 1000000000;

  static Timestamp _fromScaled(int value, int scale) {
    final nanoseconds = BigInt.from(value) * BigInt.from(scale);
    if (nanoseconds < _minBigInt) return const Timestamp._(_min);
    if (nanoseconds > _maxBigInt) return const Timestamp._(_max);
    return Timestamp._(nanoseconds.toInt());
  }

  static Timestamp? _checked(BigInt nanoseconds) {
    if (nanoseconds < _minBigInt || nanoseconds > _maxBigInt) return null;
    return Timestamp._(nanoseconds.toInt());
  }

  static bool _contains(BigInt value) =>
      value >= _minBigInt && value <= _maxBigInt;

  @override
  int compareTo(Timestamp other) =>
      nanosecondsSinceEpoch.compareTo(other.nanosecondsSinceEpoch);

  @override
  bool operator ==(Object other) =>
      other is Timestamp &&
      nanosecondsSinceEpoch == other.nanosecondsSinceEpoch;

  @override
  int get hashCode => nanosecondsSinceEpoch.hashCode;
}

/// 거래 가능한 상품의 종류입니다.
enum MarketKind { spot, perpetual }

extension MarketKindProperties on MarketKind {
  bool get isDerivative => this == MarketKind.perpetual;
}

/// 하나의 거래소 시장을 식별합니다.
final class Market {
  Market(Exchange exchange, MarketKind kind, String base, String quote)
    : exchange = exchange,
      kind = kind,
      base = _asciiUpper(base),
      quote = _asciiUpper(quote);

  factory Market.spot(Exchange exchange, String base, String quote) =>
      Market(exchange, MarketKind.spot, base, quote);

  factory Market.perpetual(Exchange exchange, String base, String quote) =>
      Market(exchange, MarketKind.perpetual, base, quote);

  final Exchange exchange;
  final MarketKind kind;
  final String base;
  final String quote;

  @override
  String toString() =>
      '${exchange.id}:$base/$quote${kind == MarketKind.perpetual ? ':perp' : ''}';

  @override
  bool operator ==(Object other) =>
      other is Market &&
      exchange == other.exchange &&
      kind == other.kind &&
      base == other.base &&
      quote == other.quote;

  @override
  int get hashCode => Object.hash(exchange, kind, base, quote);
}

/// 캔들 구간입니다.
enum Interval {
  sec1,
  min1,
  min3,
  min5,
  min15,
  min30,
  hour1,
  hour2,
  hour4,
  hour8,
  hour12,
  day1,
  day3,
  week1,
  month1,
}

extension IntervalProperties on Interval {
  /// 고정된 구간의 초 단위 길이입니다. 달력 월인 [Interval.month1]은 `null`입니다.
  int? get seconds => switch (this) {
    Interval.sec1 => 1,
    Interval.min1 => 60,
    Interval.min3 => 180,
    Interval.min5 => 300,
    Interval.min15 => 900,
    Interval.min30 => 1800,
    Interval.hour1 => 3600,
    Interval.hour2 => 7200,
    Interval.hour4 => 14400,
    Interval.hour8 => 28800,
    Interval.hour12 => 43200,
    Interval.day1 => 86400,
    Interval.day3 => 259200,
    Interval.week1 => 604800,
    Interval.month1 => null,
  };

  /// [at]에서 [count]개 구간만큼 이동한 시각입니다.
  ///
  /// 고정 구간은 초 단위 길이만큼 나노초를 정확히 이동합니다.
  /// [Interval.month1]은 UTC 달력 월을 사용하고, 대상 월에 없는 일자는
  /// 말일로 보정합니다. 이 보정 때문에 역방향 이동이 원래 시각을 복원하지 못할 수
  /// 있습니다.
  ///
  /// 거래소별 캔들 시작 격자(provider candle grid)는 적용하지 않습니다.
  /// 결과를 [Timestamp]로 표현할 수 없으면 `null`입니다.
  Timestamp? advance(Timestamp at, int count) {
    final countBigInt = BigInt.from(count);

    final fixedSeconds = seconds;
    if (fixedSeconds != null) {
      final span =
          BigInt.from(fixedSeconds) * BigInt.from(1000000000) * countBigInt;
      if (!Timestamp._contains(span)) return null;
      return Timestamp._checked(BigInt.from(at.nanosecondsSinceEpoch) + span);
    }

    if (countBigInt.abs() > BigInt.from(4294967295)) return null;

    var microseconds = at.nanosecondsSinceEpoch ~/ 1000;
    var nanosecondRemainder = at.nanosecondsSinceEpoch.remainder(1000);
    if (nanosecondRemainder < 0) {
      microseconds--;
      nanosecondRemainder += 1000;
    }
    final current = DateTime.fromMicrosecondsSinceEpoch(
      microseconds,
      isUtc: true,
    );
    final monthIndex =
        BigInt.from(current.year * 12 + current.month - 1) + countBigInt;
    if (monthIndex < BigInt.from(1677 * 12) ||
        monthIndex > BigInt.from(2262 * 12 + 11)) {
      return null;
    }

    final targetYear = (monthIndex ~/ BigInt.from(12)).toInt();
    final targetMonth = monthIndex.remainder(BigInt.from(12)).toInt() + 1;
    final lastDay = DateTime.utc(targetYear, targetMonth + 1, 0).day;
    final moved = DateTime.utc(
      targetYear,
      targetMonth,
      current.day > lastDay ? lastDay : current.day,
      current.hour,
      current.minute,
      current.second,
      current.millisecond,
      current.microsecond,
    );
    return Timestamp._checked(
      BigInt.from(moved.microsecondsSinceEpoch) * BigInt.from(1000) +
          BigInt.from(nanosecondRemainder),
    );
  }
}

/// 캔들 이력 조회 조건입니다.
final class CandleRequest {
  const CandleRequest(
    this.market,
    this.interval, {
    this.from,
    this.to,
    this.limit,
  });

  final Market market;
  final Interval interval;
  final Timestamp? from;
  final Timestamp? to;
  final int? limit;
}

/// 느린 스트림 소비자의 버퍼가 가득 찼을 때의 정책입니다.
enum Overflow { backpressure, dropNewest }

/// 스트림 연결과 버퍼 설정입니다.
final class StreamConfig {
  const StreamConfig({
    this.maxReconnectAttempts,
    this.initialReconnectDelayMs = 1000,
    this.maxReconnectDelayMs = 30000,
    this.idleTimeoutMs = 30000,
    this.bufferSize = 4096,
    this.overflow = Overflow.backpressure,
  });

  final int? maxReconnectAttempts;
  final int initialReconnectDelayMs;
  final int maxReconnectDelayMs;
  final int idleTimeoutMs;
  final int bufferSize;
  final Overflow overflow;

  @override
  bool operator ==(Object other) =>
      other is StreamConfig &&
      maxReconnectAttempts == other.maxReconnectAttempts &&
      initialReconnectDelayMs == other.initialReconnectDelayMs &&
      maxReconnectDelayMs == other.maxReconnectDelayMs &&
      idleTimeoutMs == other.idleTimeoutMs &&
      bufferSize == other.bufferSize &&
      overflow == other.overflow;

  @override
  int get hashCode => Object.hash(
    maxReconnectAttempts,
    initialReconnectDelayMs,
    maxReconnectDelayMs,
    idleTimeoutMs,
    bufferSize,
    overflow,
  );
}

/// 공통 시장 거래 상태입니다.
enum MarketStatus { active, paused, delisted, unknown }

/// 시장 목록의 한 항목입니다.
final class MarketInfo {
  MarketInfo({
    required this.market,
    required this.nativeSymbol,
    required this.status,
    this.koreanName,
    this.englishName,
  });

  final Market market;
  final String nativeSymbol;
  final MarketStatus status;
  final String? koreanName;
  final String? englishName;
}

/// 체결 또는 주문의 매수·매도 방향입니다.
enum Side { buy, sell }

extension SideProperties on Side {
  Side get flipped => this == Side.buy ? Side.sell : Side.buy;
}

/// 하나의 체결입니다.
final class Trade {
  Trade({
    required this.market,
    required this.timestamp,
    required this.price,
    required this.quantity,
    required this.takerSide,
    this.id,
  });

  final Market market;
  final Timestamp timestamp;
  final Decimal price;
  final Decimal quantity;
  final Side takerSide;
  final String? id;
}

/// 호가창의 한 가격 단계입니다.
final class Level {
  Level({required this.price, required this.quantity});

  final Decimal price;
  final Decimal quantity;
}

/// 호가창 스냅샷입니다.
final class OrderBook {
  OrderBook({
    required this.market,
    required this.timestamp,
    required Iterable<Level> bids,
    required Iterable<Level> asks,
  }) : bids = List.unmodifiable(bids),
       asks = List.unmodifiable(asks);

  final Market market;
  final Timestamp timestamp;
  final List<Level> bids;
  final List<Level> asks;

  Level? get bestBid => bids.firstOrNull;
  Level? get bestAsk => asks.firstOrNull;

  /// 최우선 매도 가격에서 최우선 매수 가격을 뺀 값입니다.
  Decimal? get spread {
    final bid = bestBid;
    final ask = bestAsk;
    return bid == null || ask == null ? null : ask.price - bid.price;
  }

  /// 최우선 매수·매도 가격의 중간값입니다.
  Decimal? get midPrice {
    final bid = bestBid;
    final ask = bestAsk;
    return bid == null || ask == null ? null : (bid.price + ask.price)._half();
  }
}

/// 한 시장의 공급자 시세 요약입니다.
final class Ticker {
  Ticker({
    required this.market,
    required this.timestamp,
    required this.lastPrice,
    this.lastTradeTime,
    this.change,
    this.changeRate,
    this.high,
    this.low,
    this.volume,
    this.quoteVolume,
  });

  final Market market;
  final Timestamp timestamp;
  final Timestamp? lastTradeTime;
  final Decimal lastPrice;
  final Decimal? change;
  final Decimal? changeRate;
  final Decimal? high;
  final Decimal? low;
  final Decimal? volume;
  final Decimal? quoteVolume;
}

/// 하나의 시가·고가·저가·종가·거래량 캔들입니다.
final class Candle {
  Candle({
    required this.market,
    required this.interval,
    required this.openTime,
    required this.open,
    required this.high,
    required this.low,
    required this.close,
    required this.volume,
    required this.closed,
    this.quoteVolume,
  });

  final Market market;
  final Interval interval;
  final Timestamp openTime;
  final Decimal open;
  final Decimal high;
  final Decimal low;
  final Decimal close;
  final Decimal volume;
  final Decimal? quoteVolume;
  final bool closed;
}

/// 한 자산의 사용 가능·잠금 잔고입니다.
final class Balance {
  Balance({
    required String asset,
    required this.available,
    required this.locked,
  }) : asset = _asciiUpper(asset);

  final String asset;
  final Decimal available;
  final Decimal locked;

  /// 사용 가능한 잔고와 잠긴 잔고의 합입니다.
  Decimal get total => available + locked;
}

/// 주문 체결 방식입니다.
enum OrderType { market, limit }

/// 주문 유지 조건입니다.
enum TimeInForce { goodTilCancelled, immediateOrCancel, fillOrKill, postOnly }

/// 주문 수량의 기준 자산을 구분합니다.
sealed class Size {
  const Size(this.value);

  const factory Size.base(Decimal value) = BaseSize;
  const factory Size.quote(Decimal value) = QuoteSize;

  final Decimal value;
}

final class BaseSize extends Size {
  const BaseSize(super.value);
}

final class QuoteSize extends Size {
  const QuoteSize(super.value);
}

/// 주문의 현재 상태입니다.
enum OrderStatus {
  accepted,
  open,
  partiallyFilled,
  filled,
  cancelled,
  rejected,
  unknown,
}

extension OrderStatusProperties on OrderStatus {
  bool get isLive => switch (this) {
    OrderStatus.accepted ||
    OrderStatus.open ||
    OrderStatus.partiallyFilled => true,
    _ => false,
  };
}

/// 거래소가 보고한 주문입니다.
final class Order {
  Order({
    required this.id,
    required this.market,
    required this.side,
    required this.status,
    required this.filledQuantity,
    required this.remainingQuantity,
    this.price,
    this.createdAt,
  });

  final String id;
  final Market market;
  final Side side;
  final OrderStatus status;
  final Decimal filledQuantity;
  final Decimal remainingQuantity;
  final Decimal? price;
  final Timestamp? createdAt;
}

/// 포지션의 증거금 방식입니다.
enum MarginMode { cross, isolated }

/// 하나의 파생상품 포지션입니다.
final class Position {
  Position({
    required this.market,
    required this.quantity,
    this.side,
    this.entryPrice,
    this.markPrice,
    this.notional,
    this.unrealizedPnl,
    this.leverage,
    this.marginMode,
  });

  final Market market;
  final Side? side;
  final Decimal quantity;
  final Decimal? entryPrice;
  final Decimal? markPrice;
  final Decimal? notional;
  final Decimal? unrealizedPnl;
  final Decimal? leverage;
  final MarginMode? marginMode;

  bool get isFlat => quantity.isZero;
}

/// 계정 전체 증거금 상태입니다.
final class MarginSummary {
  MarginSummary({
    required String asset,
    this.equity,
    this.marginBalance,
    this.availableBalance,
  }) : asset = _asciiUpper(asset);

  final String asset;
  final Decimal? equity;
  final Decimal? marginBalance;
  final Decimal? availableBalance;
}

/// 한 시점의 펀딩 비율입니다.
final class FundingRate {
  FundingRate({
    required this.market,
    required this.timestamp,
    required this.rate,
    this.markPrice,
  });

  final Market market;
  final Timestamp timestamp;
  final Decimal rate;
  final Decimal? markPrice;
}

/// 계정에 실제 반영된 펀딩 지급 내역입니다.
final class FundingPayment {
  FundingPayment({
    required this.market,
    required this.timestamp,
    required this.amount,
    this.rate,
    this.id,
  });

  final Market market;
  final Timestamp timestamp;
  final Decimal amount;
  final Decimal? rate;
  final String? id;
}

/// 페이지 이력의 불투명한 재개 위치입니다.
final class Cursor {
  const Cursor(this.value);

  final String value;

  @override
  String toString() => value;

  @override
  bool operator ==(Object other) => other is Cursor && value == other.value;

  @override
  int get hashCode => value.hashCode;
}

/// 페이지 단위 이력 결과입니다.
final class Page<T> {
  Page({required Iterable<T> items, this.next})
    : items = List.unmodifiable(items);

  final List<T> items;
  final Cursor? next;

  bool get hasMore => next != null;
}

/// 시장가 또는 지정가 주문 요청입니다.
final class OrderRequest {
  const OrderRequest._({
    required this.market,
    required this.side,
    required this.orderType,
    required this.size,
    required this.price,
    required this.timeInForce,
    required this.reduceOnly,
  });

  factory OrderRequest.market(Market market, Side side, Size size) =>
      OrderRequest._(
        market: market,
        side: side,
        orderType: OrderType.market,
        size: size,
        price: null,
        timeInForce: null,
        reduceOnly: false,
      );

  factory OrderRequest.limit(
    Market market,
    Side side,
    Size size,
    Decimal price,
  ) => OrderRequest._(
    market: market,
    side: side,
    orderType: OrderType.limit,
    size: size,
    price: price,
    timeInForce: null,
    reduceOnly: false,
  );

  final Market market;
  final Side side;
  final OrderType orderType;
  final Size size;
  final Decimal? price;
  final TimeInForce? timeInForce;
  final bool reduceOnly;

  OrderRequest withTimeInForce(TimeInForce value) => OrderRequest._(
    market: market,
    side: side,
    orderType: orderType,
    size: size,
    price: price,
    timeInForce: value,
    reduceOnly: reduceOnly,
  );

  OrderRequest asReduceOnly() => OrderRequest._(
    market: market,
    side: side,
    orderType: orderType,
    size: size,
    price: price,
    timeInForce: timeInForce,
    reduceOnly: true,
  );
}

/// 페이지 단위 이력 조회 조건입니다.
final class HistoryRequest {
  const HistoryRequest(
    this.market, {
    this.from,
    this.to,
    this.cursor,
    this.limit,
  });

  final Market market;
  final Timestamp? from;
  final Timestamp? to;
  final Cursor? cursor;
  final int? limit;
}

/// 한 시장의 레버리지 또는 증거금 방식 변경 요청입니다.
final class MarginRequest {
  const MarginRequest(this.market, {this.leverage, this.marginMode});

  final Market market;
  final Decimal? leverage;
  final MarginMode? marginMode;
}

/// 스트림에서 구독할 시장 데이터 종류입니다.
enum FeedKind { trades, orderBook, ticker, candles }

/// 스트림 피드와 캔들 구간을 함께 식별합니다.
final class Feed {
  const Feed._(this.kind, [this.interval]);

  static const Feed trades = Feed._(FeedKind.trades);
  static const Feed orderBook = Feed._(FeedKind.orderBook);
  static const Feed ticker = Feed._(FeedKind.ticker);
  const Feed.candles(Interval interval) : this._(FeedKind.candles, interval);

  final FeedKind kind;
  final Interval? interval;

  @override
  bool operator ==(Object other) =>
      other is Feed && kind == other.kind && interval == other.interval;

  @override
  int get hashCode => Object.hash(kind, interval);
}

/// 모든 시장과 모든 피드의 곱집합을 구독합니다.
final class Subscription {
  Subscription({
    Iterable<Market> markets = const [],
    Iterable<Feed> feeds = const [],
  }) : markets = List.unmodifiable(<Market>{...markets}),
       feeds = List.unmodifiable(<Feed>{...feeds});

  final List<Market> markets;
  final List<Feed> feeds;

  Subscription withMarket(Market market) =>
      Subscription(markets: [...markets, market], feeds: feeds);

  Subscription withMarkets(Iterable<Market> values) =>
      Subscription(markets: [...markets, ...values], feeds: feeds);

  Subscription withFeed(Feed feed) =>
      Subscription(markets: markets, feeds: [...feeds, feed]);
}

/// 시장 스트림에서 전달되는 이벤트입니다.
sealed class MarketEvent {
  const MarketEvent();

  const factory MarketEvent.trade(Trade value) = TradeMarketEvent;
  const factory MarketEvent.orderBook(OrderBook value) = OrderBookMarketEvent;
  const factory MarketEvent.ticker(Ticker value) = TickerMarketEvent;
  const factory MarketEvent.candle(Candle value) = CandleMarketEvent;
  const factory MarketEvent.reconnected() = ReconnectedMarketEvent;
}

final class TradeMarketEvent extends MarketEvent {
  const TradeMarketEvent(this.value);
  final Trade value;
}

final class OrderBookMarketEvent extends MarketEvent {
  const OrderBookMarketEvent(this.value);
  final OrderBook value;
}

final class TickerMarketEvent extends MarketEvent {
  const TickerMarketEvent(this.value);
  final Ticker value;
}

final class CandleMarketEvent extends MarketEvent {
  const CandleMarketEvent(this.value);
  final Candle value;
}

final class ReconnectedMarketEvent extends MarketEvent {
  const ReconnectedMarketEvent();
}

/// 비공개 계정 스트림에서 전달되는 이벤트입니다.
sealed class AccountEvent {
  const AccountEvent();

  const factory AccountEvent.balance(Balance value) = BalanceAccountEvent;
  const factory AccountEvent.order(Order value) = OrderAccountEvent;
  const factory AccountEvent.reconnected() = ReconnectedAccountEvent;
}

final class BalanceAccountEvent extends AccountEvent {
  const BalanceAccountEvent(this.value);
  final Balance value;
}

final class OrderAccountEvent extends AccountEvent {
  const OrderAccountEvent(this.value);
  final Order value;
}

final class ReconnectedAccountEvent extends AccountEvent {
  const ReconnectedAccountEvent();
}
