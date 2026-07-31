pub mod api;
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

mod adapter;
mod convert;
mod stream;

/// Native Assets가 standalone Dart 프로세스에 라이브러리를 먼저 적재하도록 하는 심벌입니다.
#[unsafe(no_mangle)]
pub extern "C" fn maxt_dart_bridge_force_load() {}
