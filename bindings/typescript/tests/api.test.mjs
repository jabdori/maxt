import assert from "node:assert/strict";
import test from "node:test";
import {
  AdapterError,
  AuthError,
  DecodeError,
  ExchangeError,
  ExchangeErrorKind,
  InvalidRequestError,
  MaxtError,
  TransportError,
  UnsupportedError,
  errorFromWire,
  errorToWire,
} from "../dist/errors.js";
import { Exchange, Feature } from "../dist/models.js";

test("exchange errors preserve every provider field", () => {
  const error = errorFromWire({
    kind: "exchange",
    exchange: "binance",
    code: "-1003",
    message: "too many requests",
    status: 429,
    exchange_kind: "rate_limited",
  });

  assert.equal(error.code, "-1003");
  assert.equal(error.status, 429);
  assert.equal(error.isRetryable(), true);
  assert.equal(error.isRateLimited(), true);
  assert.equal(error.exchangeKind.isRetryable(), true);
  assert.deepEqual(errorToWire(error), {
    kind: "exchange",
    exchange: "binance",
    code: "-1003",
    message: "too many requests",
    status: 429,
    exchange_kind: "rate_limited",
  });
});

test("all seven structured errors round-trip without losing fields", () => {
  const wires = [
    { kind: "invalid_request", field: "limit", detail: "must be positive" },
    { kind: "unsupported", feature: "candles", exchange: "upbit", detail: "not mapped" },
    { kind: "adapter", detail: "contract failed" },
    { kind: "auth", detail: "missing secret" },
    {
      kind: "exchange",
      exchange: "bithumb",
      code: "server_error",
      message: "try later",
      status: 503,
      exchange_kind: "unavailable",
    },
    { kind: "transport", detail: "socket closed" },
    { kind: "decode", detail: "unexpected null" },
  ];

  const errors = wires.map(errorFromWire);
  assert.deepEqual(errors.map(errorToWire), wires);
  assert.deepEqual(errors.map((error) => error.name), [
    "InvalidRequestError",
    "UnsupportedError",
    "AdapterError",
    "AuthError",
    "ExchangeError",
    "TransportError",
    "DecodeError",
  ]);
  assert.deepEqual(errors.map((error) => error.isRetryable()), [false, false, false, false, true, true, false]);
  assert.deepEqual(errors.map((error) => error.isRateLimited()), [false, false, false, false, false, false, false]);
});

test("arbitrary JavaScript adapter exceptions preserve their stack", () => {
  const thrown = new Error("boom");
  thrown.stack = "Error: boom\n    at adapter.js:42:7";

  assert.deepEqual(errorToWire(thrown), {
    kind: "adapter",
    detail: thrown.stack,
  });
  assert.deepEqual(errorToWire("plain rejection"), {
    kind: "adapter",
    detail: "plain rejection",
  });
  assert.deepEqual(errorToWire({ stack: "custom adapter stack" }), {
    kind: "adapter",
    detail: "custom adapter stack",
  });
});

test("hostile adapter failures always become a safe adapter wire error", () => {
  const throwingStack = Object.defineProperty({}, "stack", {
    get() { throw new Error("stack getter failed"); },
  });
  const throwingProxy = new Proxy({}, {
    getPrototypeOf() { throw new Error("prototype lookup failed"); },
  });
  const throwingKnownError = new Proxy(new InvalidRequestError("field", "detail"), {
    get() { throw new Error("known error field lookup failed"); },
  });
  const nullPrototype = Object.create(null);
  class CustomMaxtError extends MaxtError {
    get kind() { throw new Error("kind getter failed"); }
  }

  for (const failure of [
    throwingStack,
    throwingProxy,
    throwingKnownError,
    nullPrototype,
    new CustomMaxtError("custom failure"),
  ]) {
    assert.doesNotThrow(() => errorToWire(failure));
    assert.equal(errorToWire(failure).kind, "adapter");
  }
  assert.deepEqual(errorToWire(throwingProxy), {
    kind: "adapter",
    detail: "JavaScript adapter threw an unreadable value",
  });
  assert.deepEqual(errorToWire(nullPrototype), {
    kind: "adapter",
    detail: "JavaScript adapter threw an unreadable value",
  });
  assert.deepEqual(errorToWire(throwingKnownError), {
    kind: "adapter",
    detail: "JavaScript adapter threw an unreadable value",
  });
});

test("public error names stay stable through derived constructor names", () => {
  const errors = [
    new (class A extends InvalidRequestError {})("field", "detail"),
    new (class B extends UnsupportedError {})(Feature.Candles, Exchange.Upbit, "detail"),
    new (class C extends AdapterError {})("detail"),
    new (class D extends AuthError {})("detail"),
    new (class E extends ExchangeError {})(
      Exchange.Binance,
      "code",
      "message",
      null,
      ExchangeErrorKind.Unknown,
    ),
    new (class F extends TransportError {})("detail"),
    new (class G extends DecodeError {})("detail"),
  ];

  assert.deepEqual(errors.map((error) => error.name), [
    "InvalidRequestError",
    "UnsupportedError",
    "AdapterError",
    "AuthError",
    "ExchangeError",
    "TransportError",
    "DecodeError",
  ]);
});
