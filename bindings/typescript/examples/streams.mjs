// Print three public Binance trade-stream items, then close the iterator.
import {
  BinanceAdapter,
  Client,
  Exchange,
  Feed,
  Market,
  Subscription,
  initialize,
} from "@jabdori/maxt/node";

await initialize();

const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const stream = await new Client(BinanceAdapter.spot()).subscribe(
  new Subscription([market], [Feed.Trades]),
);

let remaining = 3;
for await (const item of stream) {
  console.log(item);
  if (--remaining === 0) break;
}
