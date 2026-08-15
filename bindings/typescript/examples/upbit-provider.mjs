// Read Upbit Korea provider-specific quotation data.
import { Exchange, Market, UpbitAdapter, initialize } from "@jabdori/maxt/node";

await initialize();

const adapter = new UpbitAdapter();
const market = Market.spot(Exchange.Upbit, "BTC", "KRW");
const [tickers, instruments] = await Promise.all([
  adapter.tickers([market]),
  adapter.orderbookInstruments([market]),
]);
console.log(`region=${adapter.region.id}; ticker rows=${tickers.length}`);
console.log(`order-book instrument rows=${instruments.length}`);
