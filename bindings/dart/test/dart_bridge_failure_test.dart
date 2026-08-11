import 'package:maxt/maxt.dart';
import 'package:maxt/src/adapters.dart';
import 'package:maxt/src/rust/adapter.dart' as native_adapter;
import 'package:maxt/src/rust/convert.dart' as wire;
import 'package:maxt/src/rust/convert/generated_models.dart' as wire_model;
import 'package:test/test.dart';

final class EmptyFeedAdapter extends AdapterBase {
  @override
  Exchange get exchange => Exchange.binance;

  @override
  Set<Feature> get features => const {Feature.tradeStream};
}

final class TransferLookupAdapter extends AdapterBase {
  TransferLookupRequest? depositRequest;
  TransferLookupRequest? withdrawalRequest;
  String? cancelledWithdrawalId;

  @override
  Exchange get exchange => Exchange.upbit;

  @override
  Set<Feature> get features => const {
    Feature.depositLookup,
    Feature.withdrawalLookup,
    Feature.withdrawalCancellation,
  };

  @override
  Future<Deposit> deposit(TransferLookupRequest request) async {
    depositRequest = request;
    return Deposit(
      id: request.id ?? 'deposit-1',
      asset: request.asset,
      amount: Decimal.one,
      status: DepositStatus.completed,
      providerStatus: 'DONE',
      txId: request.txId,
    );
  }

  @override
  Future<Withdrawal> withdrawal(TransferLookupRequest request) async {
    withdrawalRequest = request;
    return Withdrawal(
      id: request.id ?? 'withdrawal-1',
      asset: request.asset,
      amount: Decimal.one,
      status: WithdrawalStatus.pending,
      providerStatus: 'REQUESTED',
      txId: request.txId,
    );
  }

  @override
  Future<void> cancelWithdrawal(String withdrawalId) async {
    cancelledWithdrawalId = withdrawalId;
  }
}

void main() {
  setUpAll(Maxt.initialize);
  tearDownAll(Maxt.dispose);

  test('Subscribe 초기 실패는 원래 오류를 보존하고 registry를 비운다', () async {
    final adapter = EmptyFeedAdapter();
    final bridge = DartAdapterBridge(adapter);
    final nativeClient = await bridge.register();
    final market = const wire.WireMarket(
      exchange: wire.WireExchange.binance,
      kind: wire.WireMarketKind.perpetual,
      base: 'BTC',
      quote: 'USDT',
    );
    final subscription = native_adapter.WireSubscription(
      markets: [market],
      feeds: const [],
    );
    final config = native_adapter.WireStreamConfig(
      initialReconnectDelayMs: BigInt.from(1000),
      maxReconnectDelayMs: BigInt.from(30000),
      idleTimeoutMs: BigInt.from(30000),
      bufferSize: BigInt.from(4096),
      overflow: native_adapter.WireOverflow.backpressure,
    );

    for (var attempt = 0; attempt < 20; attempt++) {
      await expectLater(
        nativeClient.subscribe(subscription: subscription, config: config),
        throwsA(
          isA<wire.NativeError>()
              .having(
                (error) => error.kind,
                'kind',
                wire.NativeErrorKind.invalidRequest,
              )
              .having((error) => error.field, 'field', 'feeds'),
        ),
      );
    }

    expect(bridge.streams.activeStreamCount, 0);
    expect(bridge.streams.pendingCancellationCount, 0);

    final client = Client(adapter);
    await expectLater(
      client.subscribe(
        Subscription(
          markets: [Market.perpetual(Exchange.binance, 'BTC', 'USDT')],
        ),
      ),
      throwsA(
        isA<InvalidRequestError>().having(
          (error) => error.field,
          'field',
          'feeds',
        ),
      ),
    );
  });

  test('Dart Adapter의 입출금 단건 조회와 출금 취소를 Rust 경계로 왕복한다', () async {
    final adapter = TransferLookupAdapter();
    final bridge = DartAdapterBridge(adapter);
    final nativeClient = await bridge.register();

    final depositRequest = wire_model.WireTransferLookupRequest(
      asset: 'BTC',
      id: 'deposit-1',
    );
    final deposit = await nativeClient.deposit(request: depositRequest);
    expect(deposit.id, 'deposit-1');
    expect(deposit.asset, 'BTC');
    expect(adapter.depositRequest?.id, 'deposit-1');

    final withdrawalRequest = wire_model.WireTransferLookupRequest(
      asset: 'BTC',
      txId: 'tx-1',
    );
    final withdrawal = await nativeClient.withdrawal(
      request: withdrawalRequest,
    );
    expect(withdrawal.id, 'withdrawal-1');
    expect(withdrawal.txId, 'tx-1');
    expect(adapter.withdrawalRequest?.txId, 'tx-1');

    await nativeClient.cancelWithdrawal(withdrawalId: 'withdrawal-1');
    expect(adapter.cancelledWithdrawalId, 'withdrawal-1');
  });

  test('Dart Adapter는 참조값이 하나가 아닌 입출금 조회를 받지 않는다', () async {
    final adapter = TransferLookupAdapter();
    final bridge = DartAdapterBridge(adapter);
    final nativeClient = await bridge.register();

    await expectLater(
      nativeClient.deposit(
        request: const wire_model.WireTransferLookupRequest(asset: 'BTC'),
      ),
      throwsA(
        isA<wire.NativeError>()
            .having(
              (error) => error.kind,
              'kind',
              wire.NativeErrorKind.invalidRequest,
            )
            .having((error) => error.field, 'field', 'reference'),
      ),
    );
    await expectLater(
      nativeClient.withdrawal(
        request: const wire_model.WireTransferLookupRequest(
          asset: 'BTC',
          id: 'withdrawal-1',
          txId: 'tx-1',
        ),
      ),
      throwsA(
        isA<wire.NativeError>()
            .having(
              (error) => error.kind,
              'kind',
              wire.NativeErrorKind.invalidRequest,
            )
            .having((error) => error.field, 'field', 'reference'),
      ),
    );
    expect(adapter.depositRequest, isNull);
    expect(adapter.withdrawalRequest, isNull);
  });
}
