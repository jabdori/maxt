const MAX_DECIMAL_COEFFICIENT = 79228162514264337593543950335n;
const MAX_DECIMAL_SCALE = 28;
const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;
const U32_MAX = 0xffff_ffff;

function pow10(exponent: number): bigint {
  return 10n ** BigInt(exponent);
}

function plainDecimal(coefficient: bigint, scale: number): string {
  const sign = coefficient < 0n ? "-" : "";
  const digits = (coefficient < 0n ? -coefficient : coefficient).toString();
  if (scale === 0) return `${sign}${digits}`;
  if (digits.length <= scale) return `${sign}0.${digits.padStart(scale, "0")}`;
  const point = digits.length - scale;
  return `${sign}${digits.slice(0, point)}.${digits.slice(point)}`;
}

function roundHalfEven(coefficient: bigint, digits: number): bigint {
  if (digits === 0) return coefficient;
  const divisor = pow10(digits);
  const negative = coefficient < 0n;
  const absolute = negative ? -coefficient : coefficient;
  let quotient = absolute / divisor;
  const doubled = (absolute % divisor) * 2n;
  if (doubled > divisor || (doubled === divisor && quotient % 2n !== 0n)) quotient += 1n;
  return negative ? -quotient : quotient;
}

export class Decimal {
  static readonly zero = new Decimal(0n);
  static readonly one = new Decimal(1n);

  readonly coefficient!: bigint;
  readonly scale!: number;
  private readonly text!: string;

  constructor(coefficient: bigint, scale = 0) {
    return Decimal.create(coefficient, scale, plainDecimal(coefficient, scale));
  }

  static parse(text: string): Decimal {
    const match = /^([+-]?)(?:(\d+)(?:\.(\d*))?|\.(\d+))(?:[eE]([+-]?\d+))?$/.exec(text);
    if (match === null) throw new RangeError("invalid Decimal");

    const integer = match[2] ?? "";
    const fraction = match[3] ?? match[4] ?? "";
    const exponent = BigInt(match[5] ?? "0");
    const scaleValue = BigInt(fraction.length) - exponent;
    if (scaleValue > BigInt(MAX_DECIMAL_SCALE)) throw new RangeError("Decimal scale exceeds 28");

    const unsignedDigits = `${integer}${fraction}`.replace(/^0+(?=\d)/, "");
    let coefficient: bigint;
    let scale: number;
    if (scaleValue < 0n) {
      const shift = -scaleValue;
      if (shift > 29n || BigInt(unsignedDigits.length) + shift > 29n) {
        throw new RangeError("Decimal coefficient exceeds 96 bits");
      }
      coefficient = BigInt(unsignedDigits) * pow10(Number(shift));
      scale = 0;
    } else {
      coefficient = BigInt(unsignedDigits);
      scale = Number(scaleValue);
    }
    if (match[1] === "-") coefficient = -coefficient;
    return Decimal.create(coefficient, scale, text);
  }

  add(other: Decimal): Decimal {
    return this.addOrSubtract(other, false);
  }

  subtract(other: Decimal): Decimal {
    return this.addOrSubtract(other, true);
  }

  divideByInteger(divisor: bigint): Decimal {
    if (divisor === 0n) throw new RangeError("Decimal division by zero");
    const extraScale = MAX_DECIMAL_SCALE - this.scale;
    const numerator = this.coefficient * pow10(extraScale);
    const negative = (numerator < 0n) !== (divisor < 0n);
    const absoluteNumerator = numerator < 0n ? -numerator : numerator;
    const absoluteDivisor = divisor < 0n ? -divisor : divisor;
    let quotient = absoluteNumerator / absoluteDivisor;
    const doubled = (absoluteNumerator % absoluteDivisor) * 2n;
    if (doubled > absoluteDivisor || (doubled === absoluteDivisor && quotient % 2n !== 0n)) {
      quotient += 1n;
    }
    return Decimal.fromArithmetic(negative ? -quotient : quotient, MAX_DECIMAL_SCALE);
  }

  compareTo(other: Decimal): number {
    const scale = Math.max(this.scale, other.scale);
    const left = this.coefficient * pow10(scale - this.scale);
    const right = other.coefficient * pow10(scale - other.scale);
    return left < right ? -1 : left > right ? 1 : 0;
  }

