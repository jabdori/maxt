import 'dart:ffi';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

import 'rust/frb_generated.dart';

@Native<Void Function()>(
  symbol: 'maxt_dart_bridge_force_load',
  assetId: 'package:maxt/src/rust/frb_generated.io.dart',
)
external void _forceLoadNativeAsset();

/// native maxt 런타임을 초기화합니다.
abstract final class Maxt {
  static Future<void>? _initialization;
  static bool _initialized = false;

  /// 현재 isolate에서 native 런타임 초기화가 완료됐는지 나타냅니다.
  static bool get isInitialized => _initialized;

  /// 현재 isolate에서 native 라이브러리를 한 번 초기화합니다.
  static Future<void> initialize() => _initialization ??= _initialize();

  static Future<void> _initialize() async {
    _forceLoadNativeAsset();
    await MaxtRustLib.init(
      externalLibrary: ExternalLibrary.process(iKnowHowToUseIt: true),
    );
    _initialized = true;
  }
}
