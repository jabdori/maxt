# maxt for Dart and Flutter

[English](README.md) | [한국어](README.ko.md)

One Dart and Flutter API for the same operations, models, errors, and streams
on native platforms and the Web. Native builds use Dart build hooks; Web builds
use WebAssembly.

## Install

```sh
dart pub add maxt
```

For a Web application, build the package's WebAssembly files into `web/pkg`
before running or building the application:

```sh
rustup toolchain install nightly --component rust-src --target wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
dart run maxt:build_web --release
flutter build web
```

Run `dart run maxt:build_web --release` from the application root. The command
uses the installed `maxt` package, so it also works when `maxt` comes from
pub.dev. Do not commit the generated `web/pkg` files unless your deployment
process requires built assets in source control.

Serve the Web build with `Cross-Origin-Opener-Policy: same-origin` and
`Cross-Origin-Embedder-Policy: require-corp` so browsers can enable the shared
memory features used by the generated WebAssembly module. Use HTTPS in
production; `http://localhost` is suitable for local development.

## First read: Binance Spot

Initialize once in each isolate before constructing an adapter, then read
public `BTC/USDT` data. This needs no credentials and does not submit an order.

```dart
import 'package:maxt/maxt.dart';

Future<void> main() async {
  await Maxt.initialize();
  try {
    final market = Market.spot(Exchange.binance, 'BTC', 'USDT');
    final client = Client(BinanceAdapter.spot());

    final ticker = await client.ticker(market); // common API
    final average = await client.adapter.spotAveragePrice(market); // Binance-only API

    print(ticker.lastPrice);
    print('${average.minutes}-minute average: ${average.price}');
  } finally {
    await Maxt.dispose();
  }
}
```

The checked-in [public Binance example](example/public_ticker.dart) keeps
network access opt-in. Run its public read explicitly:

```sh
dart run -DMAXT_RUN_PUBLIC_READ=true example/public_ticker.dart
```

Use `Client` for common calls and `client.adapter` for provider-specific calls.
A disposed isolate cannot be initialized again.

## Support

- [x] Android
- [x] iOS
- [x] Linux
- [x] macOS
- [x] Windows
- [x] Dart Web

Dart 3.10 or a compatible Flutter SDK is required. This package does not
download prebuilt native libraries. Its build hook compiles the included Rust
source when your Dart or Flutter application is built, so Rustup and the target
platform toolchain, such as the Android NDK or Xcode, must also be installed in
development and CI environments. Web builds additionally require the Rust
nightly toolchain with `rust-src` and `wasm-pack`.

## Supported exchanges

- Binance Spot and USD-M perpetual futures
- Upbit Spot: Korea, Singapore, Indonesia, and Thailand
- Bithumb Spot
- Hyperliquid Spot and perpetual futures on mainnet and testnet

Binance testnet constructors are not exposed. Hyperliquid HIP-3 perpetual DEXs
and outcome assets are not exposed.

## Package map

| Need | Use |
| --- | --- |
| Public market data and streams | `Client` with an adapter |
| Exchange-only fields or endpoints | `client.adapter` |
| Exact prices and quantities | `Decimal`, not `double` |
| Timestamps | `Timestamp` with `BigInt` nanoseconds |
| Native lifecycle | `await Maxt.initialize()` and `await Maxt.dispose()` |
| Endpoint support and constraints | [generated endpoint reference](../common/generated/api.md) |

`Client` provides normalized calls shared by all adapters. Provider-specific
methods remain on the concrete adapter to preserve exchange-only fields.

## Authentication boundary

Public calls need no credentials. Signed account, order, and wallet operations
require both credential fields. Hyperliquid also exposes the address-scoped,
unsigned `/info` reads listed below; they require a public `address`, not a
private key. Credentialed browser calls additionally require a relay and
`allowInsecureBrowserCredentials: true`. Use `client.supports(feature)` before
optional operations when the adapter or credential state is dynamic.

## Common API

`Client` provides the same method names for every built-in adapter:

- Public REST: `markets()`, `trades()`, `orderBook()`, `ticker()`, and
  `candles()`.
- Public streams: `subscribe()` and `subscribeWith()` for trades, order books,
  tickers, and candles. Bithumb does not support candle streams.