  equals(other: Decimal): boolean {
    return this.compareTo(other) === 0;
  }

  toString(): string {
    return this.text;
  }

  [Symbol.toPrimitive](hint: string): string {
    if (hint === "number") throw new TypeError("Decimal cannot be converted to Number");
    return this.text;
  }

  private addOrSubtract(other: Decimal, subtract: boolean): Decimal {
    const scale = Math.max(this.scale, other.scale);
    const left = this.coefficient * pow10(scale - this.scale);
    const right = other.coefficient * pow10(scale - other.scale);
    return Decimal.fromArithmetic(subtract ? left - right : left + right, scale);
  }

  private static fromArithmetic(coefficient: bigint, scale: number): Decimal {
    if (coefficient === 0n) return Decimal.zero;
    const minimumDrop = Math.max(0, scale - MAX_DECIMAL_SCALE);
    for (let drop = minimumDrop; drop <= scale; drop += 1) {
      const rounded = roundHalfEven(coefficient, drop);
      if ((rounded < 0n ? -rounded : rounded) <= MAX_DECIMAL_COEFFICIENT) {
        let canonical = rounded;
        let canonicalScale = scale - drop;
        while (canonicalScale > 0 && canonical % 10n === 0n) {
          canonical /= 10n;
          canonicalScale -= 1;
        }
        return Decimal.create(canonical, canonicalScale, plainDecimal(canonical, canonicalScale));
      }
    }
    throw new RangeError("Decimal arithmetic overflow");
  }

  private static create(coefficient: bigint, scale: number, text: string): Decimal {
    if (typeof coefficient !== "bigint") throw new TypeError("coefficient must be a bigint");
    if (!Number.isInteger(scale) || scale < 0 || scale > MAX_DECIMAL_SCALE) {
      throw new RangeError("scale must be an integer from 0 through 28");
    }
    if ((coefficient < 0n ? -coefficient : coefficient) > MAX_DECIMAL_COEFFICIENT) {
      throw new RangeError("Decimal coefficient exceeds 96 bits");
    }
    const value = Object.create(Decimal.prototype) as Decimal;
    Object.defineProperties(value, {
      coefficient: { value: coefficient, enumerable: true },
      scale: { value: scale, enumerable: true },
      text: { value: text },
    });
    return Object.freeze(value) as Decimal;
  }
}

export class Timestamp {
  static readonly zero = new Timestamp(0n);

  private constructor(readonly nanosecondsSinceEpoch: bigint) {
    Object.freeze(this);
  }

  static fromNanoseconds(value: bigint): Timestamp {
    if (typeof value !== "bigint") throw new TypeError("nanoseconds must be a bigint");
    if (value < I64_MIN || value > I64_MAX) throw new RangeError("nanoseconds exceed signed i64");
    if (value === 0n) return Timestamp.zero;
    return new Timestamp(value);
  }

  static fromMicroseconds(value: bigint): Timestamp {
    return Timestamp.fromScaled(value, 1_000n);
  }

  static fromMilliseconds(value: bigint): Timestamp {
    return Timestamp.fromScaled(value, 1_000_000n);
  }

  static fromSeconds(value: bigint): Timestamp {
    return Timestamp.fromScaled(value, 1_000_000_000n);
  }

  static fromDate(value: Date): Timestamp {
    const milliseconds = value.getTime();
    if (!Number.isFinite(milliseconds)) throw new RangeError("Date is invalid");
    return Timestamp.fromMilliseconds(BigInt(Math.trunc(milliseconds)));
  }

  static now(): Timestamp {
    return Timestamp.fromDate(new Date());
  }

  get millisecondsSinceEpoch(): bigint {
    return this.nanosecondsSinceEpoch / 1_000_000n;
  }

  get secondsSinceEpoch(): bigint {
    return this.nanosecondsSinceEpoch / 1_000_000_000n;
  }

  /** Converts to Date at millisecond precision; sub-millisecond precision is discarded. */
  toDate(): Date {
    return new Date(Number(this.millisecondsSinceEpoch));
  }

  compareTo(other: Timestamp): number {
    return this.nanosecondsSinceEpoch < other.nanosecondsSinceEpoch
      ? -1
      : this.nanosecondsSinceEpoch > other.nanosecondsSinceEpoch
        ? 1
        : 0;
  }

