import 'dart:async';

import 'package:maxt/src/adapters.dart';
import 'package:test/test.dart';

void main() {
  test('CancelStream이 Subscribe 등록보다 먼저 와도 나중 stream을 한 번 닫는다', () async {
    final registry = DartStreamRegistry();
    var closeCount = 0;

    await registry.cancel('7');
    final accepted = await registry.register('7', () async => closeCount++);

    expect(accepted, isFalse);
    expect(closeCount, 1);
    expect(registry.activeStreamCount, 0);
    expect(registry.pendingCancellationCount, 0);
  });

  test('Subscribe 등록 후 중복 CancelStream은 같은 close를 공유한다', () async {
    final registry = DartStreamRegistry();
    final release = Completer<void>();
    var closeCount = 0;
    expect(
      await registry.register('8', () async {
        closeCount++;
        await release.future;
      }),
      isTrue,
    );

    final first = registry.cancel('8');
    final second = registry.cancel('8');
    await Future<void>.delayed(Duration.zero);
    expect(closeCount, 1);

    release.complete();
    await Future.wait([first, second]);
    expect(closeCount, 1);
    expect(registry.activeStreamCount, 0);
    expect(registry.pendingCancellationCount, 0);
  });

  test('Subscribe 등록 실패를 알면 cancel tombstone을 누적하지 않는다', () async {
    final registry = DartStreamRegistry();

    for (var index = 0; index < 100; index++) {
      final id = index.toString();
      await registry.cancel(id);
      registry.forgetPending(id);
    }

    expect(registry.pendingCancellationCount, 0);
  });

  test('cancel close 중 pump finish가 들어와도 자기 Future를 기다리지 않는다', () async {
    final registry = DartStreamRegistry();
    var closeCount = 0;
    await registry.register('cycle', () async {
      closeCount++;
      await registry.finish('cycle');
    });

    await registry.cancel('cycle').timeout(const Duration(seconds: 1));

    expect(closeCount, 1);
    expect(registry.activeStreamCount, 0);
    expect(registry.pendingCancellationCount, 0);
  });

  test('close 오류를 그대로 전파하면서 active·pending 상태를 정리한다', () async {
    final registry = DartStreamRegistry();

    await registry.cancel('late');
    await expectLater(
      registry.register('late', () async => throw StateError('close failed')),
      throwsStateError,
    );
    await registry.register(
      'active',
      () async => throw StateError('close failed'),
    );
    await expectLater(registry.cancel('active'), throwsStateError);

    expect(registry.activeStreamCount, 0);
    expect(registry.pendingCancellationCount, 0);
  });
}
