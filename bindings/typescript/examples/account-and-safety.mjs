// Build requests without a financial write, then read an Upbit account if keys exist.
import {
  Client,
  Decimal,
  Exchange,
  Market,
  Network,
  OrderRequest,
  Side,
  Size,
  TransferHistoryRequest,
  UpbitAdapter,
  initialize,
} from "@jabdori/maxt/node";

await initialize();

const market = Market.spot(Exchange.Upbit, "BTC", "KRW");
const draft = OrderRequest.limit(
  market,
  Side.Buy,
  Size.base(Decimal.parse("0.0001")),
  Decimal.parse("100000"),
  { clientId: "docs-example-only" },
);
const history = new TransferHistoryRequest("BTC", Network.Bitcoin, null, 20);
console.log("Order draft only; it was not sent:", draft);
console.log("Transfer-history request:", history);

const { UPBIT_ACCESS_KEY: accessKey, UPBIT_SECRET_KEY: secretKey } = process.env;
if (accessKey === undefined || secretKey === undefined) {
  console.log("Set UPBIT_ACCESS_KEY and UPBIT_SECRET_KEY to run the read-only account section.");
} else {
  const client = new Client(new UpbitAdapter({ accessKey, secretKey }));
  const [balances, orders] = await Promise.all([client.balances(), client.openOrders()]);
  console.log(`${balances.length} balances and ${orders.length} open orders`);
}
