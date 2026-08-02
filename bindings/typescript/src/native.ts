import { AdapterError, InvalidRequestError } from "./errors.js";
import type { ErrorWire } from "./errors.js";
import type {
  BalanceWire, CandleRequestWire, CandleWire, DecimalWire, FeedWire, FundingPaymentWire,
  FundingRateWire, HistoryRequestWire, LevelWire, MarginRequestWire, MarginSummaryWire,
  MarketInfoWire, MarketKindWire, MarketWire, OrderBookWire, OrderRequestWire, OrderWire,
  PositionWire, SizeWire, StreamConfigWire, SubscriptionWire, TickerWire, TimestampWire,
  TradeWire,
} from "./generated/contract.js";
export type {
  BalanceWire, CandleRequestWire, CandleWire, DecimalWire, FeedWire, FundingPaymentWire,
  FundingRateWire, HistoryRequestWire, LevelWire, MarginRequestWire, MarginSummaryWire,
  MarketInfoWire, MarketKindWire, MarketWire, OrderBookWire, OrderRequestWire, OrderWire,
  PositionWire, SizeWire, StreamConfigWire, SubscriptionWire, TickerWire, TimestampWire,
  TradeWire,
} from "./generated/contract.js";
import type {
  AccountStreamItemWire, AdapterCallWire, AdapterReplyWire, BinanceListenKeyWire,
  BinanceOptionsWire, BinanceSpotOrderDetailWire, BinanceSymbolFiltersWire,
  BithumbMarketAlertWire, BithumbOptionsWire, HyperliquidAssetContextWire,
  HyperliquidLedgerEntryWire, HyperliquidOptionsWire, MarketStreamItemWire, PageWire,
  UpbitMarketEventWire, UpbitOptionsWire,
} from "./generated/contract.js";
export type {
  AccountStreamItemWire, AdapterCallWire, AdapterReplyWire, BinanceListenKeyWire,
  BinanceOptionsWire, BinanceSpotOrderDetailWire, BinanceSymbolFiltersWire,
  BithumbMarketAlertWire, BithumbOptionsWire, HyperliquidAssetContextWire,
  HyperliquidLedgerEntryWire, HyperliquidOptionsWire, MarketStreamItemWire, PageWire,
  UpbitMarketEventWire, UpbitOptionsWire,
} from "./generated/contract.js";

export interface InitializeOptions {
  readonly wasmUrl?: string | URL;
  readonly allowInsecureBrowserCredentials?: boolean;
}

export interface NormalizedInitializeOptions {
  readonly wasmUrl: string | null;
  readonly allowInsecureBrowserCredentials: boolean;
}

export type NativeOutcome<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: ErrorWire };

export interface NativeStreamHandle<I> {
  next(): Promise<NativeOutcome<I | null>>;
  close(): Promise<NativeOutcome<null>>;
}

export interface NativeClientHandle {
  exchange(): string;
  supports(feature: string): boolean;
  markets(kind: MarketKindWire): Promise<NativeOutcome<readonly MarketInfoWire[]>>;
  trades(market: MarketWire, limit: number | null): Promise<NativeOutcome<readonly TradeWire[]>>;
  orderBook(market: MarketWire, depth: number | null): Promise<NativeOutcome<OrderBookWire>>;
  ticker(market: MarketWire): Promise<NativeOutcome<TickerWire>>;
  candles(request: CandleRequestWire): Promise<NativeOutcome<readonly CandleWire[]>>;
  subscribe(subscription: SubscriptionWire): Promise<NativeOutcome<NativeStreamHandle<MarketStreamItemWire>>>;
  subscribeWith(
    subscription: SubscriptionWire,
    config: StreamConfigWire,
  ): Promise<NativeOutcome<NativeStreamHandle<MarketStreamItemWire>>>;
  balances(): Promise<NativeOutcome<readonly BalanceWire[]>>;
  openOrders(): Promise<NativeOutcome<readonly OrderWire[]>>;
  openOrdersOn(market: MarketWire): Promise<NativeOutcome<readonly OrderWire[]>>;
  subscribeAccount(): Promise<NativeOutcome<NativeStreamHandle<AccountStreamItemWire>>>;
  subscribeAccountWith(
    config: StreamConfigWire,
  ): Promise<NativeOutcome<NativeStreamHandle<AccountStreamItemWire>>>;
  placeOrder(request: OrderRequestWire): Promise<NativeOutcome<OrderWire>>;
  cancelOrder(market: MarketWire, orderId: string): Promise<NativeOutcome<OrderWire>>;
  positions(): Promise<NativeOutcome<readonly PositionWire[]>>;
  positionsOn(market: MarketWire): Promise<NativeOutcome<readonly PositionWire[]>>;
  marginSummary(): Promise<NativeOutcome<MarginSummaryWire>>;
  fundingRates(request: HistoryRequestWire): Promise<NativeOutcome<PageWire<FundingRateWire>>>;
  fundingPayments(request: HistoryRequestWire): Promise<NativeOutcome<PageWire<FundingPaymentWire>>>;
  setMargin(request: MarginRequestWire): Promise<NativeOutcome<null>>;
}