- Public funding history: `fundingRates()` on Binance USD-M and Hyperliquid
  perpetual markets.
- Private Spot: `balances()`, `openOrders()`, `placeOrder()`, `cancelOrder()`,
  and `subscribeAccount()` on every exchange.
- Private order lookup: `order()`, `orderByClientId()`, `ordersByIds()`, and
  `orderHistory()` on Upbit and Bithumb.
- Private order rules: `orderRules()` on Upbit and Bithumb.
- Private batch cancellation: `cancelOrders()` on Upbit and Bithumb.
- Private wallet lookup and cancellation: `deposit()`, `withdrawal()`, and
  `cancelWithdrawal()` on Upbit and Bithumb. Lookups require an asset and one
  exchange ID or transaction ID; cancellation must be followed by a lookup.
- Private perpetuals: `positions()`, `marginSummary()`, `setMargin()`, and
  `fundingPayments()` on Binance USD-M and Hyperliquid.

## Exchange-specific API

Exchange-specific methods remain available through `client.adapter`.

| Adapter | Construction | Additional methods |
| --- | --- | --- |
| `BinanceAdapter` | `BinanceAdapter.spot()` | Public: `aggregateTrades()`, `spotAveragePrice()`, `spotSymbolFilters()`, `spotExchangeInfo()`; authenticated: `spotOrder()`, `spotAccountInformation()`, `spotCancelAllOpenOrders()`, `accountTrades()`, `c2cTradeHistory()`, `testOrder()`, `cancelAllOpenOrders()`; Wallet: `allCoinsInformation()`, `apiKeyPermissions()`, `depositHistory()`, `questionnaireRequirements()`, `withdrawAddressList()`, `withdrawHistory()` |
| `BinanceAdapter` | `BinanceAdapter.usdMFutures()` | Public: `markPrice()`, `markPrices()`, `openInterest()`, `aggregateTrades()`, `usdMExchangeInfo()`; authenticated: `usdMAccountInformation()`, `usdMPositionInformation()`, `accountTrades()`, `testOrder()`, `cancelAllOpenOrders()`, `usdMCreateListenKey()`, `usdMKeepaliveListenKey()`, `usdMCloseListenKey()` |
| `UpbitAdapter` | `UpbitAdapter()` or `UpbitAdapter.withRegion(...)` | `orderBooks()`, `orderBooksAtLevel()`, `tickers()`, `tickersByQuote()`, `yearCandles()`, `orderbookInstruments()`, `marketEvents()`; authenticated: `testOrder()`, `orderDetail()`, `closedOrders()`, `depositInfo()`, `withdrawalAddresses()`, `travelRuleVasps()`, `verifyTravelRuleByUuid()`, `verifyTravelRuleByTxid()`, `batchCancelOpenOrders()`, `cancelAndNewOrder()`; Korea only: `depositKrw()`, `withdrawKrw()`, `apiKeys()`, `listPockets()`, `listPocketApiKeys()`, `subPocketBalances()`, `universalTransfer()`, `universalTransfers()`, `subPocketTransfer()`, `subPocketTransfers()` |
| `BithumbAdapter` | `BithumbAdapter()` | `marketWarnings()`, `marketAlerts()`, `notices()`, `transferFees()`; authenticated: `apiKeys()`, `withdrawalAddresses()`, `orderDetail()`, `orderList()`, `closedOrders()`, `krwWithdrawals()`, `withdrawKrw()`, `krwDeposits()`, `depositKrw()`, `pendingOrders()`, `batchOrders()`, `twapOrders()`, `createTwapOrder()`, `cancelTwapOrder()` |
| `HyperliquidAdapter` | `HyperliquidAdapter()` or `HyperliquidAdapter.testnet()` | Public: `allMids()`, `assetContext()`, `candleSnapshot()`, `l2Book()`, `recentTrades()`, `fundingHistory()`, `spotMeta()`, `spotMetaAndAssetContexts()`; address-scoped, unsigned reads: `userFunding()`, `spotClearinghouseState()`, `basicOpenOrders()`, `orderStatus(reference)`, `historicalOrders()`, `userFills()`, `userFillsByTime()`, `nonFundingLedger()`, `userRateLimit()`, `userRole()`, `referral()`, `userFees()`, `portfolio()`, `subAccounts()`, `userVaultEquities()` |