  equals(other: Timestamp): boolean {
    return this.nanosecondsSinceEpoch === other.nanosecondsSinceEpoch;
  }

  private static fromScaled(value: bigint, scale: bigint): Timestamp {
    if (typeof value !== "bigint") throw new TypeError("timestamp value must be a bigint");
    const nanoseconds = value * scale;
    if (nanoseconds < I64_MIN) return new Timestamp(I64_MIN);
    if (nanoseconds > I64_MAX) return new Timestamp(I64_MAX);
    return Timestamp.fromNanoseconds(nanoseconds);
  }
}

class StringValue {
  protected constructor(readonly id: string) {}
  toString(): string { return this.id; }
}

export class Exchange extends StringValue {
  static readonly Upbit = new Exchange("upbit", "Upbit");
  static readonly Bithumb = new Exchange("bithumb", "Bithumb");
  static readonly Binance = new Exchange("binance", "Binance");
  static readonly Hyperliquid = new Exchange("hyperliquid", "Hyperliquid");
  static readonly values: readonly Exchange[] = Object.freeze([
    Exchange.Upbit, Exchange.Bithumb, Exchange.Binance, Exchange.Hyperliquid,
  ]);
  private constructor(id: string, readonly displayName: string) { super(id); Object.freeze(this); }
}

export class Feature extends StringValue {
  static readonly Markets = new Feature("markets", false, false);
  static readonly Trades = new Feature("trades", false, false);
  static readonly OrderBook = new Feature("order_book", false, false);
  static readonly Ticker = new Feature("ticker", false, false);
  static readonly Candles = new Feature("candles", false, false);
  static readonly TradeStream = new Feature("trade_stream", false, false);
  static readonly OrderBookStream = new Feature("order_book_stream", false, false);
  static readonly TickerStream = new Feature("ticker_stream", false, false);
  static readonly CandleStream = new Feature("candle_stream", false, false);
  static readonly Balances = new Feature("balances", true, false);
  static readonly OpenOrders = new Feature("open_orders", true, false);
  static readonly AccountStream = new Feature("account_stream", true, false);
  static readonly Trading = new Feature("trading", true, false);
  static readonly Positions = new Feature("positions", true, true);
  static readonly Margin = new Feature("margin", true, true);
  static readonly FundingRates = new Feature("funding_rates", false, true);
  static readonly FundingPayments = new Feature("funding_payments", true, true);
  static readonly MarginConfig = new Feature("margin_config", true, true);
  static readonly ReduceOnlyOrders = new Feature("reduce_only_orders", true, true);
  static readonly values: readonly Feature[] = Object.freeze([
    Feature.Markets, Feature.Trades, Feature.OrderBook, Feature.Ticker, Feature.Candles,
    Feature.TradeStream, Feature.OrderBookStream, Feature.TickerStream, Feature.CandleStream,
    Feature.Balances, Feature.OpenOrders, Feature.AccountStream, Feature.Trading, Feature.Positions,
    Feature.Margin, Feature.FundingRates, Feature.FundingPayments, Feature.MarginConfig,
    Feature.ReduceOnlyOrders,
  ]);
  private constructor(id: string, readonly needsCredentials: boolean, readonly isDerivativesOnly: boolean) {
    super(id); Object.freeze(this);
  }
}

export class MarketKind extends StringValue {
  static readonly Spot = new MarketKind("spot", false);
  static readonly Perpetual = new MarketKind("perpetual", true);
  static readonly values: readonly MarketKind[] = Object.freeze([MarketKind.Spot, MarketKind.Perpetual]);
  private constructor(id: string, readonly isDerivative: boolean) { super(id); Object.freeze(this); }
}

export class MarketStatus extends StringValue {
  static readonly Active = new MarketStatus("active");
  static readonly Paused = new MarketStatus("paused");
  static readonly Delisted = new MarketStatus("delisted");
  static readonly Unknown = new MarketStatus("unknown");
  static readonly values: readonly MarketStatus[] = Object.freeze([
    MarketStatus.Active, MarketStatus.Paused, MarketStatus.Delisted, MarketStatus.Unknown,
  ]);
  private constructor(id: string) { super(id); Object.freeze(this); }
}

