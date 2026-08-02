import { AdapterError, InvalidRequestError } from "./errors.js";
import type { NativeBackend } from "./generated/api.js";

export interface InitializeOptions {
  readonly wasmUrl?: string | URL;
  readonly allowInsecureBrowserCredentials?: boolean;
}

export interface NormalizedInitializeOptions {
  readonly wasmUrl: string | null;
  readonly allowInsecureBrowserCredentials: boolean;
}

let installedBackend: NativeBackend | undefined;
let initializedOptions: NormalizedInitializeOptions | undefined;
let initialization: Promise<void> | undefined;

export function installBackend(backend: NativeBackend): void {
  if (installedBackend === backend) return;
  if (installedBackend !== undefined) throw new AdapterError("maxt backend is already installed");
  installedBackend = backend;
}

export function getBackend(): NativeBackend {
  if (installedBackend === undefined) throw new AdapterError("maxt backend is not installed");
  return installedBackend;
}

export function initialize(options: InitializeOptions = {}): Promise<void> {
  let normalized: NormalizedInitializeOptions;
  try {
    normalized = normalizeInitializeOptions(options);
  } catch (error) {
    return Promise.reject(error);
  }
  if (initialization !== undefined) {
    return sameInitializeOptions(initializedOptions!, normalized)
      ? initialization
      : Promise.reject(new InvalidRequestError(
        "initialize",
        "maxt is already initialized with different options",
      ));
  }

  initializedOptions = normalized;
  initialization = Promise.resolve().then(() => getBackend().initialize(normalized));
  return initialization;
}

export function ensureInitialized(): Promise<void> {
  return initialization ?? initialize();
}

function normalizeInitializeOptions(options: InitializeOptions): NormalizedInitializeOptions {
  const allowInsecureBrowserCredentials = options.allowInsecureBrowserCredentials;
  if (allowInsecureBrowserCredentials !== undefined
    && typeof allowInsecureBrowserCredentials !== "boolean") {
    throw new InvalidRequestError(
      "allowInsecureBrowserCredentials",
      "must be a boolean",
    );
  }
  return {
    wasmUrl: normalizeWasmUrl(options.wasmUrl),
    allowInsecureBrowserCredentials: allowInsecureBrowserCredentials ?? false,
  };
}

function normalizeWasmUrl(wasmUrl: string | URL | undefined): string | null {
  if (wasmUrl === undefined) return null;
  try {
    return new URL(String(wasmUrl), import.meta.url).href;
  } catch {
    throw new InvalidRequestError("wasmUrl", "must be a valid URL");
  }
}

function sameInitializeOptions(
  left: NormalizedInitializeOptions,
  right: NormalizedInitializeOptions,
): boolean {
  return left.wasmUrl === right.wasmUrl
    && left.allowInsecureBrowserCredentials === right.allowInsecureBrowserCredentials;
}
