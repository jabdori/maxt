// Read public Binance Spot candles and prepare a paged history request.
import {
  BinanceAdapter,
  CandleRequest,
  Client,
  Exchange,
  HistoryRequest,
  Interval,
  Market,
  initialize,
} from "@jabdori/maxt/node";

await initialize();

const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const client = new Client(BinanceAdapter.spot());
const candles = await client.candles(new CandleRequest(market, Interval.Min1, null, null, 5));
for (const candle of candles) {
  console.log(`${candle.openTime}: close=${candle.close} volume=${candle.volume}`);
}

// Reuse a previous page cursor only when the prior response supplied one.
const history = new HistoryRequest(market, null, null, null, 100);
console.log("Private history request prepared:", history);