export interface ForeignAdapterCallbacks {
  dispatch(call: AdapterCallWire): Promise<NativeOutcome<AdapterReplyWire>>;
  streamNext(
    id: string,
  ): Promise<NativeOutcome<MarketStreamItemWire | AccountStreamItemWire | null>>;
  streamClose(id: string): Promise<NativeOutcome<null>>;
}

export interface NativeUpbitHandle {
  client(): NativeClientHandle;
  region(): string;
  orderBooks(
    markets: readonly MarketWire[],
    depth: number | null,
  ): Promise<NativeOutcome<readonly OrderBookWire[]>>;
  tickers(markets: readonly MarketWire[]): Promise<NativeOutcome<readonly TickerWire[]>>;
  marketEvents(): Promise<NativeOutcome<readonly (readonly [MarketWire, UpbitMarketEventWire])[]>>;
}

export interface NativeBithumbHandle {
  client(): NativeClientHandle;
  marketWarnings(): Promise<NativeOutcome<readonly (readonly [MarketWire, string])[]>>;
  marketAlerts(): Promise<NativeOutcome<readonly (readonly [MarketWire, BithumbMarketAlertWire])[]>>;
}

export interface NativeBinanceHandle {
  client(): NativeClientHandle;
  venue(): string;
  spotSymbolFilters(market: MarketWire): Promise<NativeOutcome<BinanceSymbolFiltersWire>>;
  spotOrder(market: MarketWire, orderId: string): Promise<NativeOutcome<BinanceSpotOrderDetailWire>>;
  usdMCreateListenKey(): Promise<NativeOutcome<BinanceListenKeyWire>>;
  usdMKeepaliveListenKey(id: string): Promise<NativeOutcome<null>>;
  usdMCloseListenKey(id: string): Promise<NativeOutcome<null>>;
}

export interface NativeHyperliquidHandle {
  client(): NativeClientHandle;
  isTestnet(): boolean;
  nonFundingLedger(
    from: TimestampWire | null,
    to: TimestampWire | null,
    cursor: string | null,
    limit: number | null,
  ): Promise<NativeOutcome<PageWire<HyperliquidLedgerEntryWire>>>;
  assetContext(market: MarketWire): Promise<NativeOutcome<HyperliquidAssetContextWire>>;
}

export interface NativeBackend {
  initialize(options: NormalizedInitializeOptions): Promise<void>;
  customClient(
    exchange: string,
    features: readonly string[],
    callbacks: ForeignAdapterCallbacks,
  ): NativeClientHandle;
  upbit(options: UpbitOptionsWire): NativeUpbitHandle;
  bithumb(options: BithumbOptionsWire): NativeBithumbHandle;
  binance(options: BinanceOptionsWire): NativeBinanceHandle;
  hyperliquid(options: HyperliquidOptionsWire): NativeHyperliquidHandle;
}

let installedBackend: NativeBackend | undefined;
let initializedOptions: NormalizedInitializeOptions | undefined;
let initialization: Promise<void> | undefined;

export function installBackend(backend: NativeBackend): void {
  if (installedBackend === backend) return;
  if (installedBackend !== undefined) throw new AdapterError("maxt backend is already installed");
  installedBackend = backend;
}

export function initialize(options: InitializeOptions = {}): Promise<void> {
  let normalized: NormalizedInitializeOptions;
  try {
    normalized = normalizeInitializeOptions(options);
  } catch (error) {
    return Promise.reject(error);
  }
  if (initialization !== undefined) {
    return sameInitializeOptions(initializedOptions!, normalized)
      ? initialization
      : Promise.reject(new InvalidRequestError(
        "initialize",
        "maxt is already initialized with different options",
      ));
  }

  initializedOptions = normalized;
  initialization = Promise.resolve().then(() => {
    if (installedBackend === undefined) throw new AdapterError("maxt backend is not installed");
    return installedBackend.initialize(normalized);
  });
  return initialization;
}

export function ensureInitialized(): Promise<void> {
  return initialization ?? initialize();
}

function normalizeInitializeOptions(options: InitializeOptions): NormalizedInitializeOptions {
  const allowInsecureBrowserCredentials = options.allowInsecureBrowserCredentials;
  if (allowInsecureBrowserCredentials !== undefined
    && typeof allowInsecureBrowserCredentials !== "boolean") {
    throw new InvalidRequestError(
      "allowInsecureBrowserCredentials",
      "must be a boolean",
    );
  }
  return {
    wasmUrl: normalizeWasmUrl(options.wasmUrl),
    allowInsecureBrowserCredentials: allowInsecureBrowserCredentials ?? false,
  };
}

function normalizeWasmUrl(wasmUrl: string | URL | undefined): string | null {
  if (wasmUrl === undefined) return null;
  try {
    return new URL(String(wasmUrl), import.meta.url).href;
  } catch {
    throw new InvalidRequestError("wasmUrl", "must be a valid URL");
  }
}

function sameInitializeOptions(
  left: NormalizedInitializeOptions,
  right: NormalizedInitializeOptions,
): boolean {
  return left.wasmUrl === right.wasmUrl
    && left.allowInsecureBrowserCredentials === right.allowInsecureBrowserCredentials;
}
