import 'adapter.dart';
import 'generated_client.dart';

/// Exposes one exchange adapter through the common API.
///
/// Initialize the native runtime with `await Maxt.initialize()` before creating it.
final class Client<A extends Adapter> extends GeneratedClient<A> {
  /// Creates a client that exposes the common API of [adapter].
  Client(super.adapter);
}