export class Side extends StringValue {
  static readonly Buy = new Side("buy");
  static readonly Sell = new Side("sell");
  static readonly values: readonly Side[] = Object.freeze([Side.Buy, Side.Sell]);
  private constructor(id: string) { super(id); Object.freeze(this); }
  get flipped(): Side { return this === Side.Buy ? Side.Sell : Side.Buy; }
}

export class Interval extends StringValue {
  static readonly Sec1 = new Interval("sec1", 1);
  static readonly Min1 = new Interval("min1", 60);
  static readonly Min3 = new Interval("min3", 180);
  static readonly Min5 = new Interval("min5", 300);
  static readonly Min15 = new Interval("min15", 900);
  static readonly Min30 = new Interval("min30", 1_800);
  static readonly Hour1 = new Interval("hour1", 3_600);
  static readonly Hour2 = new Interval("hour2", 7_200);
  static readonly Hour4 = new Interval("hour4", 14_400);
  static readonly Hour8 = new Interval("hour8", 28_800);
  static readonly Hour12 = new Interval("hour12", 43_200);
  static readonly Day1 = new Interval("day1", 86_400);
  static readonly Day3 = new Interval("day3", 259_200);
  static readonly Week1 = new Interval("week1", 604_800);
  static readonly Month1 = new Interval("month1", null);
  static readonly values: readonly Interval[] = Object.freeze([
    Interval.Sec1, Interval.Min1, Interval.Min3, Interval.Min5, Interval.Min15, Interval.Min30,
    Interval.Hour1, Interval.Hour2, Interval.Hour4, Interval.Hour8, Interval.Hour12, Interval.Day1,
    Interval.Day3, Interval.Week1, Interval.Month1,
  ]);
  private constructor(id: string, readonly seconds: number | null) { super(id); Object.freeze(this); }
}

export class Overflow extends StringValue {
  static readonly Backpressure = new Overflow("backpressure");
  static readonly DropNewest = new Overflow("drop_newest");
  static readonly values: readonly Overflow[] = Object.freeze([Overflow.Backpressure, Overflow.DropNewest]);
  private constructor(id: string) { super(id); Object.freeze(this); }
}

export class MarginMode extends StringValue {
  static readonly Cross = new MarginMode("cross");
  static readonly Isolated = new MarginMode("isolated");
  static readonly values: readonly MarginMode[] = Object.freeze([MarginMode.Cross, MarginMode.Isolated]);
  private constructor(id: string) { super(id); Object.freeze(this); }
}

export class OrderStatus extends StringValue {
  static readonly Accepted = new OrderStatus("accepted", true);
  static readonly Open = new OrderStatus("open", true);
  static readonly PartiallyFilled = new OrderStatus("partially_filled", true);
  static readonly Filled = new OrderStatus("filled", false);
  static readonly Cancelled = new OrderStatus("cancelled", false);
  static readonly Rejected = new OrderStatus("rejected", false);
  static readonly Unknown = new OrderStatus("unknown", false);
  static readonly values: readonly OrderStatus[] = Object.freeze([
    OrderStatus.Accepted, OrderStatus.Open, OrderStatus.PartiallyFilled, OrderStatus.Filled,
    OrderStatus.Cancelled, OrderStatus.Rejected, OrderStatus.Unknown,
  ]);
  private constructor(id: string, readonly isLive: boolean) { super(id); Object.freeze(this); }
}

export class OrderType extends StringValue {
  static readonly Market = new OrderType("market");
  static readonly Limit = new OrderType("limit");
  static readonly values: readonly OrderType[] = Object.freeze([OrderType.Market, OrderType.Limit]);
  private constructor(id: string) { super(id); Object.freeze(this); }
}

export class TimeInForce extends StringValue {
  static readonly GoodTilCancelled = new TimeInForce("good_til_cancelled");
  static readonly ImmediateOrCancel = new TimeInForce("immediate_or_cancel");
  static readonly FillOrKill = new TimeInForce("fill_or_kill");
  static readonly PostOnly = new TimeInForce("post_only");
  static readonly values: readonly TimeInForce[] = Object.freeze([
    TimeInForce.GoodTilCancelled, TimeInForce.ImmediateOrCancel,
    TimeInForce.FillOrKill, TimeInForce.PostOnly,
  ]);
  private constructor(id: string) { super(id); Object.freeze(this); }
}

