import {
  BinanceAdapter,
  ChainDestination,
  Client,
  CancelOrdersRequest,
  Decimal,
  type DepositAddressEntry,
  DepositAddressRequest,
  Exchange,
  Market,
  Network,
  OrderIdKind,
  OrderLookupRequest,
  TransferDestination,
  TransferError,
  TransferErrorKind,
  TransferHistoryRequest,
  TransferLookupRequest,
  WithdrawRequest,
  type AssetNetwork,
  type Deposit,
  type Page,
  type Ticker,
  type Withdrawal,
  type WithdrawalQuote,
} from "../src/node.js";

type NodeExports = typeof import("../src/node.js");
type BrowserExports = typeof import("../src/browser.js");
type Assert<T extends true> = T;
type _BrowserContainsExactlyNodeExports = Assert<
  BrowserExports extends NodeExports
    ? NodeExports extends BrowserExports ? true : false
    : false
>;

const adapter = BinanceAdapter.spot();
const client = new Client(adapter);
const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const ticker: Promise<Ticker> = client.ticker(market);
const filters = client.adapter.spotSymbolFilters(market);
const destination = TransferDestination.chain(
  new ChainDestination("BTC", Network.Bitcoin, "bc1qdestination"),
);
const withdrawRequest = new WithdrawRequest("BTC", Network.Bitcoin, Decimal.one, destination);
const networks: Promise<readonly AssetNetwork[]> = client.assetNetworks("BTC");
const addresses: Promise<readonly DepositAddressEntry[]> = client.depositAddresses();
const address = client.depositAddress(new DepositAddressRequest("BTC", Network.Bitcoin));
const quote: Promise<WithdrawalQuote> = client.prepareWithdrawal(withdrawRequest);
const withdrawal: Promise<Withdrawal> = client.withdraw(withdrawRequest);
const deposit = client.deposit(new TransferLookupRequest("BTC", "deposit-1"));
const withdrawalLookup = client.withdrawal(new TransferLookupRequest("BTC", "withdrawal-1"));
const cancellation: Promise<void> = client.cancelWithdrawal("withdrawal-1");
const deposits: Promise<Page<Deposit>> = client.deposits(new TransferHistoryRequest());
const withdrawals: Promise<Page<Withdrawal>> = client.withdrawals(new TransferHistoryRequest());
const orders = client.ordersByIds(new OrderLookupRequest(OrderIdKind.Exchange, ["order-1"], market));
const cancellations = client.cancelOrders(
  new CancelOrdersRequest(OrderIdKind.Exchange, ["order-1"]),
);
const transferError = new TransferError(TransferErrorKind.NetworkMismatch, "chains differ");
void ticker;
void filters;
void networks;
void addresses;
void address;
void quote;
void withdrawal;
void deposit;
void withdrawalLookup;
void cancellation;
void deposits;
void withdrawals;
void orders;
void cancellations;
void transferError;
void (null as unknown as _BrowserContainsExactlyNodeExports);
