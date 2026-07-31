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
