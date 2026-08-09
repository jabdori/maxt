import { AdapterError, InvalidRequestError, errorFromWire } from "./errors.js";
import {
  NATIVE_API_VERSION,
  createJsonBackend,
  type NativeBackend,
  type NativeOutcome,
  type RawNativeModule,
} from "./generated/api.js";
import type { ErrorWire } from "./generated/contract.js";
import type { NormalizedInitializeOptions } from "./native.js";

interface BrowserWasmModule extends RawNativeModule {
  readonly default: (input?: string | URL) => unknown | Promise<unknown>;
  readonly configureRelay?: (relayUrl: string) => unknown;
}

export type BrowserWasmLoader = () => Promise<unknown>;

export function createBrowserBackend(
  loadWasm: BrowserWasmLoader = loadBrowserWasm,
): NativeBackend {
  let selected: NormalizedInitializeOptions | undefined;
  let backend: NativeBackend | undefined;

  const initializedBackend = (): NativeBackend => {
    if (backend === undefined) {
      throw new AdapterError("browser backend is not initialized; await initialize() first");
    }
    return backend;
  };

  const requireCredentialRelay = (hasCredentials: boolean): void => {
    if (!hasCredentials) return;
    if (selected?.allowInsecureBrowserCredentials !== true) {
      throw new InvalidRequestError(
        "allowInsecureBrowserCredentials",
        "browser credentials require explicit initialize opt-in",
      );
    }
    if (selected.relayUrl === null) {
      throw new InvalidRequestError(
        "relayUrl",
        "browser credentials require a relay URL",
      );
    }
  };

  return {
    async initialize(options) {
      if (selected !== undefined && !sameOptions(selected, options)) {
        throw new InvalidRequestError(
          "initialize",
          "WebAssembly was already initialized with different options",
        );
      }
      if (backend !== undefined) return;
      selected = options;

      const wasm = browserWasmModule(await loadWasm());
      await wasm.default(options.wasmUrl ?? undefined);
      if (options.relayUrl !== null) {
        if (wasm.configureRelay === undefined) {
          throw new AdapterError("WebAssembly backend does not support relay configuration");
        }
        unwrapOptionalOutcome(wasm.configureRelay(options.relayUrl));
      }
      backend = createJsonBackend(wasm);
    },
    customClient: (exchange, features, callbacks) =>
      initializedBackend().customClient(exchange, features, callbacks),
    upbit(options) {
      requireCredentialRelay(options.access_key !== null || options.secret_key !== null);
      return initializedBackend().upbit(options);
    },
    bithumb(options) {
      requireCredentialRelay(options.access_key !== null || options.secret_key !== null);
      return initializedBackend().bithumb(options);
    },
    binance(options) {
      requireCredentialRelay(options.api_key !== null || options.secret_key !== null);
      return initializedBackend().binance(options);
    },
    hyperliquid(options) {
      requireCredentialRelay(options.private_key !== null);
      return initializedBackend().hyperliquid(options);
    },
  };
}

async function loadBrowserWasm(): Promise<unknown> {
  return import(new URL("../wasm/maxt_wasm.js", import.meta.url).href);
}

function browserWasmModule(value: unknown): BrowserWasmModule {
  if (typeof value !== "object" || value === null
    || !("NATIVE_API_VERSION" in value)
    || value.NATIVE_API_VERSION !== NATIVE_API_VERSION
    || !("default" in value)
    || typeof value.default !== "function") {
    throw new AdapterError(`unsupported maxt WebAssembly API; expected version ${NATIVE_API_VERSION}`);
  }
  return value as BrowserWasmModule;
}

function unwrapOptionalOutcome(value: unknown): void {
  if (typeof value !== "object" || value === null || !("ok" in value)) return;
  const outcome = value as NativeOutcome<null>;
  if (!outcome.ok) throw errorFromWire(outcome.error as ErrorWire);
}

function sameOptions(
  left: NormalizedInitializeOptions,
  right: NormalizedInitializeOptions,
): boolean {
  return left.wasmUrl === right.wasmUrl
    && left.allowInsecureBrowserCredentials === right.allowInsecureBrowserCredentials
    && left.relayUrl === right.relayUrl;
}
