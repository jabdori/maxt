export 'generated_identifiers.dart';

import 'generated_identifiers.dart';

part 'generated_models.dart';

String _asciiUpper(String value) => String.fromCharCodes(
  value.codeUnits.map(
    (code) => code >= 0x61 && code <= 0x7a ? code - 0x20 : code,
  ),
);

extension ExchangeName on Exchange {
  /// Stable lowercase identifier used in logs and errors.
  String get id => name;

  /// Human-readable exchange name.
  String get displayName => switch (this) {
    Exchange.upbit => 'Upbit',
    Exchange.bithumb => 'Bithumb',
    Exchange.binance => 'Binance',
    Exchange.hyperliquid => 'Hyperliquid',
  };
}

extension FeatureProperties on Feature {
  /// Whether this feature requires credentials.
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

  /// Whether this feature is meaningful only for derivative markets.
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

/// Exact decimal that preserves its string representation without floating-point conversion.
///
/// The absolute coefficient is 96-bit and the scale is in the range 0 through 28.
final class Decimal implements Comparable<Decimal> {
  Decimal._(this._value, this._coefficient, this._scale);

  /// Parses a finite decimal in plain or scientific notation without rounding.
  ///
  /// Throws [FormatException] when the value is outside the maxt Decimal range.
  factory Decimal.parse(String value) {
    final match = _pattern.firstMatch(value);
    if (match == null) {
      throw FormatException('Invalid decimal', value);
    }

    final integer = match.group(2) ?? '';
    final fraction = match.group(3) ?? match.group(4) ?? '';
    final exponentText = match.group(5);
    var exponent = 0;
    if (exponentText != null) {
      final parsed = _boundedExponent(exponentText, integer.length);
      if (parsed == null) {
        throw FormatException(
          'Decimal scientific notation is too large',
          value,
        );
      }
      exponent = parsed;
    }
    if (exponent < fraction.length - _maxScale) {
      throw FormatException('Decimal is outside the maxt Decimal range', value);
    }

    final digits = '$integer$fraction';
    final coefficientDigits = _coefficientDigits(
      digits,
      fraction.length - exponent,
    );
    if (coefficientDigits == null) {
      throw FormatException('Decimal is outside the maxt Decimal range', value);
    }

    var coefficient = BigInt.parse(coefficientDigits);
    if (match.group(1) == '-') coefficient = -coefficient;
    var scale = fraction.length - exponent;

    if (coefficient == BigInt.zero) {
      scale = 0;
    } else {
      while (coefficient.remainder(_ten) == BigInt.zero) {
        coefficient ~/= _ten;
        scale--;
      }
    }
    return Decimal._(value, coefficient, scale);
  }

  static final RegExp _pattern = RegExp(
    r'^([+-]?)(?:(\d+)(?:\.(\d*))?|\.(\d+))(?:[eE]([+-]?\d+))?$',
  );

  static const String _maxCoefficientText = '79228162514264337593543950335';
  static const int _maxScale = 28;
  static const int _maxPointShift = 64;
  static final Decimal zero = Decimal.parse('0');
  static final Decimal one = Decimal.parse('1');
  static final BigInt _maxCoefficient = BigInt.parse(_maxCoefficientText);
  static final BigInt _ten = BigInt.from(10);

  final String _value;
  final BigInt _coefficient;
  final int _scale;

  bool get isZero => _coefficient == BigInt.zero;

  /// Adds two values.
  ///
  /// Rounds half-even when reducing precision and throws [RangeError] on overflow.
  Decimal operator +(Decimal other) => _addOrSubtract(other, subtract: false);

  /// Subtracts another value.
  ///
  /// Rounds half-even when reducing precision and throws [RangeError] on overflow.
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

