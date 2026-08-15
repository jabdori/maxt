import 'errors.dart';
import 'models.dart';
import 'rust/api.dart' as native;
import 'rust/convert.dart' as wire;
import 'rust/frb_generated.dart';

StreamConfig defaultStreamConfig() =>
    const StreamConfig(overflow: Overflow.dropNewest);
bool get bridgeCustomAdapters => false;

/// Manages the lifecycle of the maxt WebAssembly runtime.
abstract final class Maxt {
  static Future<void>? _initialization;
  static bool _initialized = false;
  static bool _disposed = false;
  static String? _relayUrl;
  static bool? _allowInsecureBrowserCredentials;

  /// Whether WebAssembly runtime initialization has completed.
  static bool get isInitialized => _initialized;

  /// Initializes the WebAssembly runtime once in the current browser context.
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

  /// Releases WebAssembly runtime resources in the current browser context.
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

/// Validates the explicit security opt-in required for browser credentials.
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
