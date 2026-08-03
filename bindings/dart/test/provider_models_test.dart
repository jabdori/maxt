import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

void main() {
  test('Binance 시장 식별자는 기존 공개 이름과 wire 값을 유지한다', () {
    expect(BinanceMarket.values, [
      BinanceMarket.spot,
      BinanceMarket.usdMFutures,
    ]);
    expect(BinanceMarket.spot.wireName, 'spot');
    expect(BinanceMarket.usdMFutures.wireName, 'usd_m');
  });

  test('기타 원장 종류는 알려진 공급자 이름과 variant를 구분한다', () {
    final otherDeposit = HyperliquidLedgerKind.other('deposit');
    final sameOtherDeposit = HyperliquidLedgerKind.other('deposit');

    expect(otherDeposit.isOther, isTrue);
    expect(otherDeposit, isNot(HyperliquidLedgerKind.deposit));
    expect(otherDeposit, sameOtherDeposit);
    expect(otherDeposit.hashCode, sameOtherDeposit.hashCode);
    expect({
      HyperliquidLedgerKind.deposit,
      otherDeposit,
      sameOtherDeposit,
    }, hasLength(2));
  });

  test('빈 공급자 이름도 기타 원장 종류로 보존한다', () {
    final kind = HyperliquidLedgerKind.other('');

    expect(kind.providerName, isEmpty);
    expect(kind.isOther, isTrue);
  });

  test('알려진 원장 종류의 기존 값은 유지한다', () {
    expect(HyperliquidLedgerKind.deposit.providerName, 'deposit');
    expect(HyperliquidLedgerKind.deposit.isOther, isFalse);
    expect(HyperliquidLedgerKind.deposit, same(HyperliquidLedgerKind.deposit));
  });

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
      endsAt: Timestamp.fromNanoseconds(BigInt.parse("1700000000123456789")),
    );
    final ledger = HyperliquidLedgerEntry(
      kind: HyperliquidLedgerKind.other('futureKind'),
      time: Timestamp.fromNanoseconds(BigInt.parse("1700000000123456790")),
      hash: '0x01',
      amount: Decimal.parse('0.000000000000000001'),
    );

    expect(context.midPrice.toString(), '12345678901234567890.00000001');
    expect(
      alert.endsAt.nanosecondsSinceEpoch,
      BigInt.parse('1700000000123456789'),
    );
    expect(ledger.kind.isOther, isTrue);
    expect(ledger.amount.toString(), '0.000000000000000001');
    expect(market.kind, MarketKind.perpetual);
  });
}