export class SizeKind extends StringValue {
  static readonly Base = new SizeKind("base");
  static readonly Quote = new SizeKind("quote");
  static readonly values: readonly SizeKind[] = Object.freeze([SizeKind.Base, SizeKind.Quote]);
  private constructor(id: string) { super(id); Object.freeze(this); }
}

export class UpbitRegion extends StringValue {
  static readonly Korea = new UpbitRegion("korea");
  static readonly Singapore = new UpbitRegion("singapore");
  static readonly Indonesia = new UpbitRegion("indonesia");
  static readonly Thailand = new UpbitRegion("thailand");
  static readonly values: readonly UpbitRegion[] = Object.freeze([
    UpbitRegion.Korea, UpbitRegion.Singapore, UpbitRegion.Indonesia, UpbitRegion.Thailand,
  ]);
  private constructor(id: string) { super(id); Object.freeze(this); }
}

export class BithumbAlertStep extends StringValue {
  static readonly Caution = new BithumbAlertStep("caution");
  static readonly Warning = new BithumbAlertStep("warning");
  static readonly Danger = new BithumbAlertStep("danger");
  static readonly Unknown = new BithumbAlertStep("unknown");
  static readonly values: readonly BithumbAlertStep[] = Object.freeze([
    BithumbAlertStep.Caution, BithumbAlertStep.Warning, BithumbAlertStep.Danger,
    BithumbAlertStep.Unknown,
  ]);
  private constructor(id: string) { super(id); Object.freeze(this); }
}

export class BinanceMarket extends StringValue {
  static readonly Spot = new BinanceMarket("spot");
  static readonly UsdMFutures = new BinanceMarket("usd_m");
  static readonly values: readonly BinanceMarket[] = Object.freeze([BinanceMarket.Spot, BinanceMarket.UsdMFutures]);
  private constructor(id: string) { super(id); Object.freeze(this); }
}

export class HyperliquidLedgerKind extends StringValue {
  static readonly Deposit = new HyperliquidLedgerKind("deposit");
  static readonly Withdraw = new HyperliquidLedgerKind("withdraw");
  static readonly InternalTransfer = new HyperliquidLedgerKind("internal_transfer");
  static readonly SubAccountTransfer = new HyperliquidLedgerKind("sub_account_transfer");
  static readonly SpotTransfer = new HyperliquidLedgerKind("spot_transfer");
  static readonly AccountClassTransfer = new HyperliquidLedgerKind("account_class_transfer");
  static readonly VaultDeposit = new HyperliquidLedgerKind("vault_deposit");
  static readonly VaultWithdraw = new HyperliquidLedgerKind("vault_withdraw");
  static readonly VaultDistribution = new HyperliquidLedgerKind("vault_distribution");
  static readonly Liquidation = new HyperliquidLedgerKind("liquidation");
  static readonly values: readonly HyperliquidLedgerKind[] = Object.freeze([
    HyperliquidLedgerKind.Deposit, HyperliquidLedgerKind.Withdraw,
    HyperliquidLedgerKind.InternalTransfer, HyperliquidLedgerKind.SubAccountTransfer,
    HyperliquidLedgerKind.SpotTransfer, HyperliquidLedgerKind.AccountClassTransfer,
    HyperliquidLedgerKind.VaultDeposit, HyperliquidLedgerKind.VaultWithdraw,
    HyperliquidLedgerKind.VaultDistribution, HyperliquidLedgerKind.Liquidation,
  ]);
  private constructor(id: string) { super(id); Object.freeze(this); }
  static other(value: string): HyperliquidLedgerKind { return new HyperliquidLedgerKind(value); }
}

function asciiUpper(value: string): string {
  return value.replace(/[a-z]/g, (character) => character.toUpperCase());
}

function freezeRecord<T extends object>(value: T): void {
  Object.freeze(value);
}

function checkedUnsigned(value: number, maximum: number, name: string): number {
  if (!Number.isSafeInteger(value) || value < 0 || value > maximum) {
    throw new RangeError(`${name} must be a non-negative safe integer within its Rust range`);
  }
  return value;
}