  static int? _boundedExponent(String text, int wholeLength) {
    final minimum = -_maxPointShift - wholeLength;
    final maximum = _maxPointShift - wholeLength;
    var start = 0;
    var negative = false;
    final sign = text.codeUnitAt(0);
    if (sign == 45 || sign == 43) {
      negative = sign == 45;
      start++;
    }
    while (start < text.length && text.codeUnitAt(start) == 48) {
      start++;
    }
    if (start == text.length) {
      return minimum <= 0 && maximum >= 0 ? 0 : null;
    }
    if (_compareSignedDigits(negative, text, start, minimum) < 0 ||
        _compareSignedDigits(negative, text, start, maximum) > 0) {
      return null;
    }

    var result = 0;
    for (var index = start; index < text.length; index++) {
      final digit = text.codeUnitAt(index) - 48;
      result = negative ? result * 10 - digit : result * 10 + digit;
    }
    return result;
  }

  static int _compareSignedDigits(
    bool negative,
    String text,
    int start,
    int bound,
  ) {
    final boundText = bound.toString();
    final boundNegative = boundText.codeUnitAt(0) == 45;
    if (negative != boundNegative) return negative ? -1 : 1;

    final boundStart = boundNegative ? 1 : 0;
    final length = text.length - start;
    final boundLength = boundText.length - boundStart;
    var comparison = length.compareTo(boundLength);
    if (comparison == 0) {
      for (var index = 0; index < length; index++) {
        comparison = text
            .codeUnitAt(start + index)
            .compareTo(boundText.codeUnitAt(boundStart + index));
        if (comparison != 0) break;
      }
    }
    return negative ? -comparison : comparison;
  }

