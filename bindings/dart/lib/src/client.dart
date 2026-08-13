import 'adapter.dart';
import 'generated_client.dart';

/// 하나의 거래소 어댑터를 공통 API로 노출합니다.
///
/// 생성하기 전에 `await Maxt.initialize()`로 native 런타임을
/// 초기화해야 합니다.
final class Client<A extends Adapter> extends GeneratedClient<A> {
  /// [adapter]의 공통 API를 노출하는 클라이언트를 만듭니다.
  Client(super.adapter);
}
