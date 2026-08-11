# maxt for TypeScript

[English](README.md) | [한국어](README.ko.md)

One TypeScript API backed by native Node.js code or Browser WebAssembly
(WASM). Both backends use the same generated models, errors, adapters, and
stream contract. Generation checks keep that contract aligned with the
compiled backend API.

## Support

- [x] Node.js 22 or newer
- [x] Browser WebAssembly

The Node.js package is ESM-only. Prebuilt native modules cover glibc Linux
(x64 and ARM64), macOS (x64 and ARM64), and Windows (x64). Alpine and other
musl Linux distributions are not supported in 0.1.0. Browser tests cover
Chromium, Firefox, and WebKit.

## Install

```sh
npm install @jabdori/maxt
```

## Supported exchanges

- Upbit Spot: Korea, Singapore, Indonesia, and Thailand
- Bithumb Spot
- Binance Spot and USD-M perpetual futures
- Hyperliquid Spot and perpetual futures on mainnet and testnet

Binance testnet constructors are not exposed. Hyperliquid HIP-3 perpetual DEXs
and outcome assets are not exposed.

## Common API

`Client` provides the same method names in Node.js and Browser WASM:

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

Public calls need no credentials. Private calls require both credential fields.
Browser private calls additionally require a relay and
`allowInsecureBrowserCredentials: true`. Use `client.supports(feature)` before
optional operations when the adapter or credential state is dynamic.

## Exchange-specific API

Exchange-specific methods remain available through `client.adapter` on both
backends.

| Adapter | Construction | Additional methods |
| --- | --- | --- |
| `UpbitAdapter` | `new UpbitAdapter()` or `UpbitAdapter.withRegion(...)` | `orderBooks()`, `orderBooksAtLevel()`, `tickers()`, `tickersByQuote()`, `yearCandles()`, `orderbookInstruments()`, `marketEvents()`; authenticated: `testOrder()`, `depositInfo()`, `batchCancelOpenOrders()` |
| `BithumbAdapter` | `new BithumbAdapter()` | `marketWarnings()`, `marketAlerts()`, `notices()`, `transferFees()`; authenticated: `apiKeys()`, `pendingOrders()`, `twapOrders()`, `createTwapOrder()`, `cancelTwapOrder()` |
| `BinanceAdapter` | `BinanceAdapter.spot()` | `spotSymbolFilters()`; authenticated: `spotOrder()` |
| `BinanceAdapter` | `BinanceAdapter.usdMFutures()` | Public: `markPrice()`, `markPrices()`, `openInterest()`; authenticated: `usdMCreateListenKey()`, `usdMKeepaliveListenKey()`, `usdMCloseListenKey()` |
| `HyperliquidAdapter` | `new HyperliquidAdapter()` or `HyperliquidAdapter.testnet()` | Public: `allMids()`; `assetContext()`, `nonFundingLedger()` |

`UpbitAdapter.testOrder()` validates an order without creating it. The returned
`Order` is a dry-run result: do not query or cancel its `id`, and do not treat
its status as a live order.

`UpbitAdapter.depositInfo(asset, network)` returns the provider's deposit
availability, minimum amount, confirmation, and precision metadata. Upbit may
delay this information by several minutes; it is not a real-time service-status signal.

`UpbitAdapter.batchCancelOpenOrders(request)` is a financial write.
`UpbitBatchCancelScope.all()` explicitly selects every eligible market; Upbit
still applies the request count (default 20, maximum 300 `wait` orders), and
the result preserves partial failures.

`BithumbAdapter.twapOrders(request)` is an authenticated, read-only history
query for Bithumb's KRW markets. `createTwapOrder()` and
`cancelTwapOrder()` are financial writes; do not call them in a read-only
verification.

```ts
const adapter = new BithumbAdapter({ accessKey, secretKey });
const market = Market.spot(Exchange.Bithumb, "BTC", "KRW");
const page = await adapter.twapOrders(
  new BithumbTwapOrdersRequest(market, [], null, null, 20, null),
);
```

