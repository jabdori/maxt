import { Exchange, Feature } from "./models.js";
import { ExchangeErrorKind, TransferErrorKind } from "./generated/identifiers.js";
import type { ErrorWire } from "./generated/contract.js";
export type { ErrorWire } from "./generated/contract.js";
export { ExchangeErrorKind, TransferErrorKind } from "./generated/identifiers.js";

export abstract class MaxtError extends Error {
  abstract readonly kind: ErrorWire["kind"];

  protected constructor(message: string, options?: ErrorOptions) {
    super(message, options);
    this.name = "MaxtError";
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
    this.name = "InvalidRequestError";
  }
}

export class TransferError extends MaxtError {
  readonly kind = "transfer";

  constructor(readonly transferKind: TransferErrorKind, readonly detail: string) {
    super(`transfer rejected (${transferKind.id}): ${detail}`);
    this.name = "TransferError";
  }
}

export class UnsupportedError extends MaxtError {
  readonly kind = "unsupported";

  constructor(readonly feature: Feature, readonly exchange: Exchange, readonly detail: string) {
    super(`${exchange.id} adapter does not support ${feature.id}: ${detail}`);
    this.name = "UnsupportedError";
  }
}

export class AdapterError extends MaxtError {
  readonly kind = "adapter";

  constructor(readonly detail: string, options?: ErrorOptions) {
    super(`adapter failed: ${detail}`, options);
    this.name = "AdapterError";
  }
}

export class AuthError extends MaxtError {
  readonly kind = "auth";

  constructor(readonly detail: string) {
    super(`authentication failed: ${detail}`);
    this.name = "AuthError";
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
    this.name = "ExchangeError";
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
    this.name = "TransportError";
  }

  override isRetryable(): boolean {
    return true;
  }
}

export class DecodeError extends MaxtError {
  readonly kind = "decode";

  constructor(readonly detail: string) {
    super(`could not read exchange response: ${detail}`);
    this.name = "DecodeError";
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
    case "transfer":
      return new TransferError(
        valueById(TransferErrorKind.values, wire.transfer_kind, "transfer error kind"),
        wire.detail,
      );
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
  try {
    return errorToWireUnsafe(error);
  } catch {
    return unreadableAdapterFailure();
  }
}

function errorToWireUnsafe(error: unknown): ErrorWire {
  if (error instanceof InvalidRequestError) {
    return { kind: "invalid_request", field: error.field, detail: error.detail };
  }
  if (error instanceof TransferError) {
    return { kind: "transfer", transfer_kind: error.transferKind.id, detail: error.detail };
  }
  if (error instanceof UnsupportedError) {
    return {
      kind: "unsupported",
      feature: error.feature.id,
      exchange: error.exchange.id,
      detail: error.detail,
    };
  }
  if (error instanceof AdapterError) return { kind: "adapter", detail: error.detail };
  if (error instanceof AuthError) return { kind: "auth", detail: error.detail };
  if (error instanceof ExchangeError) {
    return {
      kind: "exchange",
      exchange: error.exchange.id,
      code: error.code,
      message: error.providerMessage,
      status: error.status,
      exchange_kind: error.exchangeKind.id,
    };
  }
  if (error instanceof TransportError) return { kind: "transport", detail: error.detail };
  if (error instanceof DecodeError) return { kind: "decode", detail: error.detail };
  return { kind: "adapter", detail: adapterFailureDetail(error) };
}

function adapterFailureDetail(error: unknown): string {
  if (typeof error === "object" && error !== null && "stack" in error) {
    const stack = (error as { readonly stack?: unknown }).stack;
    if (typeof stack === "string") return stack;
  }
  return String(error);
}

function unreadableAdapterFailure(): ErrorWire {
  return { kind: "adapter", detail: "JavaScript adapter threw an unreadable value" };
}

function assertNever(value: never): never {
  throw new AdapterError(`unknown structured error variant: ${String(value)}`);
}
