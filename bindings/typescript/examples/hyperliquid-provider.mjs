// Read Hyperliquid public data and optional address-scoped Info data.
import { Exchange, HyperliquidAdapter, Market, initialize } from "@jabdori/maxt/node";

await initialize();

const market = Market.perpetual(Exchange.Hyperliquid, "BTC", "USDC");
const adapter = new HyperliquidAdapter();
const [mids, book, trades] = await Promise.all([
  adapter.allMids(),
  adapter.l2Book(market),
  adapter.recentTrades(market),
]);
console.log(`${mids.length} mid prices; ${book.bids.length + book.asks.length} levels; ${trades.length} trades`);

const address = process.env.HYPERLIQUID_ADDRESS;
if (address === undefined) {
  console.log("Set HYPERLIQUID_ADDRESS for address-scoped Info reads; a private key is not needed.");
} else {
  const orders = await new HyperliquidAdapter({ address }).basicOpenOrders();
  console.log(`${orders.length} address-scoped open orders`);
}
