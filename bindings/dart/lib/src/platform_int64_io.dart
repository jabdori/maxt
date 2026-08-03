import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

PlatformInt64 platformInt64FromBigInt(BigInt value) => value.toInt();
BigInt platformInt64ToBigInt(PlatformInt64 value) => BigInt.from(value);
