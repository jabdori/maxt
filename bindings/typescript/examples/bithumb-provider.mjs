// Read public Bithumb market warnings, notices, and transfer fees.
import { BithumbAdapter, initialize } from "@jabdori/maxt/node";

await initialize();

const adapter = new BithumbAdapter();
const [warnings, notices, fees] = await Promise.all([
  adapter.marketWarnings(),
  adapter.notices(5),
  adapter.transferFees("BTC"),
]);
console.log(`${warnings.length} warning rows, ${notices.length} notices, ${fees.length} BTC fee rows`);
