import { createBrowserBackend } from "./browser-backend.js";
import { installBackend } from "./native.js";

installBackend(createBrowserBackend());

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