The Bithumb TWAP API accepts `progress`, `done`, or `cancel` states and uses a
page size from 1 through 100. Creation uses a 300–43,200 second duration and a
15/20/30/60/120 second interval; buys require `price`, sells require `volume`.

`BinanceAdapter.usdMFutures()` exposes `markPrice()`, `markPrices()`, and
`openInterest()` as public, read-only USD-M perpetual market-data calls. These
methods are fixture-verified; they have not been live-read verified.
`HyperliquidAdapter.allMids()` is also public and read-only. It returns the
default perpetual DEX mids and first-DEX spot mids; Hyperliquid falls back to
the last trade price when a book is empty. This method is fixture-verified and
has not been live-read verified.

## Node.js

Use the Node.js entry point. `initialize()` is idempotent with the same options.

```ts
import {
  BinanceAdapter,
  Client,
  Exchange,
  Market,
  initialize,
} from "@jabdori/maxt/node";

await initialize();

const client = new Client(BinanceAdapter.spot());
const market = Market.spot(Exchange.Binance, "BTC", "USDT");

const ticker = await client.ticker(market);
const filters = await client.adapter.spotSymbolFilters(market);

console.log(ticker.lastPrice.toString());
console.log(filters.tickSize?.toString());
```

`ticker()` is common. `spotSymbolFilters()` is Binance Spot-specific and is
available through `client.adapter`.

## Browser WebAssembly

Use the browser entry point and await `initialize()` before constructing an
adapter. The packaged WASM file is used by default; `wasmUrl` overrides it.

```ts
import {
  BinanceAdapter,
  Client,
  Exchange,
  Market,
  initialize,
} from "@jabdori/maxt/browser";

await initialize();

const client = new Client(BinanceAdapter.spot());
const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const ticker = await client.ticker(market);
```

Without `relayUrl`, public HTTP and WebSocket calls connect directly from the
browser and are subject to browser CORS and network policy. No relay is needed
for public operations.

Credentialed adapters require an explicit opt-in and a relay origin:

```ts
await initialize({
  relayUrl: "https://relay.example",
  allowInsecureBrowserCredentials: true,
});

const adapter = BinanceAdapter.spot({ apiKey, secretKey });
```

`relayUrl` must be an `http` or `https` origin without credentials, path,
query, or fragment. Once configured, Browser WASM sends HTTP requests through
the relay. WebSocket connections use it when the exchange requires handshake
headers; public WebSocket connections remain direct.

Deploy the relay behind an authenticated, rate-limited TLS ingress on the same
site as the application. The relay itself authenticates no users; its Origin
allowlist is not an authentication mechanism.

Warning: `allowInsecureBrowserCredentials` does not make browser credentials
safe. Raw credentials exist in JavaScript/WASM memory and can be exposed by
cross-site scripting, extensions, source maps, logs, or a compromised runtime.
The relay receives authentication headers and signed payloads in memory. Use a
trusted, TLS-protected, authenticated, rate-limited relay and narrowly scoped
credentials.

## Streams

```ts
import { Feed, StreamError, Subscription } from "@jabdori/maxt/node";

const stream = await client.subscribe(new Subscription([market], [Feed.Trades]));
try {
  for await (const item of stream) {
    if (item instanceof StreamError) console.error(item.error);
    else console.log(item.event);
  }
} finally {
  await stream.close();
}
```

`StreamError` does not terminate iteration. `close()` waits for backend cleanup.

## Custom adapters

Extend `Adapter`, set `exchange` and `features`, then override every advertised
operation. Wrap the instance with `new Client(adapter)`. Default methods reject
with `UnsupportedError`.

For custom streams, return `MarketStream` or `AccountStream` over an
`AsyncIterable` of `StreamEvent` and `StreamError`. Pass a close callback when
cleanup is required. Browser custom adapters use the same generated bridge as
Node.js and require browser initialization first.

See the [relay](../../relay/README.md),
[common data and pagination contracts](../../docs/common-api.md), and
[provider limits and data semantics](../../docs/providers.md).

## License

MIT