function checkedOptionalU32(value: number | null, name: string): number | null {
  return value === null ? null : checkedUnsigned(value, U32_MAX, name);
}

export class Size {
  private constructor(readonly kind: SizeKind, readonly value: Decimal) { freezeRecord(this); }
  static base(value: Decimal): Size { return new Size(SizeKind.Base, value); }
  static quote(value: Decimal): Size { return new Size(SizeKind.Quote, value); }
}

export class Feed {
  static readonly Trades = new Feed("trades", null);
  static readonly OrderBook = new Feed("order_book", null);
  static readonly Ticker = new Feed("ticker", null);
  private constructor(
    readonly kind: "trades" | "order_book" | "ticker" | "candles",
    readonly interval: Interval | null,
  ) { freezeRecord(this); }
  static candles(interval: Interval): Feed { return new Feed("candles", interval); }
}

export type MarketEvent =
  | { readonly kind: "trade"; readonly trade: Trade }
  | { readonly kind: "order_book"; readonly orderBook: OrderBook }
  | { readonly kind: "ticker"; readonly ticker: Ticker }
  | { readonly kind: "candle"; readonly candle: Candle }
  | { readonly kind: "reconnected" };

export type AccountEvent =
  | { readonly kind: "balance"; readonly balance: Balance }
  | { readonly kind: "order"; readonly order: Order }
  | { readonly kind: "reconnected" };

export class Market {
  readonly base: string;
  readonly quote: string;
  constructor(readonly exchange: Exchange, readonly kind: MarketKind, base: string, quote: string) {
    this.base = asciiUpper(base);
    this.quote = asciiUpper(quote);
    freezeRecord(this);
  }
  static spot(exchange: Exchange, base: string, quote: string): Market {
    return new Market(exchange, MarketKind.Spot, base, quote);
  }
  static perpetual(exchange: Exchange, base: string, quote: string): Market {
    return new Market(exchange, MarketKind.Perpetual, base, quote);
  }
  toString(): string {
    return `${this.exchange.id}:${this.base}/${this.quote}${this.kind === MarketKind.Perpetual ? ":perp" : ""}`;
  }
}

export class MarketInfo {
  constructor(
    readonly market: Market, readonly nativeSymbol: string, readonly status: MarketStatus,
    readonly koreanName: string | null, readonly englishName: string | null,
  ) { freezeRecord(this); }
}

export class Trade {
  constructor(
    readonly market: Market, readonly timestamp: Timestamp, readonly price: Decimal,
    readonly quantity: Decimal, readonly takerSide: Side, readonly id: string | null,
  ) { freezeRecord(this); }
}

export class Level {
  constructor(readonly price: Decimal, readonly quantity: Decimal) { freezeRecord(this); }
}

export class OrderBook {
  readonly bids: readonly Level[];
  readonly asks: readonly Level[];
  constructor(readonly market: Market, readonly timestamp: Timestamp, bids: readonly Level[], asks: readonly Level[]) {
    this.bids = Object.freeze([...bids]);
    this.asks = Object.freeze([...asks]);
    freezeRecord(this);
  }
  get bestBid(): Level | null { return this.bids[0] ?? null; }
  get bestAsk(): Level | null { return this.asks[0] ?? null; }
  get spread(): Decimal | null {
    return this.bestBid === null || this.bestAsk === null
      ? null : this.bestAsk.price.subtract(this.bestBid.price);
  }
  get midPrice(): Decimal | null {
    return this.bestBid === null || this.bestAsk === null
      ? null : this.bestBid.price.add(this.bestAsk.price).divideByInteger(2n);
  }
}

export class Ticker {
  constructor(
    readonly market: Market, readonly timestamp: Timestamp, readonly lastTradeTime: Timestamp | null,
    readonly lastPrice: Decimal, readonly change: Decimal | null, readonly changeRate: Decimal | null,
    readonly high: Decimal | null, readonly low: Decimal | null, readonly volume: Decimal | null,
    readonly quoteVolume: Decimal | null,
  ) { freezeRecord(this); }
}

export class Candle {
  constructor(
    readonly market: Market, readonly interval: Interval, readonly openTime: Timestamp,
    readonly open: Decimal, readonly high: Decimal, readonly low: Decimal, readonly close: Decimal,
    readonly volume: Decimal, readonly quoteVolume: Decimal | null, readonly closed: boolean,
  ) { freezeRecord(this); }
}

