import assert from "node:assert/strict";
import test from "node:test";

import {
  ADAPTER_OPERATIONS,
  CLIENT_MEMBERS,
  ERROR_VARIANTS,
  EXCHANGES,
  FEATURES,
  PROVIDER_METHODS,
} from "../dist/generated/contract.js";
import {
  Adapter,
  BinanceAdapter,
  BithumbAdapter,
  Client,
  HyperliquidAdapter,
  UpbitAdapter,
} from "../dist/generated/api.js";
import {
  AdapterError,
  AuthError,
  DecodeError,
  ExchangeError,
  InvalidRequestError,
  TransferError,
  TransportError,
  UnsupportedError,
  errorFromWire,
} from "../dist/errors.js";
import { Exchange, Feature, TransferErrorKind } from "../dist/models.js";

test("generated exchange and feature inventories match the public models", () => {
  assert.deepEqual(EXCHANGES, Exchange.values.map((value) => value.id));
  assert.deepEqual(FEATURES, Feature.values.map((value) => value.id));
});

test("generated public API inventories stay explicit", () => {
  const prototypeMembers = (value) => new Set(
    Object.getOwnPropertyNames(value.prototype).filter((name) => name !== "constructor"),
  );
  const adapterMembers = prototypeMembers(Adapter);
  adapterMembers.delete("supports");
  assert.deepEqual(adapterMembers, new Set(ADAPTER_OPERATIONS));

  const clientMembers = prototypeMembers(Client);
  clientMembers.add("adapter");
  assert.deepEqual(clientMembers, new Set(CLIENT_MEMBERS));

  const providers = {
    upbit: UpbitAdapter,
    bithumb: BithumbAdapter,
    binance: BinanceAdapter,
    hyperliquid: HyperliquidAdapter,
  };
  for (const [exchange, adapter] of Object.entries(providers)) {
    assert.deepEqual(prototypeMembers(adapter), new Set(PROVIDER_METHODS[exchange]));
  }

  const errorClasses = [
    InvalidRequestError,
    TransferError,
    UnsupportedError,
    AdapterError,
    AuthError,
    ExchangeError,
    TransportError,
    DecodeError,
  ];
  assert.deepEqual(
    errorClasses.map((error) => error.name.replace(/Error$/, "")),
    ERROR_VARIANTS,
  );

  const wires = [
    { kind: "invalid_request", field: "limit", detail: "invalid" },
    { kind: "transfer", transfer_kind: TransferErrorKind.NetworkMismatch.id, detail: "mismatch" },
    { kind: "unsupported", feature: Feature.Markets.id, exchange: Exchange.Upbit.id, detail: "unsupported" },
    { kind: "adapter", detail: "adapter" },
    { kind: "auth", detail: "auth" },
    { kind: "exchange", exchange: Exchange.Upbit.id, code: "E", message: "exchange", status: null, exchange_kind: "unknown" },
    { kind: "transport", detail: "transport" },
    { kind: "decode", detail: "decode" },
  ];
  wires.forEach((wire, index) => assert(errorFromWire(wire) instanceof errorClasses[index]));
});
