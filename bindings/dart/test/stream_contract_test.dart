import 'dart:async';

import 'package:maxt/maxt.dart';
import 'package:test/test.dart';

void main() {
  test('원본 Dart 오류도 tagged error로 바뀌고 이후 이벤트가 계속된다', () async {
    final source = StreamController<StreamItem<MarketEvent>>();
    final stream = MarketStream(source.stream);
    final items = <StreamItem<MarketEvent>>[];
    final done = Completer<void>();
    stream.listen(items.add, onDone: done.complete, cancelOnError: true);

    source
      ..add(const StreamItem.event(MarketEvent.reconnected()))
      ..addError(const DecodeError('손상된 프레임'))
      ..add(const StreamItem.event(MarketEvent.reconnected()));
    await source.close();
    await done.future;

    expect(items, hasLength(3));
    expect(items[1], isA<StreamError<MarketEvent>>());
    expect((items[1] as StreamError<MarketEvent>).error, isA<DecodeError>());
  });

  test('StreamSubscription 취소도 비동기 close를 정확히 한 번 실행한다', () async {
    final source = StreamController<StreamItem<MarketEvent>>();
    var closeCount = 0;
    final stream = MarketStream(
      source.stream,
      onClose: () async => closeCount++,
    );
    final subscription = stream.listen((_) {});

    await subscription.cancel();
    await stream.close();

    expect(closeCount, 1);
    expect(source.hasListener, isFalse);
  });

  test('native close로 pending next를 깨운 뒤 Dart 구독 취소를 완료한다', () async {
    final nativeClosed = Completer<void>();
    final source = StreamController<StreamItem<MarketEvent>>(
      onCancel: () => nativeClosed.future,
    );
    var closeCount = 0;
    final stream = MarketStream(
      source.stream,
      onClose: () {
        closeCount++;
        if (!nativeClosed.isCompleted) nativeClosed.complete();
      },
    );
    stream.listen((_) {});

    final close = stream.close();
    await Future<void>.delayed(const Duration(milliseconds: 10));
    final closeCountBeforeCancelCompletes = closeCount;
    if (!nativeClosed.isCompleted) nativeClosed.complete();
    await close;

    expect(closeCountBeforeCancelCompletes, 1);
    expect(closeCount, 1);
    expect(source.hasListener, isFalse);
    await source.close();
  });

  test('close callback 오류가 발생해도 원본 구독을 취소한다', () async {
    final source = StreamController<StreamItem<MarketEvent>>();
    final stream = MarketStream(
      source.stream,
      onClose: () async => throw StateError('close failed'),
    );
    stream.listen((_) {});

    await expectLater(stream.close(), throwsA(isA<StateError>()));

    expect(source.hasListener, isFalse);
    await source.close();
  });

  test('natural done의 close 오류를 zone에 누출하지 않고 명시적 close에 보존한다', () async {
    final zoneErrors = <Object>[];
    Object? explicitCloseError;

    await runZonedGuarded(() async {
      final stream = MarketStream(
        const Stream.empty(),
        onClose: () async => throw StateError('close failed'),
      );

      await stream.toList();
      await Future<void>.delayed(Duration.zero);
      try {
        await stream.close();
      } catch (error) {
        explicitCloseError = error;
      }
    }, (error, _) => zoneErrors.add(error));

    expect(zoneErrors, isEmpty);
    expect(explicitCloseError, isA<StateError>());
  });
}
