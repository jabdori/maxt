import 'errors.dart';
import 'models.dart';
import 'rust/api.dart' as native;
import 'rust/convert.dart' as wire;
import 'rust/frb_generated.dart';

StreamConfig defaultStreamConfig() =>
    const StreamConfig(overflow: Overflow.dropNewest);
bool get bridgeCustomAdapters => false;

/// maxt WebAssembly 런타임의 생명 주기를 관리합니다.
abstract final class Maxt {
  static Future<void>? _initialization;
  static bool _initialized = false;
  static bool _disposed = false;
  static String? _relayUrl;
  static bool? _allowInsecureBrowserCredentials;

  /// WebAssembly 런타임 초기화가 완료됐는지 나타냅니다.
  static bool get isInitialized => _initialized;

  /// 현재 브라우저 실행 환경에서 WebAssembly 런타임을 한 번 초기화합니다.
  static Future<void> initialize({
    String? relayUrl,
    bool allowInsecureBrowserCredentials = false,
  }) {
    if (_disposed) {
      return Future.error(
        StateError('The WebAssembly maxt runtime was disposed.'),
      );
    }
    final initialization = _initialization;
    if (initialization != null) {
      if (_relayUrl != relayUrl ||
          _allowInsecureBrowserCredentials != allowInsecureBrowserCredentials) {
        return Future.error(
          StateError('maxt was already initialized with different options.'),
        );
      }
      return initialization;
    }
    _relayUrl = relayUrl;
    _allowInsecureBrowserCredentials = allowInsecureBrowserCredentials;
    return _initialization = _initialize(relayUrl);
  }

  static Future<void> _initialize(String? relayUrl) async {
    await MaxtRustLib.init();
    if (relayUrl != null) {
      try {
        native.configureBrowserRelay(relayUrl: relayUrl);
      } on wire.NativeError catch (error, stackTrace) {
        Error.throwWithStackTrace(
          InvalidRequestError(
            field: error.field ?? 'relayUrl',
            detail: error.detail ?? error.message,
          ),
          stackTrace,
        );
      }
    }
    _initialized = true;
  }

  /// 현재 브라우저의 WebAssembly 런타임 자원을 정리합니다.
  static Future<void> dispose() async {
    final initialization = _initialization;
    if (initialization == null || _disposed) return;
    await initialization;
    if (_disposed) return;
    MaxtRustLib.dispose();
    _initialized = false;
    _disposed = true;
  }
}

/// 브라우저에서 인증 정보를 사용하기 위한 명시적 보안 설정을 검사합니다.
void validateBrowserCredentials(String? first, String? second) {
  if (first == null && second == null) return;
  if (Maxt._allowInsecureBrowserCredentials != true) {
    throw const InvalidRequestError(
      field: 'allowInsecureBrowserCredentials',
      detail: 'browser credentials require explicit initialize opt-in',
    );
  }
  if (Maxt._relayUrl == null) {
    throw const InvalidRequestError(
      field: 'relayUrl',
      detail: 'browser credentials require a relay URL',
    );
  }
}
