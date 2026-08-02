import { AdapterError, InvalidRequestError } from "./errors.js";
import type { ErrorWire } from "./errors.js";

export interface InitializeOptions {
  readonly wasmUrl?: string | URL;
  readonly allowInsecureBrowserCredentials?: boolean;
}

export interface NormalizedInitializeOptions {
  readonly wasmUrl: string | null;
  readonly allowInsecureBrowserCredentials: boolean;
}

export type DecimalWire = string;
export type TimestampWire = string;
export type MarketKindWire = "spot" | "perpetual";

export interface MarketWire {
  readonly exchange: string;
  readonly kind: MarketKindWire;
  readonly base: string;
  readonly quote: string;
}

export interface MarketInfoWire {
  readonly market: MarketWire;
  readonly native_symbol: string;
  readonly status: string;
  readonly korean_name: string | null;
  readonly english_name: string | null;
}

export interface TradeWire {
  readonly market: MarketWire;
  readonly timestamp: TimestampWire;
  readonly price: DecimalWire;
  readonly quantity: DecimalWire;
  readonly taker_side: string;
  readonly id: string | null;
}

export interface LevelWire {
  readonly price: DecimalWire;
  readonly quantity: DecimalWire;
}

export interface OrderBookWire {
  readonly market: MarketWire;
  readonly timestamp: TimestampWire;
  readonly bids: readonly LevelWire[];
  readonly asks: readonly LevelWire[];
}

export interface TickerWire {
  readonly market: MarketWire;
  readonly timestamp: TimestampWire;
  readonly last_trade_time: TimestampWire | null;
  readonly last_price: DecimalWire;
  readonly change: DecimalWire | null;
  readonly change_rate: DecimalWire | null;
  readonly high: DecimalWire | null;
  readonly low: DecimalWire | null;
  readonly volume: DecimalWire | null;
  readonly quote_volume: DecimalWire | null;
}

export interface CandleWire {
  readonly market: MarketWire;
  readonly interval: string;
  readonly open_time: TimestampWire;
  readonly open: DecimalWire;
  readonly high: DecimalWire;
  readonly low: DecimalWire;
  readonly close: DecimalWire;
  readonly volume: DecimalWire;
  readonly quote_volume: DecimalWire | null;
  readonly closed: boolean;
}

export interface BalanceWire {
  readonly asset: string;
  readonly available: DecimalWire;
  readonly locked: DecimalWire;
}

export interface OrderWire {
  readonly id: string;
  readonly market: MarketWire;
  readonly side: string;
  readonly status: string;
  readonly filled_quantity: DecimalWire;
  readonly remaining_quantity: DecimalWire;
  readonly price: DecimalWire | null;
  readonly created_at: TimestampWire | null;
}

export interface PositionWire {
  readonly market: MarketWire;
  readonly side: string | null;
  readonly quantity: DecimalWire;
  readonly entry_price: DecimalWire | null;
  readonly mark_price: DecimalWire | null;
  readonly notional: DecimalWire | null;
  readonly unrealized_pnl: DecimalWire | null;
  readonly leverage: DecimalWire | null;
  readonly margin_mode: string | null;
}

export interface MarginSummaryWire {
  readonly asset: string;
  readonly equity: DecimalWire | null;
  readonly margin_balance: DecimalWire | null;
  readonly available_balance: DecimalWire | null;
}

export interface FundingRateWire {
  readonly market: MarketWire;
  readonly timestamp: TimestampWire;
  readonly rate: DecimalWire;
  readonly mark_price: DecimalWire | null;
}

export interface FundingPaymentWire {
  readonly market: MarketWire;
  readonly timestamp: TimestampWire;
  readonly amount: DecimalWire;
  readonly rate: DecimalWire | null;
  readonly id: string | null;
}

export interface PageWire<T> {
  readonly items: readonly T[];
  readonly next: string | null;
}

export type SizeWire =
  | { readonly kind: "base"; readonly value: DecimalWire }
  | { readonly kind: "quote"; readonly value: DecimalWire };