export class Balance {
  readonly asset: string;
  constructor(asset: string, readonly available: Decimal, readonly locked: Decimal) {
    this.asset = asciiUpper(asset); freezeRecord(this);
  }
  get total(): Decimal { return this.available.add(this.locked); }
}

export class Order {
  constructor(
    readonly id: string, readonly market: Market, readonly side: Side, readonly status: OrderStatus,
    readonly filledQuantity: Decimal, readonly remainingQuantity: Decimal, readonly price: Decimal | null,
    readonly createdAt: Timestamp | null,
  ) { freezeRecord(this); }
}

export class Position {
  constructor(
    readonly market: Market, readonly side: Side | null, readonly quantity: Decimal,
    readonly entryPrice: Decimal | null, readonly markPrice: Decimal | null,
    readonly notional: Decimal | null, readonly unrealizedPnl: Decimal | null,
    readonly leverage: Decimal | null, readonly marginMode: MarginMode | null,
  ) { freezeRecord(this); }
  get isFlat(): boolean { return this.quantity.equals(Decimal.zero); }
}

export class MarginSummary {
  readonly asset: string;
  constructor(
    asset: string, readonly equity: Decimal | null, readonly marginBalance: Decimal | null,
    readonly availableBalance: Decimal | null,
  ) { this.asset = asciiUpper(asset); freezeRecord(this); }
}

export class FundingRate {
  constructor(
    readonly market: Market, readonly timestamp: Timestamp, readonly rate: Decimal,
    readonly markPrice: Decimal | null,
  ) { freezeRecord(this); }
}

export class FundingPayment {
  constructor(
    readonly market: Market, readonly timestamp: Timestamp, readonly amount: Decimal,
    readonly rate: Decimal | null, readonly id: string | null,
  ) { freezeRecord(this); }
}

export class Cursor {
  constructor(readonly value: string) { freezeRecord(this); }
  toString(): string { return this.value; }
}

export class Page<T> {
  readonly items: readonly T[];
  constructor(items: readonly T[], readonly next: Cursor | null) {
    this.items = Object.freeze([...items]); freezeRecord(this);
  }
  get hasMore(): boolean { return this.next !== null; }
}

export class UpbitMarketEvent {
  readonly cautions: readonly string[];
  constructor(readonly warning: boolean, cautions: readonly string[]) {
    this.cautions = Object.freeze([...cautions]); freezeRecord(this);
  }
}

export class BithumbMarketAlert {
  constructor(readonly kind: string, readonly step: BithumbAlertStep, readonly endsAt: Timestamp) {
    freezeRecord(this);
  }
}

export class BinanceSymbolFilters {
  constructor(
    readonly symbol: string, readonly tickSize: Decimal | null, readonly minPrice: Decimal | null,
    readonly maxPrice: Decimal | null, readonly stepSize: Decimal | null,
    readonly minQuantity: Decimal | null, readonly maxQuantity: Decimal | null,
    readonly minNotional: Decimal | null,
  ) { freezeRecord(this); }
}

export class BinanceSpotOrderDetail {
  constructor(
    readonly order: Order, readonly clientOrderId: string, readonly orderType: string,
    readonly timeInForce: string, readonly filledQuoteQuantity: Decimal,
    readonly updatedAt: Timestamp | null,
  ) { freezeRecord(this); }
}

export class HyperliquidLedgerEntry {
  constructor(
    readonly kind: HyperliquidLedgerKind, readonly time: Timestamp, readonly hash: string,
    readonly asset: string | null, readonly amount: Decimal | null, readonly fee: Decimal | null,
    readonly counterparty: string | null,
  ) { freezeRecord(this); }
}

export class HyperliquidAssetContext {
  readonly sizeDecimals: number;
  readonly priceDecimals: number;
  constructor(
    readonly midPrice: Decimal | null, readonly markPrice: Decimal | null,
    readonly oraclePrice: Decimal | null, readonly fundingRate: Decimal | null,
    readonly openInterest: Decimal | null, sizeDecimals: number, priceDecimals: number,
  ) {
    this.sizeDecimals = checkedUnsigned(sizeDecimals, U32_MAX, "sizeDecimals");
    this.priceDecimals = checkedUnsigned(priceDecimals, U32_MAX, "priceDecimals");
    freezeRecord(this);
  }
}

