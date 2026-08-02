import assert from "node:assert/strict";
import test from "node:test";

import { createBrowserBackend } from "../dist/browser-backend.js";
import { NATIVE_API_VERSION } from "../dist/generated/api.js";

const options = (overrides = {}) => ({
  wasmUrl: null,
  allowInsecureBrowserCredentials: false,
  relayUrl: null,
  ...overrides,
});

function rawClient(exchange = "binance") {
  return {
    exchange: () => exchange,
    supports: () => true,
  };
}

function rawModule(overrides = {}) {
  return {
    NATIVE_API_VERSION,
    default: async () => {},
    createCustomClient: () => rawClient(),
    createUpbit: () => ({ client: () => rawClient("upbit"), region: () => "korea" }),
    createBithumb: () => ({ client: () => rawClient("bithumb") }),
    createBinance: () => ({ client: () => rawClient(), venue: () => "spot" }),
    createHyperliquid: () => ({ client: () => rawClient("hyperliquid"), isTestnet: () => false }),
    ...overrides,
  };
}

test("browser backend initializes the wasm-bindgen module with wasm and relay URLs", async () => {
  const calls = [];
  const raw = rawModule({
    default: async (wasmUrl) => { calls.push(["wasm", wasmUrl]); },
    configureRelay: (relayUrl) => {
      calls.push(["relay", relayUrl]);
      return { ok: true, value: null };
    },
  });
  const backend = createBrowserBackend(async () => raw);

  await backend.initialize(options({
    wasmUrl: "https://cdn.example.test/maxt.wasm",
    relayUrl: "https://relay.example.test/maxt",
  }));

  assert.deepEqual(calls, [
    ["wasm", "https://cdn.example.test/maxt.wasm"],
    ["relay", "https://relay.example.test/maxt"],
  ]);
});

test("browser credentials require both explicit opt-in and a relay URL", async () => {
  let factories = 0;
  const raw = rawModule({
    configureRelay: () => ({ ok: true, value: null }),
    createBinance: () => {
      factories += 1;
      return { client: () => rawClient(), venue: () => "spot" };
    },
  });
  const credentials = { venue: "spot", api_key: "key", secret_key: "secret" };

  const denied = createBrowserBackend(async () => raw);
  await denied.initialize(options({ relayUrl: "https://relay.example.test" }));
  assert.throws(
    () => denied.binance(credentials),
    (error) => error.name === "InvalidRequestError"
      && error.field === "allowInsecureBrowserCredentials"
      && !error.message.includes("key")
      && !error.message.includes("secret"),
  );

  const missingRelay = createBrowserBackend(async () => raw);
  await missingRelay.initialize(options({ allowInsecureBrowserCredentials: true }));
  assert.throws(
    () => missingRelay.binance(credentials),
    (error) => error.name === "InvalidRequestError" && error.field === "relayUrl",
  );

  const allowed = createBrowserBackend(async () => raw);
  await allowed.initialize(options({
    allowInsecureBrowserCredentials: true,
    relayUrl: "https://relay.example.test",
  }));
  assert.doesNotThrow(() => allowed.binance(credentials));
  assert.equal(factories, 1);
});

test("browser custom adapters use the shared JSON backend bridge", async () => {
  let rawDispatch;
  let received;
  const raw = rawModule({
    createCustomClient(exchange, features, dispatch) {
      assert.equal(exchange, "binance");
      assert.deepEqual(features, ["markets"]);
      rawDispatch = dispatch;
      return rawClient(exchange);
    },
  });
  const backend = createBrowserBackend(async () => raw);
  await backend.initialize(options());

  const client = backend.customClient("binance", ["markets"], {
    dispatch: async (call) => {
      received = call;
      return { ok: true, value: { kind: "unit" } };
    },
    streamNext: async () => ({ ok: true, value: null }),
    streamClose: async () => ({ ok: true, value: null }),
  });

  assert.equal(client.exchange(), "binance");
  assert.deepEqual(
    JSON.parse(await rawDispatch(JSON.stringify({ kind: "balances" }))),
    { ok: true, value: { kind: "unit" } },
  );
  assert.deepEqual(received, { kind: "balances" });
});
