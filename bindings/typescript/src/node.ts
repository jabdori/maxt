import { createRequire } from "node:module";

import { AdapterError } from "./errors.js";
import {
  NATIVE_API_VERSION,
  createJsonBackend,
  type RawNativeModule,
} from "./generated/api.js";
import { installBackend } from "./native.js";

const loaded: unknown = createRequire(import.meta.url)("../native.cjs");
if (typeof loaded !== "object" || loaded === null
  || !("NATIVE_API_VERSION" in loaded)
  || loaded.NATIVE_API_VERSION !== NATIVE_API_VERSION) {
  throw new AdapterError(`unsupported maxt native API; expected version ${NATIVE_API_VERSION}`);
}
installBackend(createJsonBackend(loaded as RawNativeModule));

export {
  Adapter,
  BinanceAdapter,
  BinanceListenKey,
  BithumbAdapter,
  Client,
  HyperliquidAdapter,
  UpbitAdapter,
} from "./generated/api.js";
export * from "./errors.js";
export * from "./models.js";
export { initialize, type InitializeOptions } from "./native.js";
export {
  AccountStream,
  AsyncStream,
  MarketStream,
  StreamError,
  StreamEvent,
  type StreamItem,
} from "./stream.js";
