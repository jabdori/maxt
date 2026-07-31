import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

final class BareAdapter extends AdapterBase {
  @override
  Exchange get exchange => Exchange.upbit;

  @override
  Set<Feature> get features => const {Feature.markets};
}

void main() {
  test('native 초기화 전에는 Dart Adapter Client 생성을 명확히 거절한다', () {
    expect(
      () => Client(BareAdapter()),
      throwsA(
        isA<StateError>().having(
          (error) => error.message,
          'message',
          contains('await Maxt.initialize()'),
        ),
      ),
    );
  });

  group('native 초기화 후', () {
    setUpAll(Maxt.initialize);

    test('Client는 사용자 Adapter 원 객체를 보존한다', () {
      final adapter = BareAdapter();
      final client = Client(adapter);

      expect(client.adapter, same(adapter));
      expect(client.exchange, Exchange.upbit);
      expect(client.supports(Feature.markets), isTrue);
      expect(client.supports(Feature.balances), isFalse);
    });
  });

  test('Adapter 기본 메서드는 해당 기능의 UnsupportedError를 반환한다', () async {
    final adapter = BareAdapter();

    await expectLater(
      adapter.balances(),
      throwsA(
        isA<UnsupportedError>().having(
          (error) => error.message,
          'message',
          'upbit has no endpoint for balances',
        ),
      ),
    );
  });
}
