import assert from "node:assert/strict";
import test from "node:test";
import { initialize, installBackend } from "../dist/native.js";

async function freshNative(name) {
  const url = new URL("../dist/native.js", import.meta.url);
  url.searchParams.set("test", name);
  return import(url.href);
}

test("initialize is idempotent but rejects a different configuration", async () => {
  let calls = 0;
  installBackend({ initialize: async () => { calls += 1; } });

  await initialize({ allowInsecureBrowserCredentials: false });
  await initialize({ allowInsecureBrowserCredentials: false });
  assert.equal(calls, 1);
  await assert.rejects(
    initialize({ wasmUrl: "https://example.test/maxt.wasm" }),
    { name: "InvalidRequestError" },
  );
});

test("concurrent initialization shares one promise and normalized options", async () => {
  const { ensureInitialized: ensureFresh, initialize: initializeFresh, installBackend: installFresh } =
    await freshNative("concurrent");
  let captured;
  let release;
  installFresh({
    initialize: (options) => {
      captured = options;
      return new Promise((resolve) => { release = resolve; });
    },
  });

  const absoluteWasmUrl = new URL("../dist/maxt.wasm", import.meta.url).href;
  const first = initializeFresh({ wasmUrl: "./maxt.wasm" });
  const second = initializeFresh({
    wasmUrl: absoluteWasmUrl,
    allowInsecureBrowserCredentials: false,
  });
  assert.equal(first, second);
  assert.equal(ensureFresh(), first);
  await Promise.resolve();
  assert.deepEqual(captured, {
    wasmUrl: absoluteWasmUrl,
    allowInsecureBrowserCredentials: false,
    relayUrl: null,
  });

  release();
  await first;
});

test("a failed first initialization is shared and never retried", async () => {
  const { initialize: initializeFresh, installBackend: installFresh } = await freshNative("failed");
  let calls = 0;
  installFresh({
    initialize: async () => {
      calls += 1;
      throw new Error("wasm load failed");
    },
  });

  const first = initializeFresh();
  await assert.rejects(first, /wasm load failed/);
  const second = initializeFresh({ allowInsecureBrowserCredentials: false });
  assert.equal(second, first);
  await assert.rejects(second, /wasm load failed/);
  assert.equal(calls, 1);
});

test("installBackend accepts only the same backend object", async () => {
  const { installBackend: installFresh } = await freshNative("backend-identity");
  const backend = { initialize: async () => {} };

  installFresh(backend);
  installFresh(backend);
  assert.throws(
    () => installFresh({ initialize: async () => {} }),
    { name: "AdapterError" },
  );
});

test("initialize rejects a non-boolean browser credential gate before backend use", async () => {
  const { initialize: initializeFresh, installBackend: installFresh } = await freshNative("invalid-gate");
  let calls = 0;
  installFresh({ initialize: async () => { calls += 1; } });

  const result = initializeFresh({ allowInsecureBrowserCredentials: "false" });
  assert.equal(result instanceof Promise, true);
  await assert.rejects(
    result,
    (error) => error.name === "InvalidRequestError"
      && error.field === "allowInsecureBrowserCredentials",
  );
  assert.equal(calls, 0);
});

test("initialize rejects an invalid wasm URL asynchronously with a structured error", async () => {
  const { initialize: initializeFresh } = await freshNative("invalid-wasm-url");
  let result;

  assert.doesNotThrow(() => {
    result = initializeFresh({ wasmUrl: "http://[" });
  });
  assert.equal(result instanceof Promise, true);
  await assert.rejects(
    result,
    (error) => error.name === "InvalidRequestError" && error.field === "wasmUrl",
  );
});

test("initialize normalizes and compares the relay URL", async () => {
  const { initialize: initializeFresh, installBackend: installFresh } = await freshNative("relay-url");
  let captured;
  installFresh({ initialize: async (options) => { captured = options; } });

  await initializeFresh({ relayUrl: "https://relay.example.test/api" });
  assert.equal(captured.relayUrl, "https://relay.example.test/api");
  await assert.rejects(
    initializeFresh({ relayUrl: "https://other.example.test/api" }),
    (error) => error.name === "InvalidRequestError" && error.field === "initialize",
  );
});

test("initialize rejects invalid URL option types before backend use", async () => {
  const { initialize: initializeFresh, installBackend: installFresh } = await freshNative("invalid-url-type");
  let calls = 0;
  installFresh({ initialize: async () => { calls += 1; } });

  await assert.rejects(
    initializeFresh({ relayUrl: false }),
    (error) => error.name === "InvalidRequestError" && error.field === "relayUrl",
  );
  assert.equal(calls, 0);
});
