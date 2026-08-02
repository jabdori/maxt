import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

void main() {
  test('Decimal은 double 없이 정확한 문자열과 수치 동등성을 보존한다', () {
    final value = Decimal.parse('12345678901234567890.1234567800');

    expect(value.toString(), '12345678901234567890.1234567800');
    expect(value, Decimal.parse('12345678901234567890.12345678'));
    expect(Decimal.parse('-0.000e9').isZero, isTrue);
  });

  test('Decimal은 Rust Decimal의 범위와 정확도를 벗어난 값을 거절한다', () {
    for (final value in [
      '2.5e-28',
      '2e-29',
      '79228162514264337593543950335.4',
      '1e29',
      '1e2000000000',
      '1e-2000000000',
    ]) {
      expect(() => Decimal.parse(value), throwsFormatException, reason: value);
    }
  });

  test('Decimal은 수치 비교와 정확한 덧셈·뺄셈을 제공한다', () {
    final smaller = Decimal.parse('-0.1');
    final equal = Decimal.parse('1.00');
    final larger = Decimal.parse('1e1');

    expect(smaller.compareTo(equal), lessThan(0));
    expect(equal.compareTo(Decimal.parse('1')), 0);
    expect(larger.compareTo(equal), greaterThan(0));
    expect(smaller < equal, isTrue);
    expect(smaller <= equal, isTrue);
    expect(larger > equal, isTrue);
    expect(larger >= Decimal.parse('10.0'), isTrue);
    expect(
      Decimal.parse('7314.6229858868828353570724702') + Decimal.parse('1000'),
      Decimal.parse('8314.622985886882835357072470'),
    );
    expect(Decimal.parse('1') - Decimal.parse('1.5'), Decimal.parse('-0.5'));
  });

  test('Balance.total은 Rust와 같은 정밀도와 범위를 사용한다', () {
    final balance = Balance(
      asset: 'btc',
      available: Decimal.parse('7314.6229858868828353570724702'),
      locked: Decimal.parse('1000'),
    );
    final overflow = Balance(
      asset: 'btc',
      available: Decimal.parse('79228162514264337593543950335'),
      locked: Decimal.one,
    );

    expect(balance.total, Decimal.parse('8314.622985886882835357072470'));
    expect(() => overflow.total, throwsRangeError);
  });

  test('OrderBook은 양쪽 최우선 호가로 spread와 midPrice를 계산한다', () {
    final normal = _orderBook(bids: ['99'], asks: ['101']);
    final oneSided = _orderBook(bids: ['99']);
    final empty = _orderBook();
    final crossed = _orderBook(bids: ['102'], asks: ['101']);

    expect(normal.spread, Decimal.parse('2'));
    expect(normal.midPrice, Decimal.parse('100'));
    expect(oneSided.spread, isNull);
    expect(oneSided.midPrice, isNull);
    expect(empty.spread, isNull);
    expect(empty.midPrice, isNull);
    expect(crossed.spread, Decimal.parse('-1'));
    expect(crossed.midPrice, Decimal.parse('101.5'));
  });

  test('OrderBook.midPrice는 절반 경계에서 half-even으로 반올림한다', () {
    expect(_orderBook(bids: ['0'], asks: ['1e-28']).midPrice, Decimal.zero);
    expect(
      _orderBook(bids: ['1e-28'], asks: ['2e-28']).midPrice,
      Decimal.parse('2e-28'),
    );
    expect(
      _orderBook(bids: ['0'], asks: ['79228162514264337593543950335']).midPrice,
      Decimal.parse('39614081257132168796771975168'),
    );
  });

  test('OrderBook.midPrice는 Rust처럼 합이 넘치면 실패한다', () {
    final book = _orderBook(
      bids: ['1'],
      asks: ['79228162514264337593543950335'],
    );

    expect(() => book.midPrice, throwsRangeError);
  });

  test('Timestamp는 Unix epoch 나노초를 int로 왕복한다', () {
    const nanoseconds = 1700000000123456789;
    final timestamp = Timestamp.fromNanoseconds(nanoseconds);

    expect(timestamp.nanosecondsSinceEpoch, nanoseconds);
  });

  test('Timestamp 단위 변환은 i64 범위에서 포화하고 epoch 방향으로 절삭한다', () {
    expect(
      Timestamp.fromSeconds(9223372037).nanosecondsSinceEpoch,
      9223372036854775807,
    );
    expect(
      Timestamp.fromSeconds(-9223372037).nanosecondsSinceEpoch,
      -9223372036854775808,
    );
    expect(
      Timestamp.fromMilliseconds(9223372036855).nanosecondsSinceEpoch,
      9223372036854775807,
    );
    expect(
      Timestamp.fromMilliseconds(-9223372036855).nanosecondsSinceEpoch,
      -9223372036854775808,
    );
    expect(
      Timestamp.fromMicroseconds(9223372036854776).nanosecondsSinceEpoch,
      9223372036854775807,
    );
    expect(
      Timestamp.fromMicroseconds(-9223372036854776).nanosecondsSinceEpoch,
      -9223372036854775808,
    );

    final negative = Timestamp.fromNanoseconds(-1999999999);
    expect(negative.secondsSinceEpoch, -1);
    expect(negative.millisecondsSinceEpoch, -1999);
    expect(Timestamp.fromNanoseconds(-999999).millisecondsSinceEpoch, 0);
  });

  test('Timestamp.now는 Dart 시계 범위와 i64 범위 안의 시각을 반환한다', () {
    const clockAdjustmentToleranceMicroseconds = 10000;
    final before = DateTime.now().microsecondsSinceEpoch;
    final now = Timestamp.now().nanosecondsSinceEpoch;
    final after = DateTime.now().microsecondsSinceEpoch;

    expect(now, inInclusiveRange(0, 9223372036854775807));
    expect(now.remainder(1000), 0);
    expect(
      now,
      inInclusiveRange(
        (before - clockAdjustmentToleranceMicroseconds) * 1000,
        (after + clockAdjustmentToleranceMicroseconds) * 1000,
      ),
    );
  });

  test('Interval은 고정 길이와 정확한 나노초 이동을 제공한다', () {
    expect(
      {for (final interval in Interval.values) interval: interval.seconds},
      {
        Interval.sec1: 1,
        Interval.min1: 60,
        Interval.min3: 180,
        Interval.min5: 300,
        Interval.min15: 900,
        Interval.min30: 1800,
        Interval.hour1: 3600,
        Interval.hour2: 7200,
        Interval.hour4: 14400,
        Interval.hour8: 28800,
        Interval.hour12: 43200,
        Interval.day1: 86400,
        Interval.day3: 259200,
        Interval.week1: 604800,
        Interval.month1: null,
      },
    );

    final at = Timestamp.fromNanoseconds(1700000000123456789);
    expect(
      Interval.min1.advance(at, 2),
      Timestamp.fromNanoseconds(1700000120123456789),
    );
    expect(
      Interval.week1.advance(at, -1),
      Timestamp.fromNanoseconds(1700000000123456789 - 604800000000000),
    );
  });

  test('month1은 UTC 달력 월을 이동하고 나노초를 보존한다', () {
    final january31 = Timestamp.fromNanoseconds(1706659200000000456);
    final february29 = Timestamp.fromNanoseconds(1709164800000000456);
    final march31 = Timestamp.fromNanoseconds(1711843200000000456);

    expect(Interval.month1.advance(january31, 1), february29);
    expect(Interval.month1.advance(march31, -1), february29);
    expect(Interval.month1.advance(january31, 0), january31);
    expect(
      Interval.month1.advance(Timestamp.fromNanoseconds(-2678400000000001), 1),
      Timestamp.fromNanoseconds(-86400000000001),
    );
  });

  test('Interval 이동은 i64 범위를 넘으면 null을 반환한다', () {
    final late = Timestamp.fromNanoseconds(9220000000000000000);

    expect(Interval.month1.advance(late, 12), isNull);
    expect(Interval.week1.advance(late, 9223372036854775807), isNull);
    expect(Interval.month1.advance(Timestamp.zero, 4294967296), isNull);
  });

  test('공통 값과 설정 생성자는 Rust 기본값을 보존한다', () {
    final market = Market.spot(Exchange.upbit, 'btc', 'krw');
    final candles = CandleRequest(market, Interval.min1);
    const stream = StreamConfig();

    expect(market.base, 'BTC');
    expect(market.quote, 'KRW');
    expect(candles.from, isNull);
    expect(candles.to, isNull);
    expect(candles.limit, isNull);
    expect(stream.maxReconnectAttempts, isNull);
    expect(stream.initialReconnectDelayMs, 1000);
    expect(stream.maxReconnectDelayMs, 30000);
    expect(stream.idleTimeoutMs, 30000);
    expect(stream.bufferSize, 4096);
    expect(stream.overflow, Overflow.backpressure);
  });
}

OrderBook _orderBook({
  List<String> bids = const [],
  List<String> asks = const [],
}) {
  final market = Market.spot(Exchange.upbit, 'btc', 'krw');
  Level level(String price) =>
      Level(price: Decimal.parse(price), quantity: Decimal.one);

  return OrderBook(
    market: market,
    timestamp: Timestamp.zero,
    bids: bids.map(level),
    asks: asks.map(level),
  );
}
