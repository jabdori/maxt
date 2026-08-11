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
    final notice = BithumbNotice(
      categories: ['입출금'],
      title: '네트워크 점검 안내',
      url: 'https://feed.bithumb.com/notice/1654458',
      publishedAt: Timestamp.fromNanoseconds(BigInt.parse('1700000000123456790')),
      modifiedAt: Timestamp.fromNanoseconds(BigInt.parse('1700000000123456791')),
    );
    final fee = BithumbAssetFee(
      displayName: '비트코인',
      asset: 'btc',
      networks: [
        BithumbNetworkFee(
          network: Network.bitcoin,
          providerName: 'Bitcoin',
          depositFee: Decimal.zero,
          minimumDeposit: Decimal.zero,
          withdrawalFee: WithdrawalFee.fixed(Decimal.parse('0.0002')),
          minimumWithdrawal: Decimal.parse('0.001'),
        ),
      ],
    );
    final apiKey = BithumbApiKey(
      accessKey: 'example-access-key-1',
      expiresAt: Timestamp.fromSeconds(1812672000),
    );
    final pendingOrders = BithumbPendingOrdersRequest(
      market: Market.spot(Exchange.bithumb, 'BTC', 'KRW'),
      state: BithumbPendingOrderState.watch,
      limit: 25,
      orderBy: BithumbOrderDirection.ascending,
      cursor: Cursor('page+/=='),
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
    expect(notice.categories, ['입출금']);
    expect(notice.modifiedAt.nanosecondsSinceEpoch, BigInt.parse('1700000000123456791'));
    expect(fee.asset, 'BTC');
    expect(fee.networks.single.withdrawalFee, isA<WithdrawalFeeFixed>());
    expect(apiKey.accessKey, 'example-access-key-1');
    expect(apiKey.expiresAt, Timestamp.fromSeconds(1812672000));
    expect(pendingOrders.state, BithumbPendingOrderState.watch);
    expect(pendingOrders.orderBy, BithumbOrderDirection.ascending);
    expect(pendingOrders.cursor?.value, 'page+/==');
    expect(market.kind, MarketKind.perpetual);
  });

  test('Upbit 연간 캔들과 호가 정책은 지역별 필드를 잃지 않는다', () {
    final market = Market.spot(Exchange.upbit, 'BTC', 'KRW');
    final annual = UpbitYearCandle(
      market: market,
      openTime: Timestamp.fromSeconds(1767225600),
      koreaOpenTime: Timestamp.fromSeconds(1767225600),
      timestamp: Timestamp.fromNanoseconds(BigInt.parse('1786467753786000000')),
      open: Decimal.parse('128000000.00000000'),
      high: Decimal.parse('143050000.00000000'),
      low: Decimal.parse('88770000.00000000'),
      close: Decimal.parse('89587000.00000000'),
      volume: Decimal.parse('348666.78732189'),
      quoteVolume: Decimal.parse('37189906239683.17623000'),
      firstDayOfPeriod: '2026-01-01',
    );
    final policy = UpbitOrderBookInstrument(
      market: market,
      quoteCurrency: 'KRW',
      tickSize: Decimal.parse('1000'),
      supportedLevels: [Decimal.zero, Decimal.parse('10000')],
    );
    final deposit = UpbitDepositInfo(
      asset: 'btc',
      network: Network.bitcoin,
      providerNetwork: 'BTC',
      isDepositPossible: true,
      minimumDepositAmount: Decimal.parse('0.0005'),
      minimumDepositConfirmations: BigInt.parse('18446744073709551615'),
      decimalPrecision: BigInt.parse('18446744073709551615'),
    );

    expect(annual.koreaOpenTime, annual.openTime);
    expect(annual.quoteVolume.toString(), '37189906239683.17623000');
    expect(policy.supportedLevels, [Decimal.zero, Decimal.parse('10000')]);
    expect(deposit.asset, 'BTC');
    expect(deposit.minimumDepositAmount, Decimal.parse('0.0005'));
    expect(
      deposit.minimumDepositConfirmations,
      BigInt.parse('18446744073709551615'),
    );
    expect(deposit.decimalPrecision, BigInt.parse('18446744073709551615'));
    expect(deposit.depositImpossibleReason, isNull);
  });
}
