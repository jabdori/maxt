import assert from "node:assert/strict";
import { createRequire } from "node:module";
import test from "node:test";

import * as maxt from "../dist/node.js";
import {
  Adapter,
  BinanceAdapter,
  BinanceMarket,
  BithumbAdapter,
  Client,
  Decimal,
  Exchange,
  Feature,
  InvalidRequestError,
  Market,
  Ticker,
  Timestamp,
} from "../dist/node.js";
import {
  RAW_NATIVE_CLIENT_MEMBERS,
  RAW_NATIVE_EXPORTS,
  RAW_PROVIDER_MEMBERS,
  PROVIDER_CONSTRUCTORS,
} from "../dist/generated/contract.js";

const raw = createRequire(import.meta.url)("../native.cjs");

test("generated inventories match the loaded native module exactly", () => {
  assert.deepEqual(Object.keys(raw).sort(), [...RAW_NATIVE_EXPORTS].sort());
  assert.deepEqual(
    Object.getOwnPropertyNames(raw.NativeClient.prototype)
      .filter((name) => name !== "constructor")
      .sort(),
    [...RAW_NATIVE_CLIENT_MEMBERS].sort(),
  );
  for (const [exchange, constructor] of Object.entries({
    upbit: raw.NativeUpbit,
    bithumb: raw.NativeBithumb,
    binance: raw.NativeBinance,
    hyperliquid: raw.NativeHyperliquid,
  })) {
    assert.deepEqual(
      Object.getOwnPropertyNames(constructor.prototype)
        .filter((name) => name !== "constructor")
        .sort(),
      [...RAW_PROVIDER_MEMBERS[exchange]].sort(),
    );
  }
});

test("every generated provider adapter is exported publicly", () => {
  assert.deepEqual(
    Object.keys(maxt).filter((name) => name === "Adapter" || name.endsWith("Adapter")).sort(),
    ["Adapter", "BinanceAdapter", "BithumbAdapter", "HyperliquidAdapter", "UpbitAdapter"],
  );

  for (const [exchange, constructor] of Object.entries({
    upbit: maxt.UpbitAdapter,
    bithumb: maxt.BithumbAdapter,
    binance: maxt.BinanceAdapter,
    hyperliquid: maxt.HyperliquidAdapter,
  })) {
    assert.deepEqual(
      Object.getOwnPropertyNames(constructor)
        .filter((name) => !["length", "name", "prototype"].includes(name))
        .sort(),
      PROVIDER_CONSTRUCTORS[exchange]
        .filter((name) => name !== "constructor")
        .sort(),
    );
  }
});

test("built-in adapters load through the generated Node backend", () => {
  const spot = BinanceAdapter.spot();
  assert.equal(spot.exchange, Exchange.Binance);
  assert.equal(spot.venue, BinanceMarket.Spot);

  const futures = BinanceAdapter.usdMFutures();
  assert.equal(futures.venue, BinanceMarket.UsdMFutures);

  assert.throws(
    () => new BithumbAdapter({ accessKey: "incomplete" }),
    InvalidRequestError,
  );
});

test("custom Adapter calls round-trip through Rust without losing values", async () => {
  const market = Market.spot(Exchange.Binance, "BTC", "USDT");
  const expected = new Ticker(
    market,
    Timestamp.fromNanoseconds(123n),
    null,
    Decimal.parse("100.25"),
    null,
    null,
    null,
    null,
    Decimal.parse("2.5"),
    null,
  );

  class FixtureAdapter extends Adapter {
    exchange = Exchange.Binance;
    features = new Set([Feature.Ticker]);
    async ticker(requested) {
      assert.equal(requested.toString(), market.toString());
      return expected;
    }
  }

  const actual = await new Client(new FixtureAdapter()).ticker(market);
  assert.equal(actual.lastPrice.toString(), "100.25");
  assert.equal(actual.timestamp.nanosecondsSinceEpoch, 123n);
  assert.equal(actual.volume?.toString(), "2.5");
});
