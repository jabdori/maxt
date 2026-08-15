import {
  BithumbAlertStep,
  BithumbClosedOrderState,
  BithumbOrderDirection,
  BithumbOrderListState,
  BithumbPendingOrderState,
  BithumbTwapOrderDirection,
  BithumbTwapState,
  BinanceC2cTradeType,
  BinanceMarket,
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
  UpbitOrderDirection,
  UpbitClosedOrderState,
  UpbitKrwTwoFactorType,
  UpbitPocketTransferDirection,
  UpbitPocketTransferOrder,
  UpbitPocketTransferState,
  UpbitSmpType,
  WithdrawalStatus,
} from "./generated/identifiers.js";

export {
  BinanceMarket,
  BithumbAlertStep,
  BithumbClosedOrderState,
  BithumbOrderDirection,
  BithumbOrderListState,
  BithumbPendingOrderState,
  BithumbTwapOrderDirection,
  BithumbTwapState,
  BinanceC2cTradeType,
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
  UpbitOrderDirection,
  UpbitClosedOrderState,
  UpbitRegion,
  UpbitKrwTwoFactorType,
  UpbitPocketTransferDirection,
  UpbitPocketTransferOrder,
  UpbitPocketTransferState,
  UpbitSmpType,
  WithdrawalStatus,
} from "./generated/identifiers.js";

export * from "./generated/provider_models.js";

const MAX_DECIMAL_COEFFICIENT = 79228162514264337593543950335n;
const MAX_DECIMAL_SCALE = 28;
const MAX_DECIMAL_POINT_SHIFT = 64;
const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;
const U64_MAX = (1n << 64n) - 1n;
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

