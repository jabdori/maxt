import { InvalidRequestError, type MaxtError } from "./errors.js";
import type { AccountEvent, MarketEvent } from "./models.js";

export class StreamEvent<T> {
  readonly kind = "event";

  constructor(readonly event: T) {
    Object.freeze(this);
  }
}

export class StreamError {
  readonly kind = "error";

  constructor(readonly error: MaxtError) {
    Object.freeze(this);
  }
}

export type StreamItem<T> = StreamEvent<T> | StreamError;

export class AsyncStream<T> implements AsyncIterableIterator<T> {
  readonly #iterator: AsyncIterator<T>;
  readonly #closeHook: () => void | Promise<void>;
  #pending = false;
  #closed = false;
  #closePromise: Promise<void> | null = null;
  #wakeClosed: (() => void) | null = null;

  constructor(source: AsyncIterable<T>, close: () => void | Promise<void> = () => {}) {
    this.#iterator = source[Symbol.asyncIterator]();
    this.#closeHook = close;
  }

  [Symbol.asyncIterator](): AsyncIterableIterator<T> {
    return this;
  }

  async next(): Promise<IteratorResult<T>> {
    if (this.#closed) return { done: true, value: undefined };
    if (this.#pending) {
      throw new InvalidRequestError("next", "maxt streams allow only one pending next() call");
    }

    this.#pending = true;
    try {
      const closed = new Promise<IteratorResult<T>>((resolve) => {
        this.#wakeClosed = () => resolve({ done: true, value: undefined });
      });
      return await Promise.race([this.#iterator.next(), closed]);
    } finally {
      this.#wakeClosed = null;
      this.#pending = false;
    }
  }

  return(): Promise<IteratorResult<T>> {
    return this.close().then(() => ({ done: true, value: undefined }));
  }

  close(): Promise<void> {
    if (this.#closePromise !== null) return this.#closePromise;

    this.#closed = true;
    this.#wakeClosed?.();
    this.#closePromise = (async () => {
      let failure: unknown;
      try {
        await this.#closeHook();
      } catch (error) {
        failure = error;
      }
      try {
        await this.#iterator.return?.();
      } catch (error) {
        failure ??= error;
      }
      if (failure !== undefined) throw failure;
    })();
    return this.#closePromise;
  }
}

export class MarketStream extends AsyncStream<StreamItem<MarketEvent>> {}

export class AccountStream extends AsyncStream<StreamItem<AccountEvent>> {}
