import {
  BithumbAlertStep,
  DepositStatus,
  Exchange,
  HyperliquidLedgerKind,
  Interval,
  MarginMode,
  MarketKind,
  MarketStatus,
  Network,
  OrderIdKind,
  OrderStatus,
  OrderType,
  Overflow,
  Side,
  SizeKind,
  TimeInForce,
  TransferErrorKind,
  WithdrawalStatus,
} from "./generated/identifiers.js";

export {
  BinanceMarket,
  BithumbAlertStep,
  DepositStatus,
  Exchange,
  Feature,
  HyperliquidLedgerKind,
  Interval,
  MarginMode,
  MarketKind,
  MarketStatus,
  Network,
  OrderIdKind,
  OrderStatus,
  OrderType,
  Overflow,
  Side,
  SizeKind,
  TimeInForce,
  TransferErrorKind,
  UpbitRegion,
  WithdrawalStatus,
} from "./generated/identifiers.js";

const MAX_DECIMAL_COEFFICIENT = 79228162514264337593543950335n;
const MAX_DECIMAL_SCALE = 28;
const MAX_DECIMAL_POINT_SHIFT = 64;
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
    const exponentText = match[5];
    const exponent = exponentText === undefined
      ? 0
      : Decimal.boundedExponent(exponentText, integer.length);
    if (exponent === null) throw new RangeError("Decimal scientific notation is too large");
    if (exponent < fraction.length - MAX_DECIMAL_SCALE) {
      throw new RangeError("Decimal scale exceeds 28");
    }

    const scaleValue = fraction.length - exponent;
    const coefficientDigits = Decimal.coefficientDigits(`${integer}${fraction}`, scaleValue);
    if (coefficientDigits === null) throw new RangeError("Decimal coefficient exceeds 96 bits");
    let coefficient = BigInt(coefficientDigits);
    let scale = scaleValue;
    if (coefficient === 0n) {
      scale = 0;
    } else if (scale < 0) {
      coefficient *= pow10(-scale);
      scale = 0;
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
    let finalScale = -1;
    let finalNumerator = 0n;
    let finalDenominator = 1n;
    for (let targetScale = MAX_DECIMAL_SCALE; targetScale >= 0; targetScale -= 1) {
      const shift = targetScale - this.scale;
      const numerator = shift >= 0 ? this.coefficient * pow10(shift) : this.coefficient;
      const denominator = shift >= 0 ? divisor : divisor * pow10(-shift);
      const absoluteNumerator = numerator < 0n ? -numerator : numerator;
      const absoluteDenominator = denominator < 0n ? -denominator : denominator;
      if (absoluteNumerator * 2n < (MAX_DECIMAL_COEFFICIENT * 2n + 1n) * absoluteDenominator) {
        finalScale = targetScale;
        finalNumerator = numerator;
        finalDenominator = denominator;
        break;
      }
    }
    if (finalScale < 0) throw new RangeError("Decimal arithmetic overflow");
    return Decimal.canonical(
      Decimal.roundRationalHalfEven(finalNumerator, finalDenominator),
      finalScale,
    );
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
        return Decimal.canonical(rounded, scale - drop);
      }
    }
    throw new RangeError("Decimal arithmetic overflow");
  }

  private static boundedExponent(text: string, wholeLength: number): number | null {
    const minimum = -MAX_DECIMAL_POINT_SHIFT - wholeLength;
    const maximum = MAX_DECIMAL_POINT_SHIFT - wholeLength;
    let start = 0;
    let negative = false;
    const sign = text.charCodeAt(0);
    if (sign === 45 || sign === 43) {
      negative = sign === 45;
      start += 1;
    }
    while (start < text.length && text.charCodeAt(start) === 48) start += 1;
    if (start === text.length) return minimum <= 0 && maximum >= 0 ? 0 : null;
    if (
      Decimal.compareSignedDigits(negative, text, start, minimum) < 0
      || Decimal.compareSignedDigits(negative, text, start, maximum) > 0
    ) return null;

    let result = 0;
    for (let index = start; index < text.length; index += 1) {
      const digit = text.charCodeAt(index) - 48;
      result = negative ? result * 10 - digit : result * 10 + digit;
    }
    return result;
  }

  private static compareSignedDigits(
    negative: boolean,
    text: string,
    start: number,
    bound: number,
  ): number {
    const boundText = bound.toString();
    const boundNegative = boundText.charCodeAt(0) === 45;
    if (negative !== boundNegative) return negative ? -1 : 1;

    const boundStart = boundNegative ? 1 : 0;
    const length = text.length - start;
    const boundLength = boundText.length - boundStart;
    let comparison = length - boundLength;
    if (comparison === 0) {
      for (let index = 0; index < length; index += 1) {
        comparison = text.charCodeAt(start + index) - boundText.charCodeAt(boundStart + index);
        if (comparison !== 0) break;
      }
    }
    return negative ? -comparison : comparison;
  }

  private static coefficientDigits(digits: string, scale: number): string | null {
    let start = 0;
    while (start < digits.length && digits.charCodeAt(start) === 48) start += 1;
    if (start === digits.length) return "0";

    const significantLength = digits.length - start;
    const appendedZeros = scale < 0 ? -scale : 0;
    const expandedLength = significantLength + appendedZeros;
    const maximum = MAX_DECIMAL_COEFFICIENT.toString();
    if (expandedLength > maximum.length) return null;
    if (expandedLength === maximum.length) {
      for (let index = 0; index < expandedLength; index += 1) {
        const digit = index < significantLength ? digits.charCodeAt(start + index) : 48;
        const maximumDigit = maximum.charCodeAt(index);
        if (digit < maximumDigit) break;
        if (digit > maximumDigit) return null;
      }
    }
    return digits.slice(start);
  }

  private static roundRationalHalfEven(numerator: bigint, denominator: bigint): bigint {
    const negative = (numerator < 0n) !== (denominator < 0n);
    const absoluteNumerator = numerator < 0n ? -numerator : numerator;
    const absoluteDenominator = denominator < 0n ? -denominator : denominator;
    let quotient = absoluteNumerator / absoluteDenominator;
    const doubled = (absoluteNumerator % absoluteDenominator) * 2n;
    if (
      doubled > absoluteDenominator
      || (doubled === absoluteDenominator && quotient % 2n !== 0n)
    ) quotient += 1n;
    return negative ? -quotient : quotient;
  }

  private static canonical(coefficient: bigint, scale: number): Decimal {
    if (coefficient === 0n) return Decimal.zero;
    while (scale > 0 && coefficient % 10n === 0n) {
      coefficient /= 10n;
      scale -= 1;
    }
    return Decimal.create(coefficient, scale, plainDecimal(coefficient, scale));
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

export class OrderAccount {
  constructor(
    readonly balance: Balance,
    readonly averageBuyPrice: Decimal,
    readonly averageBuyPriceModified: boolean,
    readonly averageBuyPriceUnit: string | null,
  ) { freezeRecord(this); }
}

export class OrderOption {
  constructor(
    readonly providerId: string,
    readonly orderType: OrderType | null,
    readonly timeInForce: TimeInForce | null,
  ) { freezeRecord(this); }
}

export class OrderRules {
  readonly sides: readonly Side[];
  readonly buyOptions: readonly OrderOption[];
  readonly sellOptions: readonly OrderOption[];

  constructor(
    readonly market: Market,
    readonly marketName: string,
    readonly status: MarketStatus,
    readonly buyFeeRate: Decimal,
    readonly sellFeeRate: Decimal,
    readonly makerBuyFeeRate: Decimal,
    readonly makerSellFeeRate: Decimal,
    sides: readonly Side[],
    buyOptions: readonly OrderOption[],
    sellOptions: readonly OrderOption[],
    readonly buyPriceUnit: Decimal | null,
    readonly sellPriceUnit: Decimal | null,
    readonly minimumBuyTotal: Decimal,
    readonly minimumSellTotal: Decimal,
    readonly maximumTotal: Decimal,
    readonly quoteAccount: OrderAccount,
    readonly baseAccount: OrderAccount,
  ) {
    this.sides = Object.freeze([...sides]);
    this.buyOptions = Object.freeze([...buyOptions]);
    this.sellOptions = Object.freeze([...sellOptions]);
    freezeRecord(this);
  }
}

export type WithdrawalFee =
  | { readonly kind: "fixed"; readonly value: Decimal }
  | {
    readonly kind: "rate";
    readonly rate: Decimal;
    readonly minimum: Decimal | null;
    readonly maximum: Decimal | null;
  };

export const WithdrawalFee = Object.freeze({
  fixed(value: Decimal): WithdrawalFee {
    return Object.freeze({ kind: "fixed", value });
  },
  rate(
    rate: Decimal,
    minimum: Decimal | null = null,
    maximum: Decimal | null = null,
  ): WithdrawalFee {
    return Object.freeze({ kind: "rate", rate, minimum, maximum });
  },
});

export class AssetNetwork {
  readonly asset: string;
  constructor(
    readonly exchange: Exchange,
    asset: string,
    readonly network: Network,
    readonly providerId: string,
    readonly depositEnabled: boolean,
    readonly withdrawalEnabled: boolean,
    readonly withdrawalFee: WithdrawalFee | null,
    readonly minimumWithdrawal: Decimal | null,
    readonly maximumWithdrawal: Decimal | null,
    readonly memoRequired: boolean,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export class DepositAddress {
  readonly asset: string;
  constructor(
    readonly exchange: Exchange,
    asset: string,
    readonly network: Network,
    readonly address: string | null,
    readonly memo: string | null,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export class DepositAddressEntry {
  readonly asset: string;
  constructor(
    readonly exchange: Exchange,
    asset: string,
    readonly network: Network | null,
    readonly providerNetwork: string | null,
    readonly address: string | null,
    readonly memo: string | null,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export class ExchangeDestination {
  readonly asset: string;
  constructor(
    readonly exchange: Exchange,
    asset: string,
    readonly network: Network,
    readonly address: string,
    readonly memo: string | null = null,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export class ChainDestination {
  readonly asset: string;
  constructor(
    asset: string,
    readonly network: Network,
    readonly address: string,
    readonly memo: string | null = null,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export class ExchangeTransferRequest {
  readonly asset: string;
  constructor(
    asset: string,
    readonly sourceNetwork: Network | null,
    readonly destinationNetwork: Network | null,
    readonly amount: Decimal,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export class ChainTransferRequest {
  readonly asset: string;
  constructor(
    asset: string,
    readonly sourceNetwork: Network | null,
    readonly destination: ChainDestination,
    readonly amount: Decimal,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export type TransferDestination =
  | { readonly kind: "exchange"; readonly value: ExchangeDestination }
  | { readonly kind: "chain"; readonly value: ChainDestination };

export const TransferDestination = Object.freeze({
  exchange(value: ExchangeDestination): TransferDestination {
    return Object.freeze({ kind: "exchange", value });
  },
  chain(value: ChainDestination): TransferDestination {
    return Object.freeze({ kind: "chain", value });
  },
});

export type TravelRuleRequirement =
  | { readonly kind: "not_required" }
  | { readonly kind: "required"; readonly consentUrl: string | null };

export const TravelRuleRequirement = Object.freeze({
  NotRequired: Object.freeze({ kind: "not_required" }) as TravelRuleRequirement,
  required(consentUrl: string | null = null): TravelRuleRequirement {
    return Object.freeze({ kind: "required", consentUrl });
  },
});

export class WithdrawalQuote {
  constructor(
    readonly fee: Decimal | null,
    readonly expectedReceive: Decimal | null,
    readonly minimumAmount: Decimal | null,
    readonly maximumAmount: Decimal | null,
    readonly addressAllowed: boolean | null,
    readonly travelRule: TravelRuleRequirement,
    readonly expiresAt: Timestamp | null,
  ) { freezeRecord(this); }
}

export class TransferPlan {
  constructor(
    readonly source: Exchange,
    readonly destination: Exchange | null,
    readonly request: WithdrawRequest,
    readonly quote: WithdrawalQuote,
    readonly createdAt: Timestamp,
    readonly expiresAt: Timestamp,
  ) { freezeRecord(this); }
}

export class Withdrawal {
  readonly asset: string;
  constructor(
    readonly id: string,
    asset: string,
    readonly network: Network | null,
    readonly providerNetwork: string | null,
    readonly amount: Decimal,
    readonly fee: Decimal | null,
    readonly destination: TransferDestination | null,
    readonly status: WithdrawalStatus,
    readonly providerStatus: string,
    readonly txId: string | null,
    readonly createdAt: Timestamp | null,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export class Deposit {
  readonly asset: string;
  constructor(
    readonly id: string,
    asset: string,
    readonly network: Network | null,
    readonly providerNetwork: string | null,
    readonly amount: Decimal,
    readonly address: string | null,
    readonly memo: string | null,
    readonly status: DepositStatus,
    readonly providerStatus: string,
    readonly txId: string | null,
    readonly createdAt: Timestamp | null,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export class Order {
  constructor(
    readonly id: string, readonly market: Market, readonly side: Side, readonly status: OrderStatus,
    readonly filledQuantity: Decimal, readonly remainingQuantity: Decimal, readonly price: Decimal | null,
    readonly createdAt: Timestamp | null,
  ) { freezeRecord(this); }
}

export class CancelledOrder {
  constructor(
    readonly orderId: string,
    readonly clientId: string | null,
    readonly market: Market | null,
    readonly cancelledAt: Timestamp | null,
  ) { freezeRecord(this); }
}

export class OrderCancelFailure {
  constructor(
    readonly orderId: string | null,
    readonly clientId: string | null,
    readonly market: Market | null,
    readonly code: string | null,
    readonly message: string | null,
  ) { freezeRecord(this); }
}

export class CancelOrdersResult {
  readonly cancelled: readonly CancelledOrder[];
  readonly failed: readonly OrderCancelFailure[];
  constructor(cancelled: readonly CancelledOrder[], failed: readonly OrderCancelFailure[]) {
    this.cancelled = Object.freeze([...cancelled]);
    this.failed = Object.freeze([...failed]);
    freezeRecord(this);
  }
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

export class UpbitYearCandle {
  constructor(
    readonly market: Market, readonly openTime: Timestamp,
    readonly koreaOpenTime: Timestamp | null, readonly timestamp: Timestamp,
    readonly open: Decimal, readonly high: Decimal, readonly low: Decimal,
    readonly close: Decimal, readonly volume: Decimal, readonly quoteVolume: Decimal,
    readonly firstDayOfPeriod: string,
  ) { freezeRecord(this); }
}

export class UpbitOrderBookInstrument {
  readonly supportedLevels: readonly Decimal[];
  constructor(
    readonly market: Market, readonly quoteCurrency: string, readonly tickSize: Decimal,
    supportedLevels: readonly Decimal[],
  ) {
    this.supportedLevels = Object.freeze([...supportedLevels]); freezeRecord(this);
  }
}

export class BithumbMarketAlert {
  constructor(readonly kind: string, readonly step: BithumbAlertStep, readonly endsAt: Timestamp) {
    freezeRecord(this);
  }
}

export class BithumbNotice {
  readonly categories: readonly string[];
  constructor(
    categories: readonly string[], readonly title: string, readonly url: string,
    readonly publishedAt: Timestamp, readonly modifiedAt: Timestamp,
  ) {
    this.categories = Object.freeze([...categories]); freezeRecord(this);
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
    readonly clientId: string | null,
  ) { freezeRecord(this); }
  static market(
    market: Market, side: Side, size: Size,
    options: { timeInForce?: TimeInForce | null; reduceOnly?: boolean; clientId?: string | null } = {},
  ): OrderRequest {
    return new OrderRequest(
      market, side, OrderType.Market, size, null,
      options.timeInForce ?? null, options.reduceOnly ?? false, options.clientId ?? null,
    );
  }
  static limit(
    market: Market, side: Side, size: Size, price: Decimal,
    options: { timeInForce?: TimeInForce | null; reduceOnly?: boolean; clientId?: string | null } = {},
  ): OrderRequest {
    return new OrderRequest(
      market, side, OrderType.Limit, size, price,
      options.timeInForce ?? null, options.reduceOnly ?? false, options.clientId ?? null,
    );
  }
  static best(
    market: Market, side: Side, size: Size, timeInForce: TimeInForce,
    options: { reduceOnly?: boolean; clientId?: string | null } = {},
  ): OrderRequest {
    return new OrderRequest(
      market, side, OrderType.Best, size, null,
      timeInForce, options.reduceOnly ?? false, options.clientId ?? null,
    );
  }
}

export class OrderHistoryRequest {
  readonly statuses: readonly OrderStatus[];
  readonly limit: number | null;
  constructor(
    readonly market: Market | null = null,
    statuses: readonly OrderStatus[] = [],
    readonly from: Timestamp | null = null,
    readonly to: Timestamp | null = null,
    readonly cursor: Cursor | null = null,
    limit: number | null = null,
  ) {
    this.statuses = Object.freeze([...statuses]);
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
  }
}

export class OrderLookupRequest {
  readonly ids: readonly string[];
  constructor(
    readonly kind: OrderIdKind,
    ids: readonly string[],
    readonly market: Market | null = null,
  ) {
    this.ids = Object.freeze([...ids]);
    freezeRecord(this);
  }
}

export class CancelOrdersRequest {
  readonly ids: readonly string[];
  constructor(readonly kind: OrderIdKind, ids: readonly string[]) {
    this.ids = Object.freeze([...ids]);
    freezeRecord(this);
  }
}

export class DepositAddressRequest {
  readonly asset: string;
  constructor(
    asset: string,
    readonly network: Network,
    readonly amount: Decimal | null = null,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export class WithdrawRequest {
  readonly asset: string;
  constructor(
    asset: string,
    readonly network: Network,
    readonly amount: Decimal,
    readonly destination: TransferDestination,
    readonly clientId: string | null = null,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export class TransferLookupRequest {
  readonly asset: string;
  constructor(
    asset: string,
    readonly id: string | null = null,
    readonly txId: string | null = null,
  ) {
    this.asset = asciiUpper(asset);
    freezeRecord(this);
  }
}

export class TransferHistoryRequest {
  readonly asset: string | null;
  readonly limit: number | null;
  constructor(
    asset: string | null = null,
    readonly network: Network | null = null,
    readonly cursor: Cursor | null = null,
    limit: number | null = null,
  ) {
    this.asset = asset === null ? null : asciiUpper(asset);
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
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
    const uniqueMarkets: Market[] = [];
    const feedKeys = new Set<string>();
    for (const market of markets) {
      // ponytail: 구독 목록은 작습니다. 규모가 문제가 되면 구조 키 Map으로 교체합니다.
      const duplicate = uniqueMarkets.some((existing) =>
        existing.exchange === market.exchange
        && existing.kind === market.kind
        && existing.base === market.base
        && existing.quote === market.quote
      );
      if (!duplicate) uniqueMarkets.push(market);
    }
    this.markets = Object.freeze(uniqueMarkets);
    this.feeds = Object.freeze(feeds.filter((feed) => {
      const key = `${feed.kind}:${feed.interval?.id ?? ""}`;
      if (feedKeys.has(key)) return false;
      feedKeys.add(key);
      return true;
    }));
    freezeRecord(this);
  }
  withMarket(market: Market): Subscription { return this.withMarkets([market]); }
  withMarkets(markets: readonly Market[]): Subscription {
    return new Subscription([...this.markets, ...markets], this.feeds);
  }
  withFeed(feed: Feed): Subscription { return new Subscription(this.markets, [...this.feeds, feed]); }
}
