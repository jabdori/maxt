import { Exchange, Feature } from "./models.js";

export type ErrorWire =
  | { readonly kind: "invalid_request"; readonly field: string; readonly detail: string }
  | { readonly kind: "unsupported"; readonly feature: string; readonly exchange: string; readonly detail: string }
  | { readonly kind: "adapter"; readonly detail: string }
  | { readonly kind: "auth"; readonly detail: string }
  | {
    readonly kind: "exchange";
    readonly exchange: string;
    readonly code: string;
    readonly message: string;
    readonly status: number | null;
    readonly exchange_kind: string;
  }
  | { readonly kind: "transport"; readonly detail: string }
  | { readonly kind: "decode"; readonly detail: string };

export class ExchangeErrorKind {
  static readonly Rejected = new ExchangeErrorKind("rejected", false);
  static readonly RateLimited = new ExchangeErrorKind("rate_limited", true);
  static readonly Unavailable = new ExchangeErrorKind("unavailable", true);
  static readonly Unknown = new ExchangeErrorKind("unknown", false);
  static readonly values: readonly ExchangeErrorKind[] = Object.freeze([
    ExchangeErrorKind.Rejected,
    ExchangeErrorKind.RateLimited,
    ExchangeErrorKind.Unavailable,
    ExchangeErrorKind.Unknown,
  ]);

  private constructor(readonly id: string, private readonly retryable: boolean) {
    Object.freeze(this);
  }

  isRetryable(): boolean {
    return this.retryable;
  }

  toString(): string {
    return this.id;
  }
}

export abstract class MaxtError extends Error {
  abstract readonly kind: ErrorWire["kind"];

  protected constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = new.target.name;
  }

  isRetryable(): boolean {
    return false;
  }

  isRateLimited(): boolean {
    return false;
  }
}

export class InvalidRequestError extends MaxtError {
  readonly kind = "invalid_request";

  constructor(readonly field: string, readonly detail: string) {
    super(`invalid request: \`${field}\`: ${detail}`);
  }
}

export class UnsupportedError extends MaxtError {
  readonly kind = "unsupported";

  constructor(readonly feature: Feature, readonly exchange: Exchange, readonly detail: string) {
    super(`${exchange.id} adapter does not support ${feature.id}: ${detail}`);
  }
}

export class AdapterError extends MaxtError {
  readonly kind = "adapter";

  constructor(readonly detail: string, options?: ErrorOptions) {
    super(`adapter failed: ${detail}`, options);
  }
}

export class AuthError extends MaxtError {
  readonly kind = "auth";

  constructor(readonly detail: string) {
    super(`authentication failed: ${detail}`);
  }
}

export class ExchangeError extends MaxtError {
  readonly kind = "exchange";

  constructor(
    readonly exchange: Exchange,
    readonly code: string,
    readonly providerMessage: string,
    readonly status: number | null,
    readonly exchangeKind: ExchangeErrorKind,
  ) {
    super(status === null
      ? `${exchange.id} returned ${code}: ${providerMessage}`
      : `${exchange.id} returned ${status} ${code}: ${providerMessage}`);
  }

  override isRetryable(): boolean {
    return this.exchangeKind.isRetryable();
  }

  override isRateLimited(): boolean {
    return this.exchangeKind === ExchangeErrorKind.RateLimited;
  }
}

export class TransportError extends MaxtError {
  readonly kind = "transport";

  constructor(readonly detail: string) {
    super(`transport failed: ${detail}`);
  }

  override isRetryable(): boolean {
    return true;
  }
}

export class DecodeError extends MaxtError {
  readonly kind = "decode";

  constructor(readonly detail: string) {
    super(`could not read exchange response: ${detail}`);
  }
}

function valueById<T extends { readonly id: string }>(values: readonly T[], id: string, field: string): T {
  const value = values.find((candidate) => candidate.id === id);
  if (value === undefined) throw new AdapterError(`unknown ${field}: ${id}`);
  return value;
}

export function errorFromWire(wire: ErrorWire): MaxtError {
  switch (wire.kind) {
    case "invalid_request":
      return new InvalidRequestError(wire.field, wire.detail);
    case "unsupported":
      return new UnsupportedError(
        valueById(Feature.values, wire.feature, "feature"),
        valueById(Exchange.values, wire.exchange, "exchange"),
        wire.detail,
      );
    case "adapter":
      return new AdapterError(wire.detail);
    case "auth":
      return new AuthError(wire.detail);
    case "exchange":
      return new ExchangeError(
        valueById(Exchange.values, wire.exchange, "exchange"),
        wire.code,
        wire.message,
        wire.status,
        valueById(ExchangeErrorKind.values, wire.exchange_kind, "exchange error kind"),
      );
    case "transport":
      return new TransportError(wire.detail);
    case "decode":
      return new DecodeError(wire.detail);
    default:
      return assertNever(wire);
  }
}

export function errorToWire(error: unknown): ErrorWire {
  if (!(error instanceof MaxtError)) {
    return { kind: "adapter", detail: adapterFailureDetail(error) };
  }
  switch (error.kind) {
    case "invalid_request": {
      const value = error as InvalidRequestError;
      return { kind: value.kind, field: value.field, detail: value.detail };
    }
    case "unsupported": {
      const value = error as UnsupportedError;
      return {
        kind: value.kind,
        feature: value.feature.id,
        exchange: value.exchange.id,
        detail: value.detail,
      };
    }
    case "adapter":
    case "auth":
    case "transport":
    case "decode":
      return { kind: error.kind, detail: (error as AdapterError | AuthError | TransportError | DecodeError).detail };
    case "exchange": {
      const value = error as ExchangeError;
      return {
        kind: value.kind,
        exchange: value.exchange.id,
        code: value.code,
        message: value.providerMessage,
        status: value.status,
        exchange_kind: value.exchangeKind.id,
      };
    }
    default:
      return assertNever(error.kind);
  }
}

function adapterFailureDetail(error: unknown): string {
  if (typeof error === "object" && error !== null && "stack" in error) {
    const stack = (error as { readonly stack?: unknown }).stack;
    if (typeof stack === "string") return stack;
  }
  return String(error);
}

function assertNever(value: never): never {
  throw new AdapterError(`unknown structured error variant: ${String(value)}`);
}
