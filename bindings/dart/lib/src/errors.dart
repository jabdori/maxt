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

/// 요청이 프로세스를 떠나기 전에 거절됐습니다.
final class InvalidRequestError extends MaxtError {
  const InvalidRequestError({required this.field, required String detail})
    : super(detail);

  final String field;

  @override
  String toString() => 'invalid request: `$field`: $detail';
}

/// 인증된 요청을 로컬에서 만들 수 없었습니다.
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

/// 거래소 오류의 재시도 분류입니다.
enum ExchangeErrorKind { rejected, rateLimited, unavailable, unknown }

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
