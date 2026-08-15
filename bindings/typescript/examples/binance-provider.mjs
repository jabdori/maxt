// Use public Binance provider-specific Spot and USD-M reads.
import { BinanceAdapter, Exchange, Market, initialize } from "@jabdori/maxt/node";

await initialize();

const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const spot = BinanceAdapter.spot();
const [average, filters, exchange] = await Promise.all([
  spot.spotAveragePrice(market),
  spot.spotSymbolFilters(market),
  spot.spotExchangeInfo(),
]);
console.log(`${average.minutes}-minute average=${average.price}; tick=${filters.tickSize}`);
console.log(`Spot symbols: ${exchange.symbols.length}`);

const futures = BinanceAdapter.usdMFutures();
const metadata = await futures.usdMExchangeInfo();
console.log(`USD-M symbols: ${metadata.symbols.length}`);
