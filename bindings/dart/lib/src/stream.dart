import 'dart:async';

import 'models.dart';

/// 스트림에서 전달되는 이벤트 또는 스트림을 종료하지 않는 오류입니다.
sealed class StreamItem<T> {
  const StreamItem();

  const factory StreamItem.event(T event) = StreamEvent<T>;
  const factory StreamItem.error(Object error) = StreamError<T>;
}

final class StreamEvent<T> extends StreamItem<T> {
  const StreamEvent(this.event);
  final T event;
}

final class StreamError<T> extends StreamItem<T> {
  const StreamError(this.error);
  final Object error;
}

/// 이벤트와 오류를 항목으로 전달하고 명시적으로 닫을 수 있는 단일 구독 스트림입니다.
///
/// [StreamError] 항목은 스트림을 종료하지 않습니다.
abstract interface class CloseableStream<T> implements Stream<StreamItem<T>> {
  /// 스트림을 닫고 구독과 관련 자원의 정리가 끝날 때까지 기다립니다.
  Future<void> close();
}

base class _CloseableStream<T> extends Stream<StreamItem<T>>
    implements CloseableStream<T> {
  _CloseableStream(
    Stream<StreamItem<T>> source, {
    FutureOr<void> Function()? onClose,
  }) : _source = source.transform(
         StreamTransformer.fromHandlers(
           handleError: (error, stackTrace, sink) {
             sink.add(StreamItem<T>.error(error));
           },
         ),
       ),
       _onClose = onClose;

  final Stream<StreamItem<T>> _source;
  final FutureOr<void> Function()? _onClose;
  StreamSubscription<StreamItem<T>>? _subscription;
  Future<void>? _closing;
  bool _closed = false;
  bool _listened = false;

  @override
  StreamSubscription<StreamItem<T>> listen(
    void Function(StreamItem<T>)? onData, {
    Function? onError,
    void Function()? onDone,
    bool? cancelOnError,
  }) {
    if (_listened) {
      throw StateError('maxt streams are single-subscription streams');
    }
    _listened = true;
    if (_closed) {
      return Stream<StreamItem<T>>.empty().listen(
        onData,
        onError: onError,
        onDone: onDone,
        cancelOnError: cancelOnError,
      );
    }
    final subscription = _source.listen(
      onData,
      onError: onError,
      onDone: () {
        _closed = true;
        final closing = _closing ??= _close(cancelSubscription: false);
        unawaited(
          closing.then<void>((_) {}, onError: (Object _, StackTrace _) {}),
        );
        onDone?.call();
      },
      cancelOnError: cancelOnError,
    );
    _subscription = subscription;
    return _ManagedSubscription(subscription, close);
  }

  @override
  Future<void> close() => _closing ??= _close();

  Future<void> _close({bool cancelSubscription = true}) async {
    _closed = true;
    Object? closeError;
    StackTrace? closeStack;
    try {
      await _onClose?.call();
    } catch (error, stackTrace) {
      closeError = error;
      closeStack = stackTrace;
    }
    try {
      if (cancelSubscription) await _subscription?.cancel();
    } catch (error, stackTrace) {
      closeError ??= error;
      closeStack ??= stackTrace;
    }
    if (closeError != null) {
      Error.throwWithStackTrace(closeError, closeStack!);
    }
  }
}

final class _ManagedSubscription<T> implements StreamSubscription<T> {
  const _ManagedSubscription(this._inner, this._close);

  final StreamSubscription<T> _inner;
  final Future<void> Function() _close;

  @override
  Future<void> cancel() => _close();

  @override
  void onData(void Function(T data)? handleData) => _inner.onData(handleData);

  @override
  void onError(Function? handleError) => _inner.onError(handleError);

  @override
  void onDone(void Function()? handleDone) => _inner.onDone(handleDone);

  @override
  void pause([Future<void>? resumeSignal]) => _inner.pause(resumeSignal);

  @override
  void resume() => _inner.resume();

  @override
  bool get isPaused => _inner.isPaused;

  @override
  Future<E> asFuture<E>([E? futureValue]) => _inner.asFuture(futureValue);
}

/// 시장 데이터 구독 스트림입니다.
final class MarketStream extends _CloseableStream<MarketEvent> {
  /// 내부 어댑터 이벤트 소스로 시장 스트림을 만듭니다.
  ///
  /// 일반 애플리케이션에서는 [Client.subscribe]가 이 스트림을 반환합니다.
  MarketStream(super.source, {super.onClose});
}

/// 비공개 계정 구독 스트림입니다.
final class AccountStream extends _CloseableStream<AccountEvent> {
  /// 내부 어댑터 이벤트 소스로 계정 스트림을 만듭니다.
  ///
  /// 일반 애플리케이션에서는 [Client.subscribeAccount]가 이 스트림을 반환합니다.
  AccountStream(super.source, {super.onClose});
}