  static String? _coefficientDigits(String digits, int scale) {
    var start = 0;
    while (start < digits.length && digits.codeUnitAt(start) == 48) {
      start++;
    }
    if (start == digits.length) return '0';

    final significantLength = digits.length - start;
    final appendedZeros = scale < 0 ? -scale : 0;
    final expandedLength = significantLength + appendedZeros;
    if (expandedLength > _maxCoefficientText.length) return null;
    if (expandedLength == _maxCoefficientText.length) {
      for (var index = 0; index < expandedLength; index++) {
        final digit = index < significantLength
            ? digits.codeUnitAt(start + index)
            : 48;
        final maximum = _maxCoefficientText.codeUnitAt(index);
        if (digit < maximum) break;
        if (digit > maximum) return null;
      }
    }
    return digits.substring(start);
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

/// Point in time represented as UTC nanoseconds since the Unix epoch.
final class Timestamp implements Comparable<Timestamp> {
  Timestamp._(this.nanosecondsSinceEpoch);

  /// Creates a timestamp from nanoseconds since the Unix epoch.
  ///
  /// Throws [RangeError] when the value is outside the signed 64-bit (`i64`) range.
  factory Timestamp.fromNanoseconds(Object nanoseconds) {
    final value = switch (nanoseconds) {
      int value => BigInt.from(value),
      BigInt value => value,
      _ => throw ArgumentError.value(
        nanoseconds,
        'nanoseconds',
        'must be an int or BigInt',
      ),
    };
    if (value < _min || value > _max) {
      throw RangeError('nanoseconds must fit in i64: $nanoseconds');
    }
    return Timestamp._(value);
  }

  /// Creates a timestamp from microseconds since the Unix epoch.
  ///
  /// Saturates to the nearest `i64` bound when nanosecond conversion overflows.
  factory Timestamp.fromMicroseconds(int microseconds) =>
      _fromScaled(microseconds, 1000);

  /// Creates a timestamp from milliseconds since the Unix epoch.
  ///
  /// Saturates to the nearest `i64` bound when nanosecond conversion overflows.
  factory Timestamp.fromMilliseconds(int milliseconds) =>
      _fromScaled(milliseconds, 1000000);

  /// Creates a timestamp from seconds since the Unix epoch.
  ///
  /// Saturates to the nearest `i64` bound when nanosecond conversion overflows.
  factory Timestamp.fromSeconds(int seconds) =>
      _fromScaled(seconds, 1000000000);

  /// Current system time.
  factory Timestamp.now() {
    final microseconds = DateTime.now().microsecondsSinceEpoch;
    return microseconds <= 0 ? zero : _fromScaled(microseconds, 1000);
  }

  static final BigInt _min = BigInt.parse('-9223372036854775808');
  static final BigInt _max = BigInt.parse('9223372036854775807');
  static final Timestamp zero = Timestamp._(BigInt.zero);

  final BigInt nanosecondsSinceEpoch;

  /// Milliseconds since the Unix epoch, truncated toward `0`.
  int get millisecondsSinceEpoch =>
      (nanosecondsSinceEpoch ~/ BigInt.from(1000000)).toInt();

  /// Seconds since the Unix epoch, truncated toward `0`.
  int get secondsSinceEpoch =>
      (nanosecondsSinceEpoch ~/ BigInt.from(1000000000)).toInt();

  static Timestamp _fromScaled(int value, int scale) {
    final nanoseconds = BigInt.from(value) * BigInt.from(scale);
    if (nanoseconds < _min) return Timestamp._(_min);
    if (nanoseconds > _max) return Timestamp._(_max);
    return Timestamp._(nanoseconds);
  }

  static Timestamp? _checked(BigInt nanoseconds) {
    if (nanoseconds < _min || nanoseconds > _max) return null;
    return Timestamp._(nanoseconds);
  }

  static bool _contains(BigInt value) => value >= _min && value <= _max;

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

extension MarketKindProperties on MarketKind {
  bool get isDerivative => this == MarketKind.perpetual;
}

/// Identifies one exchange market.
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

extension IntervalProperties on Interval {
  /// Duration in seconds for a fixed interval; calendar month [Interval.month1] is `null`.
  int? get seconds => switch (this) {
    Interval.sec1 => 1,
    Interval.min1 => 60,
    Interval.min3 => 180,
    Interval.min5 => 300,
    Interval.min10 => 600,
    Interval.min15 => 900,
    Interval.min30 => 1800,
    Interval.hour1 => 3600,
    Interval.hour2 => 7200,
    Interval.hour4 => 14400,
    Interval.hour6 => 21600,
    Interval.hour8 => 28800,
    Interval.hour12 => 43200,
    Interval.day1 => 86400,
    Interval.day3 => 259200,
    Interval.week1 => 604800,
    Interval.month1 => null,
  };

  /// Moves [count] intervals from [at].
  ///
  /// Fixed intervals move by their exact duration in nanoseconds.
  /// [Interval.month1] uses UTC calendar months and clamps a missing day to
  /// the last day of the destination month, so reversing a move may not
  /// restore the original timestamp.
  ///
  /// It does not apply an exchange-specific provider candle grid.
  /// Returns `null` when the result cannot be represented as [Timestamp].
  Timestamp? advance(Timestamp at, int count) {
    final countBigInt = BigInt.from(count);

    final fixedSeconds = seconds;
    if (fixedSeconds != null) {
      final span =
          BigInt.from(fixedSeconds) * BigInt.from(1000000000) * countBigInt;
      if (!Timestamp._contains(span)) return null;
      return Timestamp._checked(at.nanosecondsSinceEpoch + span);
    }

    if (countBigInt.abs() > BigInt.from(4294967295)) return null;

    var microseconds = at.nanosecondsSinceEpoch ~/ BigInt.from(1000);
    var nanosecondRemainder = at.nanosecondsSinceEpoch.remainder(
      BigInt.from(1000),
    );
    if (nanosecondRemainder < BigInt.zero) {
      microseconds -= BigInt.one;
      nanosecondRemainder += BigInt.from(1000);
    }
    final current = DateTime.fromMicrosecondsSinceEpoch(
      microseconds.toInt(),
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
          nanosecondRemainder,
    );
  }
}

/// Conditions for querying candle history.
///
/// Queries candles whose opening time is at least [from] and before [to],
/// returning them oldest first. With [from], [limit] selects the oldest
/// candles; without it, it selects the newest candles.
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

/// Stream connection and buffering configuration.
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

/// One entry in a market list.
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

extension SideProperties on Side {
  Side get flipped => this == Side.buy ? Side.sell : Side.buy;
}

/// One trade execution.
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

/// One price level in an order book.
final class Level {
  Level({required this.price, required this.quantity});

  final Decimal price;
  final Decimal quantity;
}

/// Order-book snapshot.
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

  /// Best ask price minus best bid price.
  ///
  /// Returns `null` when either side is empty; a crossed book returns a negative value.
  Decimal? get spread {
    final bid = bestBid;
    final ask = bestAsk;
    return bid == null || ask == null ? null : ask.price - bid.price;
  }

  /// Midpoint of the best bid and ask prices, or `null` when either side is empty.
  ///
  /// Uses half-even rounding if [Decimal] precision must be reduced and throws
  /// [RangeError] when the sum is outside the [Decimal] range.
  Decimal? get midPrice {
    final bid = bestBid;
    final ask = bestAsk;
    return bid == null || ask == null ? null : (bid.price + ask.price)._half();
  }
}

/// Provider price summary for one market.
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

/// One open-high-low-close-volume candle.
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

/// Available and locked balances for one asset.
final class Balance {
  Balance({
    required String asset,
    required this.available,
    required this.locked,
  }) : asset = _asciiUpper(asset);

  final String asset;
  final Decimal available;
  final Decimal locked;

  /// Sum of the available and locked balances.
  ///
  /// Uses half-even rounding if addition reduces precision and throws
  /// [RangeError] when the sum is outside the [Decimal] range.
  Decimal get total => available + locked;
}

/// Distinguishes whether an order size is denominated in base or quote asset.
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

extension OrderStatusProperties on OrderStatus {
  bool get isLive => switch (this) {
    OrderStatus.accepted ||
    OrderStatus.open ||
    OrderStatus.partiallyFilled => true,
    _ => false,
  };
}

/// Order reported by an exchange.
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

/// One derivative position.
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

/// Account-wide margin summary.
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

/// Funding rate at one point in time.
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

/// Funding payment actually applied to an account.
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

/// Opaque resume position for paginated history.
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

/// Paginated history result.
final class Page<T> {
  Page({required Iterable<T> items, this.next})
    : items = List.unmodifiable(items);

  final List<T> items;
  final Cursor? next;

  bool get hasMore => next != null;
}

/// Market, limit, or best-limit order request.
final class OrderRequest {
  const OrderRequest._({
    required this.market,
    required this.side,
    required this.orderType,
    required this.size,
    required this.price,
    required this.timeInForce,
    required this.reduceOnly,
    required this.clientId,
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
        clientId: null,
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
    clientId: null,
  );

  factory OrderRequest.best(
    Market market,
    Side side,
    Size size,
    TimeInForce timeInForce,
  ) => OrderRequest._(
    market: market,
    side: side,
    orderType: OrderType.best,
    size: size,
    price: null,
    timeInForce: timeInForce,
    reduceOnly: false,
    clientId: null,
  );

  final Market market;
  final Side side;
  final OrderType orderType;
  final Size size;
  final Decimal? price;
  final TimeInForce? timeInForce;
  final bool reduceOnly;
  final String? clientId;

  OrderRequest withTimeInForce(TimeInForce value) => OrderRequest._(
    market: market,
    side: side,
    orderType: orderType,
    size: size,
    price: price,
    timeInForce: value,
    reduceOnly: reduceOnly,
    clientId: clientId,
  );

  OrderRequest asReduceOnly() => OrderRequest._(
    market: market,
    side: side,
    orderType: orderType,
    size: size,
    price: price,
    timeInForce: timeInForce,
    reduceOnly: true,
    clientId: clientId,
  );

  OrderRequest withClientId(String value) => OrderRequest._(
    market: market,
    side: side,
    orderType: orderType,
    size: size,
    price: price,
    timeInForce: timeInForce,
    reduceOnly: reduceOnly,
    clientId: value,
  );
}

/// Conditions for querying paginated history.
///
/// Queries entries whose timestamp is at least [from] and before [to]. With
/// [cursor], [from] is ignored and querying resumes after the previous page.
/// [limit] is a target page size; the actual count can differ to avoid splitting
/// entries with the same timestamp.
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

/// Request to change leverage or margin mode for one market.
final class MarginRequest {
  const MarginRequest(this.market, {this.leverage, this.marginMode});

  final Market market;
  final Decimal? leverage;
  final MarginMode? marginMode;
}

/// Kind of market data to subscribe to in a stream.
enum FeedKind { trades, orderBook, ticker, candles }

/// Identifies a stream feed and, for candles, its interval.
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

/// Subscribes to the Cartesian product of all selected markets and feeds.
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

/// Event delivered by a market stream.
sealed class MarketEvent {
  const MarketEvent();

  /// Single trade event.
  const factory MarketEvent.trade(Trade value) = TradeMarketEvent;

  /// Latest order-book snapshot event.
  const factory MarketEvent.orderBook(OrderBook value) = OrderBookMarketEvent;

  /// Current price summary event.
  const factory MarketEvent.ticker(Ticker value) = TickerMarketEvent;

  /// Created, updated, or closed candle event.
  const factory MarketEvent.candle(Candle value) = CandleMarketEvent;

  /// Indicates reconnection; events may have been lost while disconnected.
  const factory MarketEvent.reconnected() = ReconnectedMarketEvent;
}

final class TradeMarketEvent extends MarketEvent {
  /// Creates an event containing one trade in [value].
  const TradeMarketEvent(this.value);

  /// Trade information sent by the exchange.
  final Trade value;
}

final class OrderBookMarketEvent extends MarketEvent {
  /// Creates an event containing the latest order-book snapshot in [value].
  const OrderBookMarketEvent(this.value);

  /// Order-book snapshot sent by the exchange.
  final OrderBook value;
}

final class TickerMarketEvent extends MarketEvent {
  /// Creates an event containing the current price summary in [value].
  const TickerMarketEvent(this.value);

  /// Current price summary sent by the exchange.
  final Ticker value;
}

final class CandleMarketEvent extends MarketEvent {
  /// Creates an event containing the candle update in [value].
  const CandleMarketEvent(this.value);

  /// Candle information sent by the exchange.
  final Candle value;
}

final class ReconnectedMarketEvent extends MarketEvent {
  /// Creates a reconnection notification.
  const ReconnectedMarketEvent();
}

/// Event delivered by a private account stream.
sealed class AccountEvent {
  const AccountEvent();

  /// Account event reporting a balance change.
  const factory AccountEvent.balance(Balance value) = BalanceAccountEvent;

  /// Account event reporting order creation, fill, or cancellation state.
  const factory AccountEvent.order(Order value) = OrderAccountEvent;

  /// Indicates the account stream reconnected and state must be refreshed.
  const factory AccountEvent.reconnected() = ReconnectedAccountEvent;
}

final class BalanceAccountEvent extends AccountEvent {
  /// Creates an event containing the changed balance in [value].
  const BalanceAccountEvent(this.value);

  /// Latest balance sent by the exchange.
  final Balance value;
}

final class OrderAccountEvent extends AccountEvent {
  /// Creates an event containing the changed order in [value].
  const OrderAccountEvent(this.value);

  /// Latest order state sent by the exchange.
  final Order value;
}

final class ReconnectedAccountEvent extends AccountEvent {
  /// Creates an account reconnection notification.
  const ReconnectedAccountEvent();
}