export type FeedWire =
  | { readonly kind: "trades" }
  | { readonly kind: "order_book" }
  | { readonly kind: "ticker" }
  | { readonly kind: "candles"; readonly interval: string };

export interface SubscriptionWire {
  readonly markets: readonly MarketWire[];
  readonly feeds: readonly FeedWire[];
}

export interface StreamConfigWire {
  readonly max_reconnect_attempts: number | null;
  readonly initial_reconnect_delay_ms: number;
  readonly max_reconnect_delay_ms: number;
  readonly idle_timeout_ms: number;
  readonly buffer_size: number;
  readonly overflow: string;
}

export interface CandleRequestWire {
  readonly market: MarketWire;
  readonly interval: string;
  readonly from: TimestampWire | null;
  readonly to: TimestampWire | null;
  readonly limit: number | null;
}

export interface OrderRequestWire {
  readonly market: MarketWire;
  readonly side: string;
  readonly order_type: string;
  readonly size: SizeWire;
  readonly price: DecimalWire | null;
  readonly time_in_force: string | null;
  readonly reduce_only: boolean;
}

export interface HistoryRequestWire {
  readonly market: MarketWire;
  readonly from: TimestampWire | null;
  readonly to: TimestampWire | null;
  readonly cursor: string | null;
  readonly limit: number | null;
}

export interface MarginRequestWire {
  readonly market: MarketWire;
  readonly leverage: DecimalWire | null;
  readonly margin_mode: string | null;
}

export type MarketEventWire =
  | { readonly kind: "trade"; readonly trade: TradeWire }
  | { readonly kind: "order_book"; readonly order_book: OrderBookWire }
  | { readonly kind: "ticker"; readonly ticker: TickerWire }
  | { readonly kind: "candle"; readonly candle: CandleWire }
  | { readonly kind: "reconnected" };

export type AccountEventWire =
  | { readonly kind: "balance"; readonly balance: BalanceWire }
  | { readonly kind: "order"; readonly order: OrderWire }
  | { readonly kind: "reconnected" };

export type MarketStreamItemWire =
  | { readonly kind: "event"; readonly event: MarketEventWire }
  | { readonly kind: "error"; readonly error: ErrorWire };

export type AccountStreamItemWire =
  | { readonly kind: "event"; readonly event: AccountEventWire }
  | { readonly kind: "error"; readonly error: ErrorWire };

export interface UpbitMarketEventWire {
  readonly warning: boolean;
  readonly cautions: readonly string[];
}

export interface BithumbMarketAlertWire {
  readonly kind: string;
  readonly step: string;
  readonly ends_at: TimestampWire;
}

export interface BinanceSymbolFiltersWire {
  readonly symbol: string;
  readonly tick_size: DecimalWire | null;
  readonly min_price: DecimalWire | null;
  readonly max_price: DecimalWire | null;
  readonly step_size: DecimalWire | null;
  readonly min_quantity: DecimalWire | null;
  readonly max_quantity: DecimalWire | null;
  readonly min_notional: DecimalWire | null;
}

export interface BinanceSpotOrderDetailWire {
  readonly order: OrderWire;
  readonly client_order_id: string;
  readonly order_type: string;
  readonly time_in_force: string;
  readonly filled_quote_quantity: DecimalWire;
  readonly updated_at: TimestampWire | null;
}

export interface BinanceListenKeyWire {
  readonly id: string;
  readonly value: string;
}

export interface HyperliquidLedgerEntryWire {
  readonly kind: string;
  readonly time: TimestampWire;
  readonly hash: string;
  readonly asset: string | null;
  readonly amount: DecimalWire | null;
  readonly fee: DecimalWire | null;
  readonly counterparty: string | null;
}

