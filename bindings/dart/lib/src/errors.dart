import 'models.dart';

/// Common base class for maxt operation failures.
abstract class MaxtError implements Exception {
  const MaxtError(this.detail);

  final String detail;
  bool get isRetryable => false;
  bool get isRateLimited => false;

  @override
  String toString() => detail;
}

/// Request-value validation failed. Retrying the unchanged request also fails.
final class InvalidRequestError extends MaxtError {
  const InvalidRequestError({required this.field, required String detail})
    : super(detail);

  final String field;

  @override
  String toString() => 'invalid request: `$field`: $detail';
}

/// Transfer-safety validation failed before submitting a withdrawal.
final class TransferError extends MaxtError {
  const TransferError({required this.kind, required String detail})
    : super(detail);

  final TransferErrorKind kind;

  @override
  String toString() => 'transfer ${kind.wireName}: $detail';
}

/// The adapter does not support the feature or request shape.
final class UnsupportedError extends MaxtError {
  const UnsupportedError({
    required this.feature,
    required this.exchange,
    required String detail,
  }) : super(detail);

  final Feature feature;
  final Exchange exchange;
}

/// Credentials are missing or request signing failed.
final class AuthenticationError extends MaxtError {
  const AuthenticationError(super.detail);

  @override
  String toString() => 'authentication failed: $detail';
}

/// An external adapter boundary or the binding itself failed.
final class AdapterError extends MaxtError {
  const AdapterError(super.detail);

  @override
  String toString() => 'adapter failed: $detail';
}

extension ExchangeErrorKindProperties on ExchangeErrorKind {
  bool get isRetryable =>
      this == ExchangeErrorKind.rateLimited ||
      this == ExchangeErrorKind.unavailable;
}

/// Error returned by an exchange.
final class ExchangeError extends MaxtError {
  const ExchangeError({
    required this.exchange,
    required this.code,
    required this.message,
    required this.kind,
    this.status,
  }) : super(message);

  final Exchange exchange;
  final String code;
  final String message;
  final int? status;
  final ExchangeErrorKind kind;

  @override
  bool get isRetryable => kind.isRetryable;

  @override
  bool get isRateLimited => kind == ExchangeErrorKind.rateLimited;

  @override
  String toString() => status == null
      ? '${exchange.id} returned $code: $message'
      : '${exchange.id} returned $status $code: $message';
}

/// DNS, TLS, socket, or timeout failure.
final class TransportError extends MaxtError {
  const TransportError(super.detail);

  @override
  bool get isRetryable => true;

  @override
  String toString() => 'transport failed: $detail';
}

/// Exchange response could not be parsed.
final class DecodeError extends MaxtError {
  const DecodeError(super.detail);

  @override
  String toString() => 'could not read exchange response: $detail';
}
