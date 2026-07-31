import 'package:maxt/maxt.dart';
import 'package:maxt/src/rust/api.dart' as native;
import 'package:maxt/src/rust/stream.dart' as native_stream;
import 'package:test/test.dart';

void main() {
  setUpAll(Maxt.initialize);
  tearDownAll(Maxt.dispose);

  test('native close는 대기 중인 next를 End로 깨우고 정리를 기다린다', () async {
    final subscription = native.pendingMarketSubscriptionForTest();
    final next = native.nativeMarketSubscriptionNext(
      subscription: subscription,
    );

    await Future<void>.delayed(Duration.zero);
    await native
        .nativeMarketSubscriptionClose(subscription: subscription)
        .timeout(const Duration(seconds: 1));
    final item = await next.timeout(const Duration(seconds: 1));

    expect(item, isA<native_stream.WireMarketStreamItem_End>());
    await native.nativeMarketSubscriptionClose(subscription: subscription);
  });
}
