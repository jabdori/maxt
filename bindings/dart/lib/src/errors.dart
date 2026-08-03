import 'models.dart';

/// maxt 작업 실패의 공통 기반입니다.
abstract class MaxtError implements Exception {
  const MaxtError(this.detail);

  final String detail;
  bool get isRetryable => false;
  bool get isRateLimited => false;

  @override
  String toString() => detail;
}

/// 요청 값 검증에 실패했습니다. 같은 요청을 변경 없이 재시도해도 실패합니다.
final class InvalidRequestError extends MaxtError {
  const InvalidRequestError({required this.field, required String detail})
    : super(detail);

  final String field;

  @override
  String toString() => 'invalid request: `$field`: $detail';
}

/// 어댑터가 기능 또는 요청 형식을 지원하지 않습니다.
final class UnsupportedError extends MaxtError {
  const UnsupportedError({
    required this.feature,
    required this.exchange,
    required String detail,
  }) : super(detail);

  final Feature feature;
  final Exchange exchange;
}

/// 인증 정보가 없거나 요청 서명에 실패했습니다.
final class AuthenticationError extends MaxtError {
  const AuthenticationError(super.detail);

  @override
  String toString() => 'authentication failed: $detail';
}

/// 외부 어댑터 경계 또는 바인딩 자체가 실패했습니다.
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

/// 거래소가 반환한 오류입니다.
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

/// DNS, TLS, 소켓 또는 시간 제한 실패입니다.
final class TransportError extends MaxtError {
  const TransportError(super.detail);

  @override
  bool get isRetryable => true;

  @override
  String toString() => 'transport failed: $detail';
}

/// 거래소 응답을 해석할 수 없었습니다.
final class DecodeError extends MaxtError {
  const DecodeError(super.detail);

  @override
  String toString() => 'could not read exchange response: $detail';
}