`UpbitAdapter.testOrder()` validates an order without creating it. The returned
`Order` is a dry-run result: do not query or cancel its `id`, and do not treat
its status as a live order.

`UpbitAdapter.orderDetail(request)` is the provider-specific authenticated
`GET /v1/order` read. Supply the expected market plus a UUID and/or identifier;
one identifier is required and Upbit gives UUID priority. It preserves detailed
fills, fees, locked amounts, SMP, and time-in-force raw fields absent from the
common `Order`; reserved identifier characters are safely encoded. Fixture-verified only.

`orderHistory()` remains the common normalized history API.
`UpbitAdapter.closedOrders(request)` complements it with official closed-order
summary fields, including fees, SMP, `identifier`, and time-in-force, but no
`trades` list. Its optional `market`, `state`, and `states` filters include
mutually exclusive `state` and `states`; the creation-time window is at most seven days,
`limit` is at most 1,000, and ordering can be ascending or descending.
`Timestamp` inputs are passed directly to Upbit as milliseconds, unlike the
common history API's exclusive-end adaptation. The official endpoint does not
state time-boundary inclusion, so this API makes no further boundary claim.
Fixture-verified only; maxt has not performed a live trade or read. See the
[Korea](https://docs.upbit.com/kr/reference/list-closed-orders) and
[Global](https://global-docs.upbit.com/reference/list-closed-orders) references.

`UpbitAdapter.depositInfo(asset, network)` returns the provider's deposit
availability, minimum amount, confirmation, and precision metadata. Upbit may
delay this information by several minutes; it is not a real-time service-status signal.

`UpbitAdapter.travelRuleVasps()` lists VASPs for Travel Rule verification.
The verification methods are financial writes and are available only in Korea
and Singapore; Indonesia and Thailand fail before a network request. These
paths are fixture-verified only.

`UpbitAdapter.batchCancelOpenOrders(request)` is a financial write.
`UpbitBatchCancelScope.all()` explicitly selects every eligible market; Upbit
still applies the request count (default 20, maximum 300 `wait` orders), and
the result preserves partial failures.

`UpbitAdapter.cancelAndNewOrder(request)` is a financial write using the JSON
endpoint. The replacement keeps the original market and side; `postOnly` and
SMP cannot be combined. A successful HTTP response may still have no new order
when the previous order fills before cancellation completes. This path is
fixture-verified only.

`UpbitAdapter.depositKrw(request)` and `withdrawKrw(request)` are Korea-only
financial writes. `UpbitKrwTransferRequest` requires a positive amount and a
`UpbitKrwTwoFactorType` of `kakao`, `naver`, or `hana`; the registered account
and second factor stay on Upbit. `apiKeys()` is a Korea-only authenticated read
of access-key identifiers and expiry times. All three paths are fixture-verified
only; no live transfer is submitted by maxt.

`listPockets()`, `listPocketApiKeys(request)`, and
`subPocketBalances(pocketUuid)` are Korea-only authenticated reads for pockets,
their API keys, and a sub-pocket balance. `universalTransfer(request)` and
`subPocketTransfer(request)` are Korea-only financial writes; both request
types require a destination `to` under Upbit's current OpenAPI contract.
`universalTransfers(request)` and `subPocketTransfers(request)` list the
corresponding transfer histories. These paths are fixture-verified only.

`BithumbAdapter.batchOrders(request)` accepts 1–20 orders and can return HTTP
200 with per-item failures; inspect every `BithumbBatchOrderOutcome`. Accepted
items preserve `timeInForce` and `stpType`; rejected items preserve returned
`timeInForce`. This is a fixture-verified financial write only.

`BithumbAdapter.twapOrders(request)` is an authenticated, read-only history
query for Bithumb's KRW markets. `createTwapOrder()` and
`cancelTwapOrder()` are financial writes; do not call them in a read-only
verification.

`BithumbAdapter.krwWithdrawals()` and `krwDeposits()` read KRW transfer
history. `withdrawKrw()` and `depositKrw()` are financial writes. Bithumb
requires its registered account and Kakao second-factor flow; maxt neither
accepts nor stores those credentials. These paths are fixture-verified only.

`BithumbAdapter.withdrawalAddresses()` is an authenticated, read-only list of
registered withdrawal allowlist addresses. It is distinct from
`prepareWithdrawal()`: it does not validate a prospective withdrawal or return
a common withdrawal quote. It is fixture-verified only.

`BithumbAdapter.orderDetail(request)` retains Bithumb's provider-specific fill,
fee, cancellation, self-trade-prevention, and time-in-force fields; the
normalized common `Order` intentionally does not carry them. The expected
market in the request is checked against the response. This path is
fixture-verified only.

`BithumbAdapter.orderList(request)` is the provider-specific authenticated
`GET /v1/orders` read, separate from common `openOrders()`. It supports an
optional market, either `state` or `states`, UUID/client-ID lists of up to 100
(UUIDs take priority), plus `page >= 1`, `limit` from 1 through 100, and
`orderBy`. Its
provider fields are retained rather than reduced to common `Order`. Fixture-verified only.

`orderHistory()` remains the common normalized history API.
`BithumbAdapter.closedOrders(request)` complements it with Bithumb's official
v2 fee, cancellation, self-trade-prevention, and time-in-force metadata. It
supports an optional `market`, mutually exclusive `state` or `states` (`states[]` query parameter), start/end
times at most seven days apart, `limit` from 1 through 1,000, `orderBy`, and an
opaque `next_key` cursor. Times go directly to Bithumb as milliseconds, unlike
the common history API's exclusive-end adaptation; time-boundary inclusion is
not claimed. The page preserves `data`, `has_next`, and `next_key`, plus raw
status/type strings and optional price, creation-time, client-order, and
cancellation fields. Fixture-verified only; maxt has not performed a live
account read or trade. See [closed orders](https://apidocs.bithumb.com/reference/%EC%A2%85%EB%A3%8C-%EC%A3%BC%EB%AC%B8-%EB%AA%A9%EB%A1%9D-%EC%A1%B0%ED%9A%8C.md) and
[authentication](https://apidocs.bithumb.com/docs/%EC%9D%B8%EC%A6%9D-%ED%86%A0%ED%81%B0-%EC%83%9D%EC%84%B1%ED%95%98%EA%B8%B0).

```dart
Future<void> readTwapHistory() async {
  final adapter = BithumbAdapter(
    accessKey: accessKey,
    secretKey: secretKey,
  );
  final market = Market.spot(Exchange.bithumb, 'BTC', 'KRW');
  final page = await adapter.twapOrders(
    BithumbTwapOrdersRequest(market: market, limit: 20),
  );
}
```

The Bithumb TWAP API accepts `progress`, `done`, or `cancel` states and uses a
page size from 1 through 100. Creation uses a 300–43,200 second duration and a
15/20/30/60/120 second interval; buys require `price`, sells require `volume`.

`BinanceAdapter.usdMFutures()` exposes `markPrice()`, `markPrices()`, and
`openInterest()` as public, read-only USD-M perpetual market-data calls. These
methods are fixture-verified; they have not been live-read verified.
`aggregateTrades(request)` is a public Spot and USD-M read returning the same
provider aggregate-trade type. Both venues use an inclusive `fromId` cursor or
inclusive time bounds (not both), with `limit` from 1 through 1,000 (`null`
defaults to 500). USD-M only retains the latest 48 hours and requires a time
window shorter than one hour; Spot has no equivalent local limit. This method is
fixture-verified only.
`accountTrades(request)` is a signed Spot or USD-M account-trade page with a
1–1,000 limit (default 500) and no safe generic continuation cursor.
`c2cTradeHistory(request)` is a signed, read-only Spot/Funding Wallet SAPI call
and is unavailable on `usdMFutures()`. It requires
`BinanceC2cTradeType.buy` or `.sell`, uses a one-based page with at most 100
rows, and permits inclusive timestamp bounds spanning at most 30 days. Its
nullable `code`, `message`, `data`, `total`, and `success` envelope is preserved
instead of being converted to a common cursor. This path is fixture-verified only.
`testOrder(BinanceTestOrderRequest(...))` is signed validation that does not
reach the matching engine; `computeCommissionRates` is Spot-only.
`cancelAllOpenOrders(market)` is a signed financial write for one market.
These three paths are fixture-verified only.
`HyperliquidAdapter.allMids()` is also public and read-only. It returns the
default perpetual DEX mids and first-DEX spot mids; Hyperliquid falls back to
the last trade price when a book is empty. This method is fixture-verified and
has not been live-read verified.

`userRateLimit()`, `userRole()`, `referral()`, `userFees()`, `portfolio()`,
`subAccounts()`, and `userVaultEquities()` are public `/info` reads for the
configured Hyperliquid address. They require configuring the adapter with a
wallet address (a private key is optional) but do not use a signature. These
paths are fixture-verified only.

`userFills(aggregateByTime)` and `userFillsByTime(from, to, aggregateByTime)`
are unsigned `POST /info` reads for the configured public address; no private
key or signature is used. The latter requires `from`, accepts optional `to`,
and uses inclusive millisecond boundaries. Both preserve provider execution,
position, fee, order, direction, and raw fields; they are fixture-verified only.

`basicOpenOrders()`, `orderStatus(reference)`, and `historicalOrders()` are
also address-bound, unsigned `POST /info` reads. The first uses Hyperliquid's
compact `openOrders` response and is distinct from common `openOrders()`, which
uses `frontendOpenOrders`. `reference` accepts a numeric `oid` or a
`0x`-prefixed 32-hex-character client order ID; `unknownOid` returns normal
`HyperliquidOrderStatusResponseUnknownOrder`, while future top-level statuses
retain their status and raw JSON. Historical and found detailed orders retain
trigger, time-in-force, reduce-only, client-ID, status, and raw JSON fields;
`historicalOrders()` returns up to the latest 2,000 orders. All three require a
valid configured wallet address and fail before network I/O when it is absent
or invalid; no API key, private key, or signature is used. Fixture-verified only.

## Browser credentials

In a browser, public calls can use direct HTTP and WebSocket connections when
the exchange permits them. Set `relayUrl` when a relay is required:

```dart
await Maxt.initialize(relayUrl: 'https://relay.example');
```

Browser credentials are disabled by default because JavaScript and WebAssembly
memory are not secret storage. Credentialed browser calls require both a relay
and explicit opt-in:

```dart
await Maxt.initialize(
  relayUrl: 'https://relay.example',
  allowInsecureBrowserCredentials: true,
);
```

Use restricted exchange keys without withdrawal permission. Keep credentials
on a trusted backend when they must not be exposed to the browser.

Deploy the relay behind an authenticated, rate-limited TLS ingress on the same
site as the application. The relay does not authenticate users, and its Origin
allowlist is not authentication. See the [relay deployment and security
requirements](../../relay/README.md).

## Streams

```dart
final stream = await client.subscribe(
  Subscription(markets: [market], feeds: [Feed.trades]),
);
try {
  await for (final item in stream) {
    switch (item) {
      case StreamEvent(:final event):
        print(event);
      case StreamError(:final error):
        print(error);
    }
  }
} finally {
  await stream.close();
}
```

`StreamError` does not terminate the stream. `close()` waits for native cleanup.

## Custom adapters

Extend `AdapterBase`, implement `exchange` and `features`, then override every
advertised operation. Wrap the instance with `Client(adapter)`. Default
methods return `UnsupportedError`.

For custom streams, return `MarketStream` or `AccountStream` over a Dart
`Stream<StreamItem<T>>`. Pass `onClose` when cleanup is required.

## Contracts

- `Decimal`: exact 96-bit coefficient, scale `0..=28`.
- `Timestamp`: signed 64-bit Unix epoch nanoseconds stored as `BigInt`.
- Errors: `InvalidRequestError`, `UnsupportedError`, `AdapterError`, `AuthenticationError`, `ExchangeError`, `TransportError`, `DecodeError`.
- Credentials: omit both fields for public access; provide both for private access.

See the [common data and pagination contracts](../../docs/common-api.md) and
[provider limits and data semantics](../../docs/providers.md).

## Documentation and examples

- [Runnable Binance public-ticker example](example/public_ticker.dart)
- Generate local API documentation with `dart doc`.
- [Repository getting started guide](../../docs/getting-started.md)
- [Provider reference](../../docs/providers.md)
- [Generated endpoint coverage reference](../common/generated/api.md)

## License

MIT