export interface HyperliquidAssetContextWire {
  readonly mid_price: DecimalWire | null;
  readonly mark_price: DecimalWire | null;
  readonly oracle_price: DecimalWire | null;
  readonly funding_rate: DecimalWire | null;
  readonly open_interest: DecimalWire | null;
  readonly size_decimals: number;
  readonly price_decimals: number;
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

export type AdapterCallWire =
  | { readonly kind: "markets"; readonly market_kind: MarketKindWire }
  | { readonly kind: "trades"; readonly market: MarketWire; readonly limit: number | null }
  | { readonly kind: "order_book"; readonly market: MarketWire; readonly depth: number | null }
  | { readonly kind: "ticker"; readonly market: MarketWire }
  | { readonly kind: "candles"; readonly request: CandleRequestWire }
  | {
    readonly kind: "subscribe";
    readonly stream_id: string;
    readonly subscription: SubscriptionWire;
    readonly config: StreamConfigWire;
  }
  | { readonly kind: "balances" }
  | { readonly kind: "open_orders"; readonly market: MarketWire | null }
  | { readonly kind: "subscribe_account"; readonly stream_id: string; readonly config: StreamConfigWire }
  | { readonly kind: "place_order"; readonly request: OrderRequestWire }
  | { readonly kind: "cancel_order"; readonly market: MarketWire; readonly order_id: string }
  | { readonly kind: "positions"; readonly market: MarketWire | null }
  | { readonly kind: "margin_summary" }
  | { readonly kind: "funding_rates"; readonly request: HistoryRequestWire }
  | { readonly kind: "funding_payments"; readonly request: HistoryRequestWire }
  | { readonly kind: "set_margin"; readonly request: MarginRequestWire };

export type AdapterReplyWire =
  | { readonly kind: "markets"; readonly value: readonly MarketInfoWire[] }
  | { readonly kind: "trades"; readonly value: readonly TradeWire[] }
  | { readonly kind: "order_book"; readonly value: OrderBookWire }
  | { readonly kind: "ticker"; readonly value: TickerWire }
  | { readonly kind: "candles"; readonly value: readonly CandleWire[] }
  | { readonly kind: "market_stream"; readonly stream_id: string }
  | { readonly kind: "balances"; readonly value: readonly BalanceWire[] }
  | { readonly kind: "open_orders"; readonly value: readonly OrderWire[] }
  | { readonly kind: "account_stream"; readonly stream_id: string }
  | { readonly kind: "place_order"; readonly value: OrderWire }
  | { readonly kind: "cancel_order"; readonly value: OrderWire }
  | { readonly kind: "positions"; readonly value: readonly PositionWire[] }
  | { readonly kind: "margin_summary"; readonly value: MarginSummaryWire }
  | { readonly kind: "funding_rates"; readonly value: PageWire<FundingRateWire> }
  | { readonly kind: "funding_payments"; readonly value: PageWire<FundingPaymentWire> }
  | { readonly kind: "unit" };

export interface ForeignAdapterCallbacks {
  dispatch(call: AdapterCallWire): Promise<NativeOutcome<AdapterReplyWire>>;
  streamNext(
    id: string,
  ): Promise<NativeOutcome<MarketStreamItemWire | AccountStreamItemWire | null>>;
  streamClose(id: string): Promise<NativeOutcome<null>>;
}

export interface UpbitOptionsWire {
  readonly region: string;
  readonly access_key: string | null;
  readonly secret_key: string | null;
}

export interface BithumbOptionsWire {
  readonly access_key: string | null;
  readonly secret_key: string | null;
}

export interface BinanceOptionsWire {
  readonly venue: string;
  readonly api_key: string | null;
  readonly secret_key: string | null;
}

export interface HyperliquidOptionsWire {
  readonly testnet: boolean;
  readonly address: string | null;
  readonly private_key: string | null;
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
  const normalized = normalizeInitializeOptions(options);
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
  return {
    wasmUrl: options.wasmUrl === undefined ? null : new URL(String(options.wasmUrl), import.meta.url).href,
    allowInsecureBrowserCredentials: options.allowInsecureBrowserCredentials ?? false,
  };
}

function sameInitializeOptions(
  left: NormalizedInitializeOptions,
  right: NormalizedInitializeOptions,
): boolean {
  return left.wasmUrl === right.wasmUrl
    && left.allowInsecureBrowserCredentials === right.allowInsecureBrowserCredentials;
}
