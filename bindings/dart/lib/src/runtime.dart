import 'dart:ffi';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

import 'rust/frb_generated.dart';

@Native<Void Function()>(
  symbol: 'maxt_dart_bridge_force_load',
  assetId: 'package:maxt/src/rust/frb_generated.io.dart',
)
external void _forceLoadNativeAsset();

/// maxt 네이티브 런타임의 생명 주기를 관리합니다.
abstract final class Maxt {
  static Future<void>? _initialization;
  static bool _initialized = false;
  static bool _disposed = false;

  /// 현재 격리 실행 환경(isolate)에서 네이티브 런타임 초기화가 완료됐는지 나타냅니다.
  static bool get isInitialized => _initialized;

  /// 현재 isolate에서 네이티브 런타임을 한 번 초기화합니다.
  static Future<void> initialize() {
    if (_disposed) {
      return Future.error(
        StateError('The native maxt runtime was disposed in this isolate.'),
      );
    }
    return _initialization ??= _initialize();
  }

  static Future<void> _initialize() async {
    _forceLoadNativeAsset();
    await MaxtRustLib.init(
      externalLibrary: ExternalLibrary.process(iKnowHowToUseIt: true),
    );
    _initialized = true;
  }

  /// 현재 isolate의 네이티브 런타임 자원을 정리합니다.
  ///
  /// isolate 종료 시 한 번 호출합니다. 종료 후 같은 isolate에서 다시 초기화할 수
  /// 없습니다.
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
