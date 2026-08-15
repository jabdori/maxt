// Browser entry point: public reads connect directly. Signed calls need a relay.
import {
  BinanceAdapter,
  Client,
  Exchange,
  Market,
  initialize,
} from "@jabdori/maxt/browser";

// Set this to a trusted relay URL only for signed browser calls.
const relayUrl = null;
await initialize(
  relayUrl === null
    ? {}
    : { relayUrl, allowInsecureBrowserCredentials: true },
);

const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const ticker = await new Client(BinanceAdapter.spot()).ticker(market);
console.log(`${ticker.market}: ${ticker.lastPrice}`);
console.log("For signed browser calls, set a trusted relay URL and use the deployment controls in relay/README.md.");