export class CandleRequest {
  readonly limit: number | null;
  constructor(
    readonly market: Market, readonly interval: Interval, readonly from: Timestamp | null = null,
    readonly to: Timestamp | null = null, limit: number | null = null,
  ) { this.limit = checkedOptionalU32(limit, "limit"); freezeRecord(this); }
}

export class OrderRequest {
  private constructor(
    readonly market: Market, readonly side: Side, readonly orderType: OrderType, readonly size: Size,
    readonly price: Decimal | null, readonly timeInForce: TimeInForce | null, readonly reduceOnly: boolean,
  ) { freezeRecord(this); }
  static market(
    market: Market, side: Side, size: Size,
    options: { timeInForce?: TimeInForce | null; reduceOnly?: boolean } = {},
  ): OrderRequest {
    return new OrderRequest(
      market, side, OrderType.Market, size, null,
      options.timeInForce ?? null, options.reduceOnly ?? false,
    );
  }
  static limit(
    market: Market, side: Side, size: Size, price: Decimal,
    options: { timeInForce?: TimeInForce | null; reduceOnly?: boolean } = {},
  ): OrderRequest {
    return new OrderRequest(
      market, side, OrderType.Limit, size, price,
      options.timeInForce ?? null, options.reduceOnly ?? false,
    );
  }
}

export class HistoryRequest {
  readonly limit: number | null;
  constructor(
    readonly market: Market, readonly from: Timestamp | null = null,
    readonly to: Timestamp | null = null, readonly cursor: Cursor | null = null,
    limit: number | null = null,
  ) { this.limit = checkedOptionalU32(limit, "limit"); freezeRecord(this); }
}

export class MarginRequest {
  constructor(
    readonly market: Market, readonly leverage: Decimal | null = null,
    readonly marginMode: MarginMode | null = null,
  ) { freezeRecord(this); }
}

export class StreamConfig {
  readonly maxReconnectAttempts: number | null;
  readonly initialReconnectDelayMs: number;
  readonly maxReconnectDelayMs: number;
  readonly idleTimeoutMs: number;
  readonly bufferSize: number;
  readonly overflow: Overflow;
  constructor(options: {
    maxReconnectAttempts?: number | null;
    initialReconnectDelayMs?: number;
    maxReconnectDelayMs?: number;
    idleTimeoutMs?: number;
    bufferSize?: number;
    overflow?: Overflow;
  } = {}) {
    this.maxReconnectAttempts = checkedOptionalU32(options.maxReconnectAttempts ?? null, "maxReconnectAttempts");
    this.initialReconnectDelayMs = checkedUnsigned(
      options.initialReconnectDelayMs ?? 1_000, Number.MAX_SAFE_INTEGER, "initialReconnectDelayMs",
    );
    this.maxReconnectDelayMs = checkedUnsigned(
      options.maxReconnectDelayMs ?? 30_000, Number.MAX_SAFE_INTEGER, "maxReconnectDelayMs",
    );
    this.idleTimeoutMs = checkedUnsigned(
      options.idleTimeoutMs ?? 30_000, Number.MAX_SAFE_INTEGER, "idleTimeoutMs",
    );
    this.bufferSize = checkedUnsigned(
      options.bufferSize ?? 4_096, Number.MAX_SAFE_INTEGER, "bufferSize",
    );
    this.overflow = options.overflow ?? Overflow.Backpressure;
    freezeRecord(this);
  }
}

export class Subscription {
  readonly markets: readonly Market[];
  readonly feeds: readonly Feed[];
  constructor(markets: readonly Market[] = [], feeds: readonly Feed[] = []) {
    this.markets = Object.freeze([...markets]);
    this.feeds = Object.freeze([...feeds]);
    freezeRecord(this);
  }
  withMarket(market: Market): Subscription { return this.withMarkets([market]); }
  withMarkets(markets: readonly Market[]): Subscription {
    return new Subscription([...this.markets, ...markets], this.feeds);
  }
  withFeed(feed: Feed): Subscription { return new Subscription(this.markets, [...this.feeds, feed]); }
}
