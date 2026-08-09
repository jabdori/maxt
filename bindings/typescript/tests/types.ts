import {
  BinanceAdapter,
  ChainDestination,
  Client,
  Decimal,
  DepositAddressRequest,
  Exchange,
  Market,
  Network,
  TransferDestination,
  TransferError,
  TransferErrorKind,
  TransferHistoryRequest,
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
const address = client.depositAddress(new DepositAddressRequest("BTC", Network.Bitcoin));
const quote: Promise<WithdrawalQuote> = client.prepareWithdrawal(withdrawRequest);
const withdrawal: Promise<Withdrawal> = client.withdraw(withdrawRequest);
const deposits: Promise<Page<Deposit>> = client.deposits(new TransferHistoryRequest());
const withdrawals: Promise<Page<Withdrawal>> = client.withdrawals(new TransferHistoryRequest());
const transferError = new TransferError(TransferErrorKind.NetworkMismatch, "chains differ");
void ticker;
void filters;
void networks;
void address;
void quote;
void withdrawal;
void deposits;
void withdrawals;
void transferError;
void (null as unknown as _BrowserContainsExactlyNodeExports);
