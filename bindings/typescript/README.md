# maxt for TypeScript

[English](README.md) | [한국어](README.ko.md)

One TypeScript API backed by native Node.js code or Browser WebAssembly
(WASM). Both backends use the same generated models, errors, adapters, and
stream contract. Generation checks keep that contract aligned with the
compiled backend API.

## Support

- [x] Node.js 22 or newer
- [x] Browser WebAssembly

## Install

```sh
npm install @jabdori/maxt
```

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

See the [relay](../../relay/README.md), [common API](../../docs/common-api.md),
and [provider support](../../docs/providers.md).

## License

MIT