function checkedU64(value: bigint, name: string): bigint {
  if (typeof value !== "bigint" || value < 0n || value > U64_MAX) {
    throw new RangeError(`${name} must be an unsigned 64-bit integer`);
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

export class UpbitListedSubscription {
  readonly markets: readonly Market[];
  constructor(
    readonly feedType: string,
    markets: readonly Market[],
    readonly level: Decimal | null,
  ) {
    this.markets = Object.freeze([...markets]); freezeRecord(this);
  }
}

export class UpbitSubscriptionList {
  readonly subscriptions: readonly UpbitListedSubscription[];
  constructor(readonly ticket: string, subscriptions: readonly UpbitListedSubscription[]) {
    this.subscriptions = Object.freeze([...subscriptions]); freezeRecord(this);
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

export class UpbitOrderDetailRequest {
  constructor(
    readonly market: Market,
    readonly uuid: string | null = null,
    readonly identifier: string | null = null,
  ) { freezeRecord(this); }
}

export class UpbitOrderDetailTrade {
  constructor(
    readonly market: Market,
    readonly uuid: string,
    readonly price: Decimal,
    readonly volume: Decimal,
    readonly funds: Decimal,
    readonly trend: string,
    readonly createdAt: Timestamp,
    readonly side: string,
  ) { freezeRecord(this); }
}

export class UpbitOrderDetail {
  readonly tradesCount: number;
  readonly trades: readonly UpbitOrderDetailTrade[];
  constructor(
    readonly market: Market,
    readonly uuid: string,
    readonly side: string,
    readonly orderType: string,
    readonly price: Decimal | null,
    readonly state: string,
    readonly createdAt: Timestamp,
    readonly volume: Decimal | null,
    readonly remainingVolume: Decimal,
    readonly executedVolume: Decimal,
    readonly reservedFee: Decimal,
    readonly remainingFee: Decimal,
    readonly paidFee: Decimal,
    readonly locked: Decimal,
    tradesCount: number,
    readonly preventedVolume: Decimal,
    readonly preventedLocked: Decimal,
    readonly timeInForce: string | null,
    readonly identifier: string | null,
    readonly smpType: string | null,
    trades: readonly UpbitOrderDetailTrade[],
  ) {
    this.tradesCount = checkedUnsigned(tradesCount, U32_MAX, "tradesCount");
    this.trades = Object.freeze([...trades]);
    freezeRecord(this);
  }
}

export class UpbitClosedOrdersRequest {
  readonly states: readonly UpbitClosedOrderState[];
  readonly limit: number | null;
  constructor(
    readonly market: Market | null = null,
    readonly state: UpbitClosedOrderState | null = null,
    states: readonly UpbitClosedOrderState[] = [],
    readonly startTime: Timestamp | null = null,
    readonly endTime: Timestamp | null = null,
    limit: number | null = null,
    readonly orderBy: UpbitOrderDirection | null = null,
  ) {
    this.states = Object.freeze([...states]);
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
  }
}

export class UpbitClosedOrder {
  readonly tradesCount: number;
  constructor(
    readonly market: Market,
    readonly uuid: string,
    readonly side: string,
    readonly ordType: string,
    readonly state: string,
    readonly createdAt: Timestamp,
    readonly volume: Decimal | null,
    readonly price: Decimal | null,
    readonly remainingVolume: Decimal,
    readonly executedVolume: Decimal,
    readonly executedFunds: Decimal | null,
    readonly reservedFee: Decimal,
    readonly remainingFee: Decimal,
    readonly paidFee: Decimal,
    readonly locked: Decimal,
    tradesCount: number,
    readonly preventedVolume: Decimal,
    readonly preventedLocked: Decimal,
    readonly timeInForce: string | null,
    readonly identifier: string | null,
    readonly smpType: string | null,
  ) {
    this.tradesCount = checkedUnsigned(tradesCount, U32_MAX, "tradesCount");
    freezeRecord(this);
  }
}

export class UpbitDepositInfo {
  readonly asset: string;
  readonly minimumDepositConfirmations: bigint;
  readonly decimalPrecision: bigint;
  constructor(
    asset: string,
    readonly network: Network | null,
    readonly providerNetwork: string | null,
    readonly isDepositPossible: boolean,
    readonly depositImpossibleReason: string | null,
    readonly minimumDepositAmount: Decimal,
    minimumDepositConfirmations: bigint,
    decimalPrecision: bigint,
  ) {
    this.asset = asciiUpper(asset);
    this.minimumDepositConfirmations = checkedU64(
      minimumDepositConfirmations,
      "minimumDepositConfirmations",
    );
    this.decimalPrecision = checkedU64(decimalPrecision, "decimalPrecision");
    freezeRecord(this);
  }
}

export class UpbitWithdrawalAddress {
  constructor(
    readonly currency: string,
    readonly netType: string,
    readonly networkName: string,
    readonly withdrawAddress: string,
    readonly secondaryAddress: string | null,
    readonly beneficiaryName: string | null,
    readonly beneficiaryCompanyName: string | null,
    readonly beneficiaryType: string | null,
    readonly exchangeName: string | null,
    readonly walletType: string | null,
    readonly rawJson: string,
  ) { freezeRecord(this); }
}

export class UpbitTravelRuleVasp {
  constructor(
    readonly vaspName: string,
    readonly vaspUuid: string,
    readonly depositable: boolean,
    readonly withdrawable: boolean,
  ) { freezeRecord(this); }
}

export class UpbitTravelRuleVerification {
  constructor(
    readonly depositUuid: string,
    readonly depositState: string,
    readonly verificationResult: string,
  ) { freezeRecord(this); }
}

export type UpbitBatchCancelScope =
  | { readonly kind: "all" }
  | { readonly kind: "quote_currencies"; readonly values: readonly string[] }
  | { readonly kind: "pairs"; readonly values: readonly Market[] };

export const UpbitBatchCancelScope = Object.freeze({
  all(): UpbitBatchCancelScope {
    return Object.freeze({ kind: "all" });
  },
  quoteCurrencies(values: readonly string[]): UpbitBatchCancelScope {
    return Object.freeze({
      kind: "quote_currencies",
      values: Object.freeze([...values]),
    });
  },
  pairs(values: readonly Market[]): UpbitBatchCancelScope {
    return Object.freeze({ kind: "pairs", values: Object.freeze([...values]) });
  },
});

export class UpbitBatchCancelRequest {
  readonly excludedPairs: readonly Market[] | null;
  readonly count: number | null;

  constructor(
    readonly scope: UpbitBatchCancelScope,
    excludedPairs: readonly Market[] | null = null,
    readonly side: Side | null = null,
    count: number | null = null,
    readonly orderBy: UpbitOrderDirection | null = null,
  ) {
    this.excludedPairs = excludedPairs === null ? null : Object.freeze([...excludedPairs]);
    this.count = checkedOptionalU32(count, "count");
    freezeRecord(this);
  }
}

export type UpbitOrderReference =
  | { readonly kind: "uuid"; readonly value: string }
  | { readonly kind: "identifier"; readonly value: string };

export const UpbitOrderReference = Object.freeze({
  uuid(value: string): UpbitOrderReference { return Object.freeze({ kind: "uuid", value }); },
  identifier(value: string): UpbitOrderReference {
    return Object.freeze({ kind: "identifier", value });
  },
});

export type UpbitOrderVolume =
  | { readonly kind: "amount"; readonly value: Decimal }
  | { readonly kind: "remain_only" };

export const UpbitOrderVolume = Object.freeze({
  amount(value: Decimal): UpbitOrderVolume { return Object.freeze({ kind: "amount", value }); },
  remainOnly(): UpbitOrderVolume { return Object.freeze({ kind: "remain_only" }); },
});

export type UpbitCancelAndNewOrder =
  | {
    readonly kind: "limit";
    readonly volume: UpbitOrderVolume;
    readonly price: Decimal;
    readonly timeInForce: TimeInForce | null;
  }
  | { readonly kind: "market_buy"; readonly price: Decimal }
  | { readonly kind: "market_sell"; readonly volume: UpbitOrderVolume }
  | { readonly kind: "best_buy"; readonly price: Decimal; readonly timeInForce: TimeInForce }
  | {
    readonly kind: "best_sell";
    readonly volume: UpbitOrderVolume;
    readonly timeInForce: TimeInForce;
  };

export const UpbitCancelAndNewOrder = Object.freeze({
  limit(
    volume: UpbitOrderVolume, price: Decimal, timeInForce: TimeInForce | null = null,
  ): UpbitCancelAndNewOrder {
    return Object.freeze({ kind: "limit", volume, price, timeInForce });
  },
  marketBuy(price: Decimal): UpbitCancelAndNewOrder {
    return Object.freeze({ kind: "market_buy", price });
  },
  marketSell(volume: UpbitOrderVolume): UpbitCancelAndNewOrder {
    return Object.freeze({ kind: "market_sell", volume });
  },
  bestBuy(price: Decimal, timeInForce: TimeInForce): UpbitCancelAndNewOrder {
    return Object.freeze({ kind: "best_buy", price, timeInForce });
  },
  bestSell(volume: UpbitOrderVolume, timeInForce: TimeInForce): UpbitCancelAndNewOrder {
    return Object.freeze({ kind: "best_sell", volume, timeInForce });
  },
});

export class UpbitCancelAndNewOrderRequest {
  constructor(
    readonly previousOrder: UpbitOrderReference,
    readonly newOrder: UpbitCancelAndNewOrder,
    readonly newIdentifier: string | null = null,
    readonly newSmpType: UpbitSmpType | null = null,
  ) { freezeRecord(this); }
}

export class UpbitCancelAndNewOrderResult {
  constructor(
    readonly previousOrder: Order,
    readonly newOrderUuid: string | null,
    readonly newOrderIdentifier: string | null,
  ) { freezeRecord(this); }

  get replacementCreated(): boolean { return this.newOrderUuid !== null; }
}

export class UpbitKrwTransferRequest {
  constructor(readonly amount: Decimal, readonly twoFactorType: UpbitKrwTwoFactorType) {
    freezeRecord(this);
  }
}

export class UpbitKrwDeposit {
  constructor(
    readonly transferType: string,
    readonly uuid: string,
    readonly currency: string,
    readonly netType: string | null,
    readonly txid: string,
    readonly state: string,
    readonly createdAt: Timestamp,
    readonly doneAt: Timestamp | null,
    readonly amount: Decimal,
    readonly fee: Decimal,
    readonly transactionType: string,
  ) { freezeRecord(this); }
}

export class UpbitKrwWithdrawal {
  constructor(
    readonly transferType: string,
    readonly uuid: string,
    readonly currency: string,
    readonly netType: string | null,
    readonly txid: string | null,
    readonly state: string,
    readonly createdAt: Timestamp,
    readonly doneAt: Timestamp | null,
    readonly amount: Decimal,
    readonly fee: Decimal,
    readonly transactionType: string,
    readonly isCancelable: boolean | null,
  ) { freezeRecord(this); }
}

export class UpbitApiKey {
  constructor(readonly accessKey: string, readonly expiresAt: Timestamp) { freezeRecord(this); }
}

export class UpbitPocket {
  constructor(readonly uuid: string, readonly name: string, readonly kind: string) { freezeRecord(this); }
}

export class UpbitPocketApiKey {
  readonly permissions: readonly string[];
  readonly allowedIps: readonly string[];
  constructor(
    readonly accessKey: string,
    permissions: readonly string[],
    allowedIps: readonly string[],
    readonly createdAt: Timestamp,
    readonly expiredAt: Timestamp,
  ) {
    this.permissions = Object.freeze([...permissions]);
    this.allowedIps = Object.freeze([...allowedIps]);
    freezeRecord(this);
  }
}

export class UpbitPocketApiKeyGroup {
  readonly keys: readonly UpbitPocketApiKey[];
  constructor(readonly uuid: string, keys: readonly UpbitPocketApiKey[]) {
    this.keys = Object.freeze([...keys]);
    freezeRecord(this);
  }
}

export class UpbitPocketApiKeysRequest {
  readonly uuids: readonly string[];
  constructor(uuids: readonly string[] = [], readonly includeExpired: boolean = false) {
    this.uuids = Object.freeze([...uuids]);
    freezeRecord(this);
  }
}

export class UpbitPocketBalance {
  constructor(
    readonly currency: string,
    readonly balance: Decimal,
    readonly locked: Decimal,
    readonly avgBuyPrice: Decimal,
    readonly avgBuyPriceModified: boolean,
    readonly unitCurrency: string,
  ) { freezeRecord(this); }
}

export class UpbitPocketTransferQuery {
  readonly states: readonly UpbitPocketTransferState[];
  readonly uuids: readonly string[];
  readonly identifiers: readonly string[];
  readonly limit: number | null;
  constructor(
    readonly from: string | null = null,
    readonly to: string | null = null,
    readonly direction: UpbitPocketTransferDirection | null = null,
    states: readonly UpbitPocketTransferState[] = [],
    uuids: readonly string[] = [],
    identifiers: readonly string[] = [],
    readonly startTime: Timestamp | null = null,
    readonly endTime: Timestamp | null = null,
    readonly currency: string | null = null,
    limit: number | null = null,
    readonly orderBy: UpbitPocketTransferOrder | null = null,
  ) {
    this.states = Object.freeze([...states]);
    this.uuids = Object.freeze([...uuids]);
    this.identifiers = Object.freeze([...identifiers]);
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
  }
}

export class UpbitPocketUniversalTransferRequest {
  constructor(
    readonly from: string | null,
    readonly to: string,
    readonly currency: string,
    readonly amount: Decimal,
    readonly identifier: string | null = null,
  ) { freezeRecord(this); }
}

export class UpbitPocketTransferRequest {
  constructor(
    readonly to: string,
    readonly currency: string,
    readonly amount: Decimal,
    readonly identifier: string | null = null,
  ) { freezeRecord(this); }
}

export class UpbitPocketTransfer {
  constructor(
    readonly uuid: string,
    readonly identifier: string | null,
    readonly from: string,
    readonly to: string,
    readonly state: string,
    readonly currency: string,
    readonly amount: Decimal,
    readonly createdAt: Timestamp,
  ) { freezeRecord(this); }
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

export class BithumbApiKey {
  constructor(readonly accessKey: string, readonly expiresAt: Timestamp) { freezeRecord(this); }
}

export class BithumbKrwWithdrawalsRequest {
  readonly uuids: readonly string[];
  readonly txids: readonly string[];
  readonly page: number | null;
  readonly limit: number | null;
  constructor(
    readonly state: string | null = null,
    uuids: readonly string[] = [],
    txids: readonly string[] = [],
    page: number | null = null,
    limit: number | null = null,
    readonly orderBy: BithumbOrderDirection | null = null,
  ) {
    this.uuids = Object.freeze([...uuids]);
    this.txids = Object.freeze([...txids]);
    this.page = checkedOptionalU32(page, "page");
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
  }
}

export class BithumbKrwDepositsRequest {
  readonly uuids: readonly string[];
  readonly txids: readonly string[];
  readonly page: number | null;
  readonly limit: number | null;
  constructor(
    readonly state: string | null = null,
    uuids: readonly string[] = [],
    txids: readonly string[] = [],
    page: number | null = null,
    limit: number | null = null,
    readonly orderBy: BithumbOrderDirection | null = null,
  ) {
    this.uuids = Object.freeze([...uuids]);
    this.txids = Object.freeze([...txids]);
    this.page = checkedOptionalU32(page, "page");
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
  }
}

export class BithumbKrwTransferRequest {
  constructor(readonly amount: Decimal) { freezeRecord(this); }
}

export class BithumbKrwWithdrawal {
  constructor(
    readonly transferType: string,
    readonly uuid: string,
    readonly currency: string,
    readonly netType: string | null,
    readonly txid: string | null,
    readonly state: string,
    readonly createdAt: Timestamp | null,
    readonly doneAt: Timestamp | null,
    readonly amount: Decimal,
    readonly fee: Decimal,
    readonly transactionType: string | null,
  ) { freezeRecord(this); }
}

export class BithumbKrwDeposit {
  constructor(
    readonly transferType: string,
    readonly uuid: string,
    readonly currency: string,
    readonly netType: string | null,
    readonly txid: string | null,
    readonly state: string,
    readonly createdAt: Timestamp | null,
    readonly doneAt: Timestamp | null,
    readonly amount: Decimal,
    readonly fee: Decimal,
    readonly transactionType: string | null,
  ) { freezeRecord(this); }
}

export class BithumbPendingOrdersRequest {
  readonly limit: number | null;
  constructor(
    readonly market: Market | null = null,
    readonly state: BithumbPendingOrderState | null = null,
    limit: number | null = null,
    readonly orderBy: BithumbOrderDirection | null = null,
    readonly cursor: Cursor | null = null,
  ) {
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
  }
}

export class BithumbClosedOrdersRequest {
  readonly states: readonly BithumbClosedOrderState[];
  readonly limit: number | null;
  constructor(
    readonly market: Market | null = null,
    readonly state: BithumbClosedOrderState | null = null,
    states: readonly BithumbClosedOrderState[] = [],
    readonly startTime: Timestamp | null = null,
    readonly endTime: Timestamp | null = null,
    limit: number | null = null,
    readonly orderBy: BithumbOrderDirection | null = null,
    readonly cursor: Cursor | null = null,
  ) {
    this.states = Object.freeze([...states]);
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
  }
}

export class BithumbClosedOrder {
  readonly tradesCount: number;
  constructor(
    readonly orderId: string,
    readonly side: string,
    readonly orderType: string,
    readonly price: Decimal | null,
    readonly state: string,
    readonly market: Market,
    readonly createdAt: Timestamp | null,
    readonly volume: Decimal,
    readonly remainingVolume: Decimal,
    readonly reservedFee: Decimal,
    readonly remainingFee: Decimal,
    readonly paidFee: Decimal,
    readonly locked: Decimal,
    readonly executedVolume: Decimal,
    readonly executedFunds: Decimal,
    tradesCount: number,
    readonly clientOrderId: string | null,
    readonly stpType: string | null,
    readonly timeInForce: string | null,
    readonly cancelType: string | null,
    readonly cancelingOrderId: string | null,
  ) {
    this.tradesCount = checkedUnsigned(tradesCount, U32_MAX, "tradesCount");
    freezeRecord(this);
  }
}

export class BithumbBatchOrdersRequest {
  readonly orders: readonly OrderRequest[];
  constructor(orders: readonly OrderRequest[]) {
    this.orders = Object.freeze([...orders]);
    freezeRecord(this);
  }
}

export class BithumbBatchOrder {
  constructor(
    readonly orderId: string,
    readonly clientOrderId: string | null,
    readonly market: Market,
    readonly side: Side,
    readonly orderType: OrderType,
    readonly timeInForce: string | null,
    readonly stpType: string | null,
    readonly createdAt: Timestamp | null,
  ) { freezeRecord(this); }
}

export class BithumbBatchOrderFailure {
  constructor(
    readonly clientOrderId: string | null,
    readonly timeInForce: string | null,
    readonly code: string,
    readonly message: string,
  ) { freezeRecord(this); }
}

export type BithumbBatchOrderOutcome =
  | { readonly kind: "accepted"; readonly value: BithumbBatchOrder }
  | { readonly kind: "rejected"; readonly value: BithumbBatchOrderFailure };

export const BithumbBatchOrderOutcome = Object.freeze({
  accepted(value: BithumbBatchOrder): BithumbBatchOrderOutcome {
    return Object.freeze({ kind: "accepted", value });
  },
  rejected(value: BithumbBatchOrderFailure): BithumbBatchOrderOutcome {
    return Object.freeze({ kind: "rejected", value });
  },
});

export class BithumbBatchOrdersResult {
  readonly outcomes: readonly BithumbBatchOrderOutcome[];
  constructor(
    outcomes: readonly BithumbBatchOrderOutcome[],
    readonly rawJson: string,
  ) {
    this.outcomes = Object.freeze([...outcomes]);
    freezeRecord(this);
  }
}

export class BithumbTwapOrdersRequest {
  readonly limit: number | null;
  readonly uuids: readonly string[];
  constructor(
    readonly market: Market | null = null,
    uuids: readonly string[] = [],
    readonly state: BithumbTwapState | null = null,
    readonly cursor: Cursor | null = null,
    limit: number | null = null,
    readonly orderBy: BithumbTwapOrderDirection | null = null,
  ) {
    this.uuids = Object.freeze([...uuids]);
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
  }
}

export class BithumbTwapOrderRequest {
  readonly duration: number;
  readonly frequency: number;
  constructor(
    readonly market: Market,
    readonly side: Side,
    readonly volume: Decimal | null,
    readonly price: Decimal | null,
    duration: number,
    frequency: number,
  ) {
    this.duration = checkedUnsigned(duration, U32_MAX, "duration");
    this.frequency = checkedUnsigned(frequency, U32_MAX, "frequency");
    freezeRecord(this);
  }
}

export class BithumbTwapOrder {
  readonly totalOrderCount: number;
  readonly totalTradesCount: number;
  readonly progressCount: number;
  constructor(
    readonly id: string,
    readonly side: Side,
    readonly price: Decimal,
    readonly state: BithumbTwapState,
    readonly market: Market,
    readonly createdAt: Timestamp,
    readonly volume: Decimal,
    readonly finishedAt: Timestamp | null,
    totalOrderCount: number,
    totalTradesCount: number,
    progressCount: number,
    readonly totalExecutedAmount: Decimal,
    readonly totalExecutedVolume: Decimal,
    readonly avgTradePrice: Decimal,
    readonly walletId: string | null,
    readonly canceledAt: Timestamp | null,
    readonly cancelType: string | null,
  ) {
    this.totalOrderCount = checkedUnsigned(totalOrderCount, U32_MAX, "totalOrderCount");
    this.totalTradesCount = checkedUnsigned(totalTradesCount, U32_MAX, "totalTradesCount");
    this.progressCount = checkedUnsigned(progressCount, U32_MAX, "progressCount");
    freezeRecord(this);
  }
}

export class BithumbAssetFee {
  readonly asset: string;
  readonly networks: readonly BithumbNetworkFee[];
  constructor(
    readonly displayName: string, asset: string, networks: readonly BithumbNetworkFee[],
  ) {
    this.asset = asciiUpper(asset);
    this.networks = Object.freeze([...networks]);
    freezeRecord(this);
  }
}

export class BithumbNetworkFee {
  constructor(
    readonly network: Network,
    readonly providerName: string,
    readonly depositFee: Decimal,
    readonly minimumDeposit: Decimal,
    readonly withdrawalFee: WithdrawalFee,
    readonly minimumWithdrawal: Decimal,
  ) { freezeRecord(this); }
}

export class BithumbWithdrawalAddress {
  constructor(
    readonly currency: string,
    readonly netType: string,
    readonly networkName: string | null,
    readonly withdrawAddress: string,
    readonly secondaryAddress: string | null,
    readonly exchangeName: string | null,
    readonly ownerType: string | null,
    readonly ownerKoName: string | null,
    readonly ownerEnName: string | null,
    readonly ownerCorpKoName: string | null,
    readonly ownerCorpEnName: string | null,
  ) { freezeRecord(this); }
}

export class BithumbOrderDetailRequest {
  constructor(
    readonly market: Market,
    readonly uuid: string | null = null,
    readonly clientOrderId: string | null = null,
  ) { freezeRecord(this); }
}

export class BithumbOrderDetailTrade {
  constructor(
    readonly market: Market,
    readonly uuid: string,
    readonly price: Decimal,
    readonly volume: Decimal,
    readonly funds: Decimal,
    readonly side: string,
    readonly createdAt: Timestamp,
  ) { freezeRecord(this); }
}

export class BithumbOrderDetail {
  readonly trades: readonly BithumbOrderDetailTrade[];
  readonly tradesCount: number;
  constructor(
    readonly uuid: string,
    readonly clientOrderId: string | null,
    readonly side: string,
    readonly orderType: string,
    readonly price: Decimal,
    readonly state: string,
    readonly market: Market,
    readonly createdAt: Timestamp,
    readonly volume: Decimal,
    readonly remainingVolume: Decimal,
    readonly reservedFee: Decimal,
    readonly remainingFee: Decimal,
    readonly paidFee: Decimal,
    readonly locked: Decimal,
    readonly executedVolume: Decimal,
    readonly executedFunds: Decimal,
    tradesCount: number,
    trades: readonly BithumbOrderDetailTrade[],
    readonly stpType: string | null,
    readonly cancelType: string | null,
    readonly cancelingUuid: string | null,
    readonly timeInForce: string | null,
  ) {
    this.tradesCount = checkedUnsigned(tradesCount, U32_MAX, "tradesCount");
    this.trades = Object.freeze([...trades]);
    freezeRecord(this);
  }
}

export class BithumbOrderListRequest {
  readonly states: readonly BithumbOrderListState[];
  readonly uuids: readonly string[];
  readonly clientOrderIds: readonly string[];
  readonly page: number | null;
  readonly limit: number | null;
  constructor(
    readonly market: Market | null = null,
    readonly state: BithumbOrderListState | null = null,
    states: readonly BithumbOrderListState[] = [],
    uuids: readonly string[] = [],
    clientOrderIds: readonly string[] = [],
    page: number | null = null,
    limit: number | null = null,
    readonly orderBy: BithumbOrderDirection | null = null,
  ) {
    this.states = Object.freeze([...states]);
    this.uuids = Object.freeze([...uuids]);
    this.clientOrderIds = Object.freeze([...clientOrderIds]);
    this.page = checkedOptionalU32(page, "page");
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
  }
}

export class BithumbOrderListItem {
  readonly tradesCount: number;
  constructor(
    readonly uuid: string,
    readonly clientOrderId: string | null,
    readonly side: string,
    readonly orderType: string,
    readonly price: Decimal,
    readonly state: string,
    readonly market: Market,
    readonly createdAt: Timestamp,
    readonly volume: Decimal,
    readonly remainingVolume: Decimal,
    readonly reservedFee: Decimal,
    readonly remainingFee: Decimal,
    readonly paidFee: Decimal,
    readonly locked: Decimal,
    readonly executedVolume: Decimal,
    readonly executedFunds: Decimal,
    tradesCount: number,
    readonly stpType: string | null,
    readonly timeInForce: string | null,
    readonly rawJson: string,
  ) {
    this.tradesCount = checkedUnsigned(tradesCount, U32_MAX, "tradesCount");
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

export class BinanceSpotAveragePrice {
  constructor(
    readonly market: Market, readonly minutes: number, readonly price: Decimal,
    readonly closeTime: Timestamp,
  ) { freezeRecord(this); }
}

export class BinanceDepositHistoryRequest {
  readonly status: number | null;
  readonly offset: bigint | null;
  readonly limit: number | null;
  constructor(
    readonly coin: string | null = null,
    status: number | null = null,
    readonly startTime: Timestamp | null = null,
    readonly endTime: Timestamp | null = null,
    offset: bigint | null = null,
    limit: number | null = null,
    readonly txId: string | null = null,
    readonly includeSource: boolean = false,
  ) {
    this.status = checkedOptionalU32(status, "status");
    this.offset = offset === null ? null : checkedU64(offset, "offset");
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
  }
}

export class BinanceWithdrawHistoryRequest {
  readonly status: number | null;
  readonly offset: bigint | null;
  readonly limit: number | null;
  readonly idList: readonly string[];
  constructor(
    readonly coin: string | null = null,
    readonly withdrawOrderId: string | null = null,
    status: number | null = null,
    offset: bigint | null = null,
    limit: number | null = null,
    idList: readonly string[] = [],
    readonly startTime: Timestamp | null = null,
    readonly endTime: Timestamp | null = null,
  ) {
    this.status = checkedOptionalU32(status, "status");
    this.offset = offset === null ? null : checkedU64(offset, "offset");
    this.limit = checkedOptionalU32(limit, "limit");
    this.idList = Object.freeze([...idList]);
    freezeRecord(this);
  }
}

export class BinanceSpotCommissionRates {
  constructor(
    readonly maker: Decimal, readonly taker: Decimal,
    readonly buyer: Decimal, readonly seller: Decimal,
  ) { freezeRecord(this); }
}

export class BinanceSpotAccountBalance {
  constructor(readonly asset: string, readonly free: Decimal, readonly locked: Decimal) {
    freezeRecord(this);
  }
}

export class BinanceSpotAccountInformation {
  readonly makerCommission: bigint;
  readonly takerCommission: bigint;
  readonly buyerCommission: bigint;
  readonly sellerCommission: bigint;
  readonly balances: readonly BinanceSpotAccountBalance[];
  readonly permissions: readonly string[];
  readonly uid: bigint | null;
  constructor(
    makerCommission: bigint,
    takerCommission: bigint,
    buyerCommission: bigint,
    sellerCommission: bigint,
    readonly commissionRates: BinanceSpotCommissionRates,
    readonly canTrade: boolean,
    readonly canWithdraw: boolean,
    readonly canDeposit: boolean,
    readonly updateTime: Timestamp,
    readonly accountType: string,
    balances: readonly BinanceSpotAccountBalance[],
    permissions: readonly string[],
    uid: bigint | null,
    readonly rawJson: string,
  ) {
    this.makerCommission = checkedU64(makerCommission, "makerCommission");
    this.takerCommission = checkedU64(takerCommission, "takerCommission");
    this.buyerCommission = checkedU64(buyerCommission, "buyerCommission");
    this.sellerCommission = checkedU64(sellerCommission, "sellerCommission");
    this.balances = Object.freeze([...balances]);
    this.permissions = Object.freeze([...permissions]);
    this.uid = uid === null ? null : checkedU64(uid, "uid");
    freezeRecord(this);
  }
}

export class BinanceSpotCancelledOrder {
  constructor(
    readonly symbol: string | null,
    readonly originalClientOrderId: string | null,
    readonly orderId: string | null,
    readonly clientOrderId: string | null,
    readonly status: string | null,
    readonly price: Decimal | null,
    readonly originalQuantity: Decimal | null,
    readonly executedQuantity: Decimal | null,
    readonly cumulativeQuoteQuantity: Decimal | null,
    readonly transactTime: Timestamp | null,
    readonly orderListId: string | null,
    readonly contingencyType: string | null,
    readonly listStatusType: string | null,
    readonly listOrderStatus: string | null,
    readonly listClientOrderId: string | null,
    readonly transactionTime: Timestamp | null,
    readonly rawJson: string,
  ) { freezeRecord(this); }
}

export class BinanceSpotCancelAllOpenOrders {
  readonly reports: readonly BinanceSpotCancelledOrder[];
  constructor(reports: readonly BinanceSpotCancelledOrder[], readonly rawJson: string) {
    this.reports = Object.freeze([...reports]);
    freezeRecord(this);
  }
}

export class BinanceExchangeSymbol {
  constructor(
    readonly symbol: string,
    readonly status: string,
    readonly baseAsset: string,
    readonly quoteAsset: string,
    readonly contractType: string | null,
    readonly marginAsset: string | null,
    readonly rawJson: string,
  ) { freezeRecord(this); }
}

export class BinanceExchangeInfo {
  readonly symbols: readonly BinanceExchangeSymbol[];
  constructor(
    readonly venue: BinanceMarket,
    readonly timezone: string | null,
    readonly serverTime: Timestamp | null,
    symbols: readonly BinanceExchangeSymbol[],
    readonly rawJson: string,
  ) {
    this.symbols = Object.freeze([...symbols]);
    freezeRecord(this);
  }
}

export class BinanceUsdMAccountAsset {
  constructor(
    readonly asset: string,
    readonly walletBalance: Decimal,
    readonly unrealizedProfit: Decimal,
    readonly marginBalance: Decimal,
    readonly maintenanceMargin: Decimal,
    readonly initialMargin: Decimal,
    readonly positionInitialMargin: Decimal,
    readonly openOrderInitialMargin: Decimal,
    readonly crossWalletBalance: Decimal,
    readonly crossUnrealizedProfit: Decimal,
    readonly availableBalance: Decimal,
    readonly maxWithdrawAmount: Decimal,
    readonly updateTime: Timestamp,
  ) { freezeRecord(this); }
}

export class BinanceUsdMAccountPosition {
  constructor(
    readonly symbol: string,
    readonly positionSide: string,
    readonly positionAmount: Decimal,
    readonly unrealizedProfit: Decimal,
    readonly isolatedMargin: Decimal,
    readonly notional: Decimal,
    readonly isolatedWallet: Decimal,
    readonly initialMargin: Decimal,
    readonly maintenanceMargin: Decimal,
    readonly updateTime: Timestamp,
  ) { freezeRecord(this); }
}

export class BinanceUsdMAccountInformation {
  readonly assets: readonly BinanceUsdMAccountAsset[];
  readonly positions: readonly BinanceUsdMAccountPosition[];
  constructor(
    readonly totalInitialMargin: Decimal,
    readonly totalMaintenanceMargin: Decimal,
    readonly totalWalletBalance: Decimal,
    readonly totalUnrealizedProfit: Decimal,
    readonly totalMarginBalance: Decimal,
    readonly totalPositionInitialMargin: Decimal,
    readonly totalOpenOrderInitialMargin: Decimal,
    readonly totalCrossWalletBalance: Decimal,
    readonly totalCrossUnrealizedProfit: Decimal,
    readonly availableBalance: Decimal,
    readonly maxWithdrawAmount: Decimal,
    assets: readonly BinanceUsdMAccountAsset[],
    positions: readonly BinanceUsdMAccountPosition[],
    readonly rawJson: string,
  ) {
    this.assets = Object.freeze([...assets]);
    this.positions = Object.freeze([...positions]);
    freezeRecord(this);
  }
}

export class BinanceUsdMPositionInformation {
  readonly adl: bigint;
  constructor(
    readonly symbol: string,
    readonly positionSide: string,
    readonly positionAmount: Decimal,
    readonly entryPrice: Decimal,
    readonly breakEvenPrice: Decimal,
    readonly markPrice: Decimal,
    readonly unrealizedProfit: Decimal,
    readonly liquidationPrice: Decimal,
    readonly isolatedMargin: Decimal,
    readonly notional: Decimal,
    readonly marginAsset: string,
    readonly isolatedWallet: Decimal,
    readonly initialMargin: Decimal,
    readonly maintenanceMargin: Decimal,
    readonly positionInitialMargin: Decimal,
    readonly openOrderInitialMargin: Decimal,
    adl: bigint,
    readonly bidNotional: Decimal,
    readonly askNotional: Decimal,
    readonly updateTime: Timestamp,
    readonly rawJson: string,
  ) {
    this.adl = checkedU64(adl, "adl");
    freezeRecord(this);
  }
}

export class BinanceCoinNetworkInformation {
  readonly minimumConfirmations: bigint | null;
  readonly unlockConfirmations: bigint | null;
  constructor(
    readonly network: string,
    readonly depositEnabled: boolean,
    readonly withdrawEnabled: boolean,
    readonly busy: boolean,
    readonly withdrawalIntegerMultiple: Decimal | null,
    readonly withdrawalFee: Decimal | null,
    readonly minimumWithdrawal: Decimal | null,
    readonly maximumWithdrawal: Decimal | null,
    readonly withdrawalTag: boolean | null,
    readonly isDefault: boolean | null,
    minimumConfirmations: bigint | null,
    unlockConfirmations: bigint | null,
    readonly contractAddress: string | null,
    readonly rawJson: string,
  ) {
    this.minimumConfirmations = minimumConfirmations === null
      ? null : checkedU64(minimumConfirmations, "minimumConfirmations");
    this.unlockConfirmations = unlockConfirmations === null
      ? null : checkedU64(unlockConfirmations, "unlockConfirmations");
    freezeRecord(this);
  }
}

export class BinanceCoinInformation {
  readonly networks: readonly BinanceCoinNetworkInformation[];
  constructor(
    readonly coin: string,
    readonly depositAllEnabled: boolean,
    readonly withdrawAllEnabled: boolean,
    readonly name: string | null,
    readonly free: Decimal | null,
    readonly locked: Decimal | null,
    readonly freeze: Decimal | null,
    readonly withdrawing: Decimal | null,
    readonly isLegalMoney: boolean | null,
    readonly trading: boolean | null,
    networks: readonly BinanceCoinNetworkInformation[],
    readonly rawJson: string,
  ) {
    this.networks = Object.freeze([...networks]);
    freezeRecord(this);
  }
}

export class BinanceApiKeyPermissions {
  constructor(
    readonly ipRestrict: boolean,
    readonly createTime: Timestamp | null,
    readonly enableReading: boolean,
    readonly enableWithdrawals: boolean,
    readonly enableInternalTransfer: boolean,
    readonly enableMargin: boolean,
    readonly enableSpotAndMarginTrading: boolean,
    readonly enableFutures: boolean,
    readonly permitsUniversalTransfer: boolean,
    readonly enableVanillaOptions: boolean,
    readonly enableFixApiTrade: boolean,
    readonly enableFixReadOnly: boolean,
    readonly enablePortfolioMarginTrading: boolean,
    readonly rawJson: string,
  ) { freezeRecord(this); }
}

export class BinanceDepositHistoryEntry {
  readonly status: number;
  readonly transferType: number | null;
  constructor(
    readonly id: string,
    readonly amount: Decimal,
    readonly coin: string,
    readonly network: string,
    status: number,
    readonly address: string | null,
    readonly addressTag: string | null,
    readonly txId: string | null,
    readonly insertTime: Timestamp,
    readonly completeTime: Timestamp | null,
    transferType: number | null,
    readonly sourceAddress: string | null,
    readonly rawJson: string,
  ) {
    this.status = checkedUnsigned(status, U32_MAX, "status");
    this.transferType = checkedOptionalU32(transferType, "transferType");
    freezeRecord(this);
  }
}

export class BinanceDepositHistory {
  readonly entries: readonly BinanceDepositHistoryEntry[];
  constructor(entries: readonly BinanceDepositHistoryEntry[], readonly rawJson: string) {
    this.entries = Object.freeze([...entries]);
    freezeRecord(this);
  }
}

export class BinanceQuestionnaireRequirements {
  constructor(readonly questionnaireCountryCode: string, readonly rawJson: string) {
    freezeRecord(this);
  }
}

export class BinanceWithdrawalAddress {
  constructor(
    readonly address: string,
    readonly addressTag: string | null,
    readonly coin: string,
    readonly network: string,
    readonly whiteStatus: boolean,
    readonly name: string | null,
    readonly origin: string | null,
    readonly originType: string | null,
    readonly rawJson: string,
  ) { freezeRecord(this); }
}

export class BinanceWithdrawHistoryEntry {
  readonly status: number;
  readonly transferType: number | null;
  readonly confirmNo: number | null;
  readonly walletType: number | null;
  constructor(
    readonly id: string,
    readonly amount: Decimal,
    readonly transactionFee: Decimal,
    readonly coin: string,
    status: number,
    readonly address: string | null,
    readonly txId: string | null,
    readonly applyTime: string | null,
    readonly network: string | null,
    readonly withdrawOrderId: string | null,
    readonly info: string | null,
    transferType: number | null,
    confirmNo: number | null,
    walletType: number | null,
    readonly txKey: string | null,
    readonly completeTime: string | null,
    readonly rawJson: string,
  ) {
    this.status = checkedUnsigned(status, U32_MAX, "status");
    this.transferType = checkedOptionalU32(transferType, "transferType");
    this.confirmNo = checkedOptionalU32(confirmNo, "confirmNo");
    this.walletType = checkedOptionalU32(walletType, "walletType");
    freezeRecord(this);
  }
}

export class BinanceWithdrawHistory {
  readonly entries: readonly BinanceWithdrawHistoryEntry[];
  constructor(entries: readonly BinanceWithdrawHistoryEntry[], readonly rawJson: string) {
    this.entries = Object.freeze([...entries]);
    freezeRecord(this);
  }
}

export class BinanceMarkPrice {
  constructor(
    readonly market: Market, readonly markPrice: Decimal, readonly indexPrice: Decimal,
    readonly estimatedSettlePrice: Decimal | null, readonly lastFundingRate: Decimal,
    readonly interestRate: Decimal, readonly nextFundingTime: Timestamp, readonly time: Timestamp,
  ) { freezeRecord(this); }
}

export class BinanceOpenInterest {
  constructor(
    readonly market: Market, readonly openInterest: Decimal, readonly time: Timestamp,
  ) { freezeRecord(this); }
}

export class BinanceAggregateTradesRequest {
  readonly fromId: bigint | null;
  readonly limit: number | null;
  constructor(
    readonly market: Market,
    fromId: bigint | null = null,
    readonly startTime: Timestamp | null = null,
    readonly endTime: Timestamp | null = null,
    limit: number | null = null,
  ) {
    this.fromId = fromId === null ? null : checkedU64(fromId, "fromId");
    this.limit = checkedOptionalU32(limit, "limit");
    freezeRecord(this);
  }
}

export class BinanceAggregateTrade {
  readonly aggregateId: bigint;
  readonly firstTradeId: bigint;
  readonly lastTradeId: bigint;
  constructor(
    readonly market: Market,
    aggregateId: bigint,
    firstTradeId: bigint,
    lastTradeId: bigint,
    readonly timestamp: Timestamp,
    readonly price: Decimal,
    readonly quantity: Decimal,
    readonly normalQuantity: Decimal | null,
    readonly bestPriceMatch: boolean | null,
    readonly takerSide: Side,
    readonly rawJson: string,
  ) {
    this.aggregateId = checkedU64(aggregateId, "aggregateId");
    this.firstTradeId = checkedU64(firstTradeId, "firstTradeId");
    this.lastTradeId = checkedU64(lastTradeId, "lastTradeId");
    freezeRecord(this);
  }
}

export class BinanceAccountTrade {
  constructor(
    readonly market: Market,
    readonly id: string,
    readonly orderId: string,
    readonly timestamp: Timestamp,
    readonly side: Side,
    readonly maker: boolean,
    readonly bestMatch: boolean | null,
    readonly orderListId: string | null,
    readonly price: Decimal,
    readonly quantity: Decimal,
    readonly quoteQuantity: Decimal | null,
    readonly commission: Decimal,
    readonly commissionAsset: string,
    readonly realizedPnl: Decimal | null,
    readonly positionSide: string | null,
    readonly pair: string | null,
    readonly baseQuantity: Decimal | null,
    readonly marginAsset: string | null,
  ) { freezeRecord(this); }
}

export class BinanceTestOrderRequest {
  constructor(readonly order: OrderRequest, readonly computeCommissionRates: boolean = false) {
    freezeRecord(this);
  }
}

export class BinanceTestOrder {
  constructor(readonly responseJson: string) { freezeRecord(this); }
}

export class BinanceC2cTradeHistoryRequest {
  readonly page: number | null;
  readonly rows: number | null;
  readonly recvWindow: bigint | null;
  constructor(
    readonly tradeType: BinanceC2cTradeType,
    readonly startTimestamp: Timestamp | null = null,
    readonly endTimestamp: Timestamp | null = null,
    page: number | null = null,
    rows: number | null = null,
    recvWindow: bigint | null = null,
  ) {
    this.page = checkedOptionalU32(page, "page");
    this.rows = checkedOptionalU32(rows, "rows");
    this.recvWindow = recvWindow === null ? null : checkedU64(recvWindow, "recvWindow");
    freezeRecord(this);
  }
}

export class BinanceC2cTrade {
  readonly additionalKycVerify: number | null;
  constructor(
    readonly orderNumber: string | null,
    readonly advNo: string | null,
    readonly tradeType: string | null,
    readonly asset: string | null,
    readonly fiat: string | null,
    readonly fiatSymbol: string | null,
    readonly amount: Decimal | null,
    readonly totalPrice: Decimal | null,
    readonly unitPrice: Decimal | null,
    readonly orderStatus: string | null,
    readonly createdAt: Timestamp | null,
    readonly commission: Decimal | null,
    readonly counterpartyNickname: string | null,
    readonly payMethodName: string | null,
    additionalKycVerify: number | null,
    readonly takerCommissionRate: Decimal | null,
    readonly takerCommission: Decimal | null,
    readonly takerAmount: Decimal | null,
    readonly advertisementRole: string | null,
  ) {
    this.additionalKycVerify = checkedOptionalU32(additionalKycVerify, "additionalKycVerify");
    freezeRecord(this);
  }
}

export class BinanceC2cTradeHistoryPage {
  readonly data: readonly BinanceC2cTrade[] | null;
  readonly total: bigint | null;
  constructor(
    readonly code: string | null,
    readonly message: string | null,
    data: readonly BinanceC2cTrade[] | null,
    total: bigint | null,
    readonly success: boolean | null,
  ) {
    this.data = data === null ? null : Object.freeze([...data]);
    this.total = total === null ? null : checkedU64(total, "total");
    freezeRecord(this);
  }
}

export class HyperliquidLedgerEntry {
  constructor(
    readonly kind: HyperliquidLedgerKind, readonly time: Timestamp, readonly hash: string,
    readonly asset: string | null, readonly amount: Decimal | null, readonly fee: Decimal | null,
    readonly counterparty: string | null,
  ) { freezeRecord(this); }
}

export class HyperliquidMidPrice {
  constructor(readonly market: Market, readonly price: Decimal) { freezeRecord(this); }
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

export class HyperliquidCandleSnapshot {
  readonly tradeCount: bigint | null;
  constructor(
    readonly coin: string,
    readonly market: Market,
    readonly interval: string,
    readonly openTime: Timestamp,
    readonly closeTime: Timestamp,
    readonly open: Decimal,
    readonly high: Decimal,
    readonly low: Decimal,
    readonly close: Decimal,
    readonly volume: Decimal,
    tradeCount: bigint | null,
    readonly rawJson: string,
  ) {
    this.tradeCount = tradeCount === null ? null : checkedU64(tradeCount, "tradeCount");
    freezeRecord(this);
  }
}

export class HyperliquidBookLevel {
  readonly orderCount: bigint | null;
  constructor(readonly price: Decimal, readonly size: Decimal, orderCount: bigint | null) {
    this.orderCount = orderCount === null ? null : checkedU64(orderCount, "orderCount");
    freezeRecord(this);
  }
}

export class HyperliquidL2Book {
  readonly bids: readonly HyperliquidBookLevel[];
  readonly asks: readonly HyperliquidBookLevel[];
  constructor(
    readonly coin: string,
    readonly market: Market,
    readonly time: Timestamp,
    bids: readonly HyperliquidBookLevel[],
    asks: readonly HyperliquidBookLevel[],
    readonly rawJson: string,
  ) {
    this.bids = Object.freeze([...bids]);
    this.asks = Object.freeze([...asks]);
    freezeRecord(this);
  }
}

export class HyperliquidRecentTrade {
  readonly users: readonly string[];
  constructor(
    readonly coin: string,
    readonly market: Market,
    readonly side: string,
    readonly price: Decimal,
    readonly size: Decimal,
    readonly time: Timestamp,
    readonly tradeId: string,
    readonly hash: string | null,
    users: readonly string[],
    readonly rawJson: string,
  ) {
    this.users = Object.freeze([...users]);
    freezeRecord(this);
  }
}

/** A Hyperliquid trade update with its portable projection and native fields. */
export class HyperliquidTradeEvent {
  constructor(readonly common: Trade, readonly provider: HyperliquidRecentTrade) {
    freezeRecord(this);
  }
}

/** A Hyperliquid L2-book update with per-level native order counts. */
export class HyperliquidOrderBookEvent {
  constructor(readonly common: OrderBook, readonly provider: HyperliquidL2Book) {
    freezeRecord(this);
  }
}

/** A Hyperliquid candle update with its native trade count. */
export class HyperliquidCandleEvent {
  constructor(readonly common: Candle, readonly provider: HyperliquidCandleSnapshot) {
    freezeRecord(this);
  }
}

/** A Hyperliquid active-asset context update. */
export class HyperliquidAssetContextEvent {
  constructor(
    readonly common: Ticker,
    readonly coin: string,
    readonly midPrice: Decimal | null,
    readonly markPrice: Decimal | null,
    readonly previousDayPrice: Decimal | null,
    readonly dayBaseVolume: Decimal | null,
    readonly dayNotionalVolume: Decimal | null,
    readonly oraclePrice: Decimal | null,
    readonly fundingRate: Decimal | null,
    readonly openInterest: Decimal | null,
    readonly circulatingSupply: Decimal | null,
    readonly totalSupply: Decimal | null,
    readonly rawJson: string,
  ) { freezeRecord(this); }
}

/** A Hyperliquid order-status update with provider lifecycle fields. */
export class HyperliquidOrderUpdate {
  readonly orderId: bigint;
  constructor(
    readonly common: Order,
    readonly coin: string,
    readonly side: string,
    readonly limitPrice: Decimal,
    readonly remainingSize: Decimal,
    readonly originalSize: Decimal,
    orderId: bigint,
    readonly acceptedAt: Timestamp,
    readonly clientOrderId: string | null,
    readonly status: string,
    readonly statusAt: Timestamp | null,
    readonly rawJson: string,
  ) {
    this.orderId = checkedU64(orderId, "orderId");
    freezeRecord(this);
  }
}

export class HyperliquidFundingHistoryEntry {
  constructor(
    readonly coin: string,
    readonly market: Market,
    readonly fundingRate: Decimal,
    readonly premium: Decimal | null,
    readonly time: Timestamp,
    readonly rawJson: string,
  ) { freezeRecord(this); }
}

export class HyperliquidUserFunding {
  readonly sampleCount: bigint | null;
  constructor(
    readonly kind: string | null,
    readonly coin: string,
    readonly market: Market,
    readonly usdc: Decimal,
    readonly fundingRate: Decimal,
    readonly positionSize: Decimal | null,
    sampleCount: bigint | null,
    readonly hash: string,
    readonly time: Timestamp,
    readonly rawJson: string,
  ) {
    this.sampleCount = sampleCount === null ? null : checkedU64(sampleCount, "sampleCount");
    freezeRecord(this);
  }
}

export class HyperliquidSpotBalance {
  readonly token: number | null;
  constructor(
    readonly coin: string,
    token: number | null,
    readonly total: Decimal,
    readonly hold: Decimal,
    readonly entryNotional: Decimal | null,
    readonly rawJson: string,
  ) {
    this.token = token === null ? null : checkedUnsigned(token, U32_MAX, "token");
    freezeRecord(this);
  }
}

/** One balance from a Hyperliquid `spotState` update. */
export class HyperliquidSpotStateBalance {
  constructor(readonly common: Balance, readonly provider: HyperliquidSpotBalance) {
    freezeRecord(this);
  }
}

/** A Hyperliquid `spotState` account update. */
export class HyperliquidSpotStateEvent {
  readonly balances: readonly HyperliquidSpotStateBalance[];
  constructor(
    readonly user: string,
    balances: readonly HyperliquidSpotStateBalance[],
    readonly rawJson: string,
  ) {
    this.balances = Object.freeze([...balances]);
    freezeRecord(this);
  }
}

export type HyperliquidMarketEvent =
  | { readonly kind: "trade"; readonly value: HyperliquidTradeEvent }
  | { readonly kind: "order_book"; readonly value: HyperliquidOrderBookEvent }
  | { readonly kind: "asset_context"; readonly value: HyperliquidAssetContextEvent }
  | { readonly kind: "candle"; readonly value: HyperliquidCandleEvent }
  | { readonly kind: "reconnected" };

export type HyperliquidAccountEvent =
  | { readonly kind: "order_update"; readonly value: HyperliquidOrderUpdate }
  | { readonly kind: "spot_state"; readonly value: HyperliquidSpotStateEvent }
  | { readonly kind: "reconnected" };

export class HyperliquidSpotClearinghouseState {
  readonly balances: readonly HyperliquidSpotBalance[];
  constructor(balances: readonly HyperliquidSpotBalance[], readonly rawJson: string) {
    this.balances = Object.freeze([...balances]);
    freezeRecord(this);
  }
}

export class HyperliquidEvmContract {
  readonly extraWeiDecimals: number;
  constructor(readonly address: string, extraWeiDecimals: number) {
    this.extraWeiDecimals = checkedUnsigned(extraWeiDecimals, U32_MAX, "extraWeiDecimals");
    freezeRecord(this);
  }
}

export class HyperliquidSpotToken {
  readonly sizeDecimals: number;
  readonly weiDecimals: number | null;
  readonly index: number;
  constructor(
    readonly name: string,
    sizeDecimals: number,
    weiDecimals: number | null,
    index: number,
    readonly tokenId: string | null,
    readonly isCanonical: boolean | null,
    readonly evmContract: HyperliquidEvmContract | null,
    readonly fullName: string | null,
    readonly deployerTradingFeeShare: Decimal | null,
    readonly rawJson: string,
  ) {
    this.sizeDecimals = checkedUnsigned(sizeDecimals, U32_MAX, "sizeDecimals");
    this.weiDecimals = weiDecimals === null ? null : checkedUnsigned(weiDecimals, U32_MAX, "weiDecimals");
    this.index = checkedUnsigned(index, U32_MAX, "index");
    freezeRecord(this);
  }
}

export class HyperliquidSpotPair {
  readonly tokens: readonly number[];
  readonly index: number;
  constructor(
    readonly name: string,
    tokens: readonly number[],
    index: number,
    readonly isCanonical: boolean | null,
    readonly rawJson: string,
  ) {
    this.tokens = Object.freeze(tokens.map((token) => checkedUnsigned(token, U32_MAX, "tokens")));
    this.index = checkedUnsigned(index, U32_MAX, "index");
    freezeRecord(this);
  }
}

export class HyperliquidSpotMeta {
  readonly tokens: readonly HyperliquidSpotToken[];
  readonly universe: readonly HyperliquidSpotPair[];
  constructor(
    tokens: readonly HyperliquidSpotToken[],
    universe: readonly HyperliquidSpotPair[],
    readonly rawJson: string,
  ) {
    this.tokens = Object.freeze([...tokens]);
    this.universe = Object.freeze([...universe]);
    freezeRecord(this);
  }
}

export class HyperliquidSpotAssetContext {
  constructor(
    readonly coin: string | null,
    readonly midPrice: Decimal | null,
    readonly markPrice: Decimal | null,
    readonly previousDayPrice: Decimal | null,
    readonly dayBaseVolume: Decimal | null,
    readonly dayNotionalVolume: Decimal | null,
    readonly circulatingSupply: Decimal | null,
    readonly totalSupply: Decimal | null,
    readonly rawJson: string,
  ) { freezeRecord(this); }
}

export class HyperliquidSpotMetaAndAssetContexts {
  readonly contexts: readonly HyperliquidSpotAssetContext[];
  constructor(
    readonly meta: HyperliquidSpotMeta,
    contexts: readonly HyperliquidSpotAssetContext[],
    readonly rawJson: string,
  ) {
    this.contexts = Object.freeze([...contexts]);
    freezeRecord(this);
  }
}

export class HyperliquidUserRateLimit {
  readonly requestsUsed: bigint;
  readonly requestsCap: bigint;
  readonly requestsSurplus: bigint;
  constructor(
    readonly cumulativeVolume: Decimal,
    requestsUsed: bigint,
    requestsCap: bigint,
    requestsSurplus: bigint,
  ) {
    this.requestsUsed = checkedU64(requestsUsed, "requestsUsed");
    this.requestsCap = checkedU64(requestsCap, "requestsCap");
    this.requestsSurplus = checkedU64(requestsSurplus, "requestsSurplus");
    freezeRecord(this);
  }
}

export type HyperliquidUserRole =
  | { readonly kind: "user" }
  | { readonly kind: "agent"; readonly user: string | null }
  | { readonly kind: "vault" }
  | { readonly kind: "sub_account"; readonly master: string | null }
  | { readonly kind: "missing" }
  | { readonly kind: "other"; readonly role: string; readonly dataJson: string | null };

export class HyperliquidReferrer {
  constructor(readonly address: string, readonly code: string) { freezeRecord(this); }
}

export class HyperliquidReferral {
  constructor(
    readonly referredBy: HyperliquidReferrer | null,
    readonly cumulativeVolume: Decimal,
    readonly unclaimedRewards: Decimal,
    readonly claimedRewards: Decimal,
    readonly builderRewards: Decimal,
    readonly referrerStateJson: string,
    readonly rewardHistoryJson: string,
    readonly tokenToStateJson: string,
  ) { freezeRecord(this); }
}

export class HyperliquidDailyVolume {
  constructor(
    readonly date: string,
    readonly userCross: Decimal,
    readonly userAdd: Decimal,
    readonly exchange: Decimal,
  ) { freezeRecord(this); }
}

export class HyperliquidUserFees {
  readonly dailyVolumes: readonly HyperliquidDailyVolume[];
  constructor(
    dailyVolumes: readonly HyperliquidDailyVolume[],
    readonly feeScheduleJson: string,
    readonly userCrossRate: Decimal,
    readonly userAddRate: Decimal,
    readonly userSpotCrossRate: Decimal | null,
    readonly userSpotAddRate: Decimal | null,
    readonly activeReferralDiscount: Decimal | null,
    readonly detailsJson: string,
  ) {
    this.dailyVolumes = Object.freeze([...dailyVolumes]);
    freezeRecord(this);
  }
}

export class HyperliquidUserFill {
  readonly orderId: bigint;
  readonly tradeId: bigint;
  readonly twapId: bigint | null;
  constructor(
    readonly coin: string,
    readonly price: Decimal,
    readonly size: Decimal,
    readonly side: string,
    readonly time: Timestamp,
    readonly startPosition: Decimal,
    readonly direction: string,
    readonly closedPnl: Decimal,
    readonly hash: string,
    orderId: bigint,
    readonly crossed: boolean,
    readonly fee: Decimal,
    readonly builderFee: Decimal | null,
    tradeId: bigint,
    readonly feeToken: string,
    twapId: bigint | null,
    readonly rawJson: string,
  ) {
    this.orderId = checkedU64(orderId, "orderId");
    this.tradeId = checkedU64(tradeId, "tradeId");
    this.twapId = twapId === null ? null : checkedU64(twapId, "twapId");
    freezeRecord(this);
  }
}

export type HyperliquidOrderReference =
  | { readonly kind: "order_id"; readonly value: bigint }
  | { readonly kind: "client_order_id"; readonly value: string };

export class HyperliquidOpenOrder {
  readonly orderId: bigint;
  constructor(
    readonly coin: string,
    readonly limitPrice: Decimal,
    orderId: bigint,
    readonly side: string,
    readonly size: Decimal,
    readonly timestamp: Timestamp,
    readonly rawJson: string,
  ) {
    this.orderId = checkedU64(orderId, "orderId");
    freezeRecord(this);
  }
}

export class HyperliquidOrderDetail {
  readonly orderId: bigint;
  constructor(
    readonly coin: string,
    readonly side: string,
    readonly limitPrice: Decimal,
    readonly size: Decimal,
    orderId: bigint,
    readonly timestamp: Timestamp,
    readonly triggerCondition: string,
    readonly isTrigger: boolean,
    readonly triggerPrice: Decimal,
    readonly childrenJson: string,
    readonly isPositionTpsl: boolean,
    readonly reduceOnly: boolean,
    readonly orderType: string,
    readonly originalSize: Decimal,
    readonly timeInForce: string | null,
    readonly clientOrderId: string | null,
    readonly rawJson: string,
  ) {
    this.orderId = checkedU64(orderId, "orderId");
    freezeRecord(this);
  }
}

export class HyperliquidOrderInfo {
  constructor(
    readonly order: HyperliquidOrderDetail,
    readonly status: string,
    readonly statusTimestamp: Timestamp,
    readonly rawJson: string,
  ) { freezeRecord(this); }
}

export type HyperliquidOrderStatusResponse =
  | { readonly kind: "order"; readonly value: HyperliquidOrderInfo }
  | { readonly kind: "unknown_order" }
  | { readonly kind: "other"; readonly status: string; readonly rawJson: string };

export class HyperliquidPortfolioPoint {
  constructor(readonly time: Timestamp, readonly value: Decimal) { freezeRecord(this); }
}

export class HyperliquidPortfolioPeriod {
  readonly accountValueHistory: readonly HyperliquidPortfolioPoint[];
  readonly pnlHistory: readonly HyperliquidPortfolioPoint[];
  constructor(
    readonly period: string,
    accountValueHistory: readonly HyperliquidPortfolioPoint[],
    pnlHistory: readonly HyperliquidPortfolioPoint[],
    readonly volume: Decimal,
  ) {
    this.accountValueHistory = Object.freeze([...accountValueHistory]);
    this.pnlHistory = Object.freeze([...pnlHistory]);
    freezeRecord(this);
  }
}

export class HyperliquidSubAccount {
  constructor(
    readonly name: string,
    readonly user: string,
    readonly master: string,
    readonly perpetualStateJson: string,
    readonly spotStateJson: string,
  ) { freezeRecord(this); }
}

export class HyperliquidVaultEquity {
  constructor(
    readonly vaultAddress: string,
    readonly equity: Decimal,
    readonly lockedUntil: Timestamp | null,
  ) { freezeRecord(this); }
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
      // ponytail: subscription lists are small; use a structural-key Map if scale becomes a problem.
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
