import { BinanceAdapter, Client, Exchange, Market, type Ticker } from "../src/node.js";

const adapter = BinanceAdapter.spot();
const client = new Client(adapter);
const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const ticker: Promise<Ticker> = client.ticker(market);
const filters = client.adapter.spotSymbolFilters(market);
void ticker;
void filters;
