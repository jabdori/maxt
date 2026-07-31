import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

void main() {
  test('공급자 전용 모델도 정확 소수와 Timestamp를 유지한다', () {
    final market = Market.perpetual(Exchange.hyperliquid, 'BTC', 'USDC');
    final context = HyperliquidAssetContext(
      midPrice: Decimal.parse('12345678901234567890.00000001'),
      sizeDecimals: 5,
      priceDecimals: 1,
    );
    final alert = BithumbMarketAlert(
      market: Market.spot(Exchange.bithumb, 'BTC', 'KRW'),
      kind: 'PRICE_DIFFERENCE',
      step: BithumbAlertStep.warning,
      endsAt: Timestamp.fromNanoseconds(1700000000123456789),
    );
    final ledger = HyperliquidLedgerEntry(
      kind: HyperliquidLedgerKind.other('futureKind'),
      time: Timestamp.fromNanoseconds(1700000000123456790),
      hash: '0x01',
      amount: Decimal.parse('0.000000000000000001'),
    );

    expect(context.midPrice.toString(), '12345678901234567890.00000001');
    expect(alert.endsAt.nanosecondsSinceEpoch, 1700000000123456789);
    expect(ledger.kind.isOther, isTrue);
    expect(ledger.amount.toString(), '0.000000000000000001');
    expect(market.kind, MarketKind.perpetual);
  });
}
