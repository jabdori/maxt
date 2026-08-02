import { BinanceAdapter, Client, Exchange, Market, type Ticker } from "../src/node.js";

type NodeExports = typeof import("../src/node.js");
type BrowserExports = typeof import("../src/browser.js");
type Assert<T extends true> = T;
type _BrowserContainsExactlyNodeExports = Assert<
  BrowserExports extends NodeExports
    ? NodeExports extends BrowserExports ? true : false
    : false
>;

const adapter = BinanceAdapter.spot();
const client = new Client(adapter);
const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const ticker: Promise<Ticker> = client.ticker(market);
const filters = client.adapter.spotSymbolFilters(market);
void ticker;
void filters;
void (null as unknown as _BrowserContainsExactlyNodeExports);
