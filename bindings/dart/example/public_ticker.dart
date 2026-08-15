import 'package:maxt/maxt.dart';

/// `true`일 때만 Binance 현물 공개 시세를 한 번 읽습니다.
///
/// 기본 실행은 네이티브 라이브러리를 초기화하거나 네트워크에 연결하지 않습니다.
/// 실제 공개 읽기는 다음처럼 명시적으로 켤 수 있습니다.
///
/// ```sh
/// dart run -DMAXT_RUN_PUBLIC_READ=true example/public_ticker.dart
/// ```
const _runPublicRead = bool.fromEnvironment('MAXT_RUN_PUBLIC_READ');

Future<void> main() async {
  final market = Market.spot(Exchange.binance, 'BTC', 'USDT');
  print('준비된 공개 시장: $market');

  if (!_runPublicRead) {
    print('공개 시세 읽기는 MAXT_RUN_PUBLIC_READ=true로 명시적으로 켜주세요.');
    return;
  }

  try {
    await Maxt.initialize();

    // 인증 정보와 주문·이체 호출 없이 공개 시세만 읽습니다.
    final client = Client(BinanceAdapter.spot());
    final ticker = await client.ticker(market);
    final average = await client.adapter.spotAveragePrice(market);
    print('${ticker.market}: ${ticker.lastPrice}');
    print('Binance ${average.minutes}-minute average: ${average.price}');
  } on MaxtError catch (error) {
    print('공개 시세를 읽지 못했습니다: $error');
  } catch (error) {
    print('예기치 않은 오류가 발생했습니다: $error');
  } finally {
    if (Maxt.isInitialized) await Maxt.dispose();
  }
}
