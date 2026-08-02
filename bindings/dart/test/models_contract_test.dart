import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

void main() {
  test('Decimal은 double 없이 정확한 문자열과 수치 동등성을 보존한다', () {
    final value = Decimal.parse('12345678901234567890.1234567800');

    expect(value.toString(), '12345678901234567890.1234567800');
    expect(value, Decimal.parse('12345678901234567890.12345678'));
    expect(Decimal.parse('-0.000e9').isZero, isTrue);
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
    final before = DateTime.now().microsecondsSinceEpoch;
    final now = Timestamp.now().nanosecondsSinceEpoch;
    final after = DateTime.now().microsecondsSinceEpoch;

    expect(now, inInclusiveRange(0, 9223372036854775807));
    expect(now, inInclusiveRange(before * 1000, after * 1000));
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
