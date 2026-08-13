// Binance Spot public ticker example. No credentials and no orders.
import {
  BinanceAdapter,
  Client,
  Exchange,
  Market,
  initialize,
} from "@jabdori/maxt/node";

await initialize();

const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const client = new Client(BinanceAdapter.spot());

const ticker = await client.ticker(market);
const average = await client.adapter.spotAveragePrice(market);

console.log(`${market}: last=${ticker.lastPrice}`);
console.log(`Binance ${average.minutes}-minute average=${average.price}`);
