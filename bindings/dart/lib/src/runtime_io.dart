import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

import 'models.dart';
import 'rust/frb_generated.dart';

StreamConfig defaultStreamConfig() => const StreamConfig();
bool get bridgeCustomAdapters => true;

@Native<Void Function()>(
  symbol: 'maxt_dart_bridge_force_load',
  assetId: 'package:maxt/src/rust/frb_generated.io.dart',
)
external void _forceLoadNativeAsset();

@Native<Pointer<Utf8> Function()>(
  symbol: 'maxt_dart_bridge_library_path',
  assetId: 'package:maxt/src/rust/frb_generated.io.dart',
)
external Pointer<Utf8> _nativeLibraryPath();

/// Manages the lifecycle of the maxt native runtime.
abstract final class Maxt {
  static Future<void>? _initialization;
  static bool _initialized = false;
  static bool _disposed = false;
  static String? _relayUrl;
  static bool? _allowInsecureBrowserCredentials;

  /// Whether native runtime initialization completed in this isolate.
  static bool get isInitialized => _initialized;

  /// Initializes the native runtime once in the current isolate.
  static Future<void> initialize({
    String? relayUrl,
    bool allowInsecureBrowserCredentials = false,
  }) {
    if (_disposed) {
      return Future.error(
        StateError('The native maxt runtime was disposed in this isolate.'),
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
    return _initialization = _initialize();
  }

  static Future<void> _initialize() async {
    _forceLoadNativeAsset();
    await MaxtRustLib.init(externalLibrary: _externalLibrary());
    _initialized = true;
  }

  static ExternalLibrary _externalLibrary() {
    if (Platform.isIOS) {
      return ExternalLibrary.process(iKnowHowToUseIt: true);
    }

    final pointer = _nativeLibraryPath();
    if (pointer == nullptr) {
      throw StateError('Could not locate the native maxt library.');
    }

    final path = pointer.toDartString();
    if (Platform.isMacOS &&
        !path.endsWith('.dylib') &&
        !path.contains('.framework/')) {
      return ExternalLibrary.process(iKnowHowToUseIt: true);
    }
    return ExternalLibrary.open(path);
  }

  /// Releases native runtime resources for the current isolate.
  ///
  /// Call once when the isolate exits. It cannot be initialized again afterward.
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

/// Browser credential policy does not apply on native platforms.
void validateBrowserCredentials(String? first, String? second) {}
