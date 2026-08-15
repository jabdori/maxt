import 'dart:async';

import 'models.dart';

part 'generated_provider_streams.dart';

/// Event or non-terminal error delivered by a stream.
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

/// Single-subscription stream that delivers events and errors as items and can be explicitly closed.
///
/// A [StreamError] item does not terminate the stream.
abstract interface class CloseableStream<T> implements Stream<StreamItem<T>> {
  /// Closes the stream and waits for subscription-resource cleanup.
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

/// Market-data subscription stream.
final class MarketStream extends _CloseableStream<MarketEvent> {
  /// Creates a market stream from an internal adapter event source.
  ///
  /// Normal applications receive this stream from [Client.subscribe].
  MarketStream(super.source, {super.onClose});
}

/// Private-account subscription stream.
final class AccountStream extends _CloseableStream<AccountEvent> {
  /// Creates an account stream from an internal adapter event source.
  ///
  /// Normal applications receive this stream from [Client.subscribeAccount].
  AccountStream(super.source, {super.onClose});
}

/// Single-subscription stream that also preserves Hyperliquid-native market fields.
final class HyperliquidMarketStream
    extends _CloseableStream<HyperliquidMarketEvent> {
  /// Creates a stream from an internal Hyperliquid event source.
  HyperliquidMarketStream(super.source, {super.onClose});
}

/// Single-subscription stream that also preserves Hyperliquid-native account fields.
final class HyperliquidAccountStream
    extends _CloseableStream<HyperliquidAccountEvent> {
  /// Creates a stream from an internal Hyperliquid event source.
  HyperliquidAccountStream(super.source, {super.onClose});
}
