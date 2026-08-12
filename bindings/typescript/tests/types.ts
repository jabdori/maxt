import {
  BinanceAdapter,
  BinanceAggregateTradesRequest,
  type BinanceAggregateTrade,
  type BinanceMarkPrice,
  type BinanceOpenInterest,
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
  OrderRequest,
  OrderLookupRequest,
  TransferDestination,
  TransferError,
  TransferErrorKind,
  TransferHistoryRequest,
  TransferLookupRequest,
  WithdrawRequest,
  UpbitAdapter,
  UpbitBatchCancelRequest,
  UpbitBatchCancelScope,
  UpbitCancelAndNewOrder,
  UpbitCancelAndNewOrderRequest,
  UpbitOrderDirection,
  UpbitOrderReference,
  UpbitOrderVolume,
  UpbitSmpType,
  BithumbAdapter,
  BithumbBatchOrdersRequest,
  type BithumbBatchOrdersResult,
  BithumbOrderDirection,
  BithumbPendingOrderState,
  BithumbPendingOrdersRequest,
  BithumbTwapOrder,
  BithumbTwapOrderRequest,
  BithumbTwapOrdersRequest,
  Cursor,
  HyperliquidAdapter,
  type HyperliquidMidPrice,
  type AssetNetwork,
  type Deposit,
  type Page,
  type Ticker,
  type Withdrawal,
  type WithdrawalQuote,
  type UpbitOrderBookInstrument,
  type UpbitDepositInfo,
  type UpbitYearCandle,
  type BithumbApiKey,
  type BithumbAssetFee,
  type BithumbNotice,
  type Order,
  Side,
  Size,
  TimeInForce,
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
const usdM = BinanceAdapter.usdMFutures();
const usdMMarket = Market.perpetual(Exchange.Binance, "BTC", "USDT");
const markPrice: Promise<BinanceMarkPrice> = usdM.markPrice(usdMMarket);
const markPrices: Promise<readonly BinanceMarkPrice[]> = usdM.markPrices();
const openInterest: Promise<BinanceOpenInterest> = usdM.openInterest(usdMMarket);
const aggregateTrades: Promise<readonly BinanceAggregateTrade[]> = usdM.aggregateTrades(
  new BinanceAggregateTradesRequest(usdMMarket, 100n),
);
const hyperliquid = new HyperliquidAdapter();
const allMids: Promise<readonly HyperliquidMidPrice[]> = hyperliquid.allMids();
const upbit = new UpbitAdapter();
const upbitMarket = Market.spot(Exchange.Upbit, "BTC", "KRW");
const quoteTickers: Promise<readonly Ticker[]> = upbit.tickersByQuote(["KRW"]);
const aggregatedBooks = upbit.orderBooksAtLevel(
  [upbitMarket],
  Decimal.parse("100000"),
);
const yearCandles: Promise<readonly UpbitYearCandle[]> = upbit.yearCandles(upbitMarket);
const instruments: Promise<readonly UpbitOrderBookInstrument[]> =
  upbit.orderbookInstruments([upbitMarket]);
const testOrder: Promise<Order> = upbit.testOrder(
  OrderRequest.limit(
    upbitMarket,
    Side.Buy,
    Size.base(Decimal.parse("0.01")),
    Decimal.parse("100000000"),
  ),
);
const depositInfo: Promise<UpbitDepositInfo> = upbit.depositInfo("BTC", Network.Bitcoin);
const batchCancellation = upbit.batchCancelOpenOrders(
  new UpbitBatchCancelRequest(
    UpbitBatchCancelScope.all(),
    null,
    null,
    20,
    UpbitOrderDirection.Ascending,
  ),
);
const cancelAndNew = upbit.cancelAndNewOrder(
  new UpbitCancelAndNewOrderRequest(
    UpbitOrderReference.identifier("old-client-id"),
    UpbitCancelAndNewOrder.limit(
      UpbitOrderVolume.amount(Decimal.parse("0.01")),
      Decimal.parse("100000000"),
      TimeInForce.ImmediateOrCancel,
    ),
    "new-client-id",
    UpbitSmpType.Reduce,
  ),
);
const bithumb = new BithumbAdapter();
const notices: Promise<readonly BithumbNotice[]> = bithumb.notices();
const transferFees: Promise<readonly BithumbAssetFee[]> = bithumb.transferFees("BTC");
const apiKeys: Promise<readonly BithumbApiKey[]> = bithumb.apiKeys();
const pendingOrders: Promise<Page<Order>> = bithumb.pendingOrders(
  new BithumbPendingOrdersRequest(
    Market.spot(Exchange.Bithumb, "BTC", "KRW"),
    BithumbPendingOrderState.Watch,
    25,
    BithumbOrderDirection.Ascending,
    new Cursor("page+/=="),
  ),
);
const batchOrders: Promise<BithumbBatchOrdersResult> = bithumb.batchOrders(
  new BithumbBatchOrdersRequest([]),
);
const twapOrders: Promise<Page<BithumbTwapOrder>> = bithumb.twapOrders(
  new BithumbTwapOrdersRequest(),
);
const twapOrder: Promise<string> = bithumb.createTwapOrder(
  new BithumbTwapOrderRequest(
    Market.spot(Exchange.Bithumb, "BTC", "KRW"),
    Side.Buy,
    null,
    Decimal.one,
    300,
    30,
  ),
);
const twapCancellation: Promise<string> = bithumb.cancelTwapOrder("twap-1");
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
void markPrice;
void markPrices;
void openInterest;
void aggregateTrades;
void allMids;
void quoteTickers;
void aggregatedBooks;
void yearCandles;
void instruments;
void testOrder;
void depositInfo;
void batchCancellation;
void cancelAndNew;
void notices;
void transferFees;
void apiKeys;
void pendingOrders;
void batchOrders;
void twapOrders;
void twapOrder;
void twapCancellation;
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
