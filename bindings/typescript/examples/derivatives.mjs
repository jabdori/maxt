// Read public Binance USD-M perpetual market data and funding history.
import {
  BinanceAdapter,
  Client,
  Exchange,
  HistoryRequest,
  Market,
  initialize,
} from "@jabdori/maxt/node";

await initialize();

const market = Market.perpetual(Exchange.Binance, "BTC", "USDT");
const adapter = BinanceAdapter.usdMFutures();
const [mark, interest] = await Promise.all([
  adapter.markPrice(market),
  adapter.openInterest(market),
]);
console.log(`mark=${mark.markPrice}; open interest=${interest.openInterest}`);

const funding = await new Client(adapter).fundingRates(new HistoryRequest(market, null, null, null, 5));
console.log(`${funding.items.length} funding rows; next=${funding.next}`);
