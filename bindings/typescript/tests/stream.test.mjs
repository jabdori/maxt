import assert from "node:assert/strict";
import test from "node:test";

import { InvalidRequestError } from "../dist/errors.js";
import { AsyncStream } from "../dist/stream.js";

test("close wakes a pending next and closes the source exactly once", async () => {
  let sourceCloseCount = 0;
  let release;
  const source = {
    [Symbol.asyncIterator]() {
      return {
        next: () => new Promise((resolve) => { release = resolve; }),
        return: async () => {
          sourceCloseCount += 1;
          release?.({ done: true, value: undefined });
          return { done: true, value: undefined };
        },
      };
    },
  };
  const stream = new AsyncStream(source);
  const pending = stream.next();

  await Promise.all([stream.close(), stream.close()]);

  assert.deepEqual(await pending, { done: true, value: undefined });
  assert.equal(sourceCloseCount, 1);
});

test("only one next call may be pending", async () => {
  const source = {
    [Symbol.asyncIterator]() {
      return { next: () => new Promise(() => {}) };
    },
  };
  const stream = new AsyncStream(source);
  void stream.next();

  await assert.rejects(stream.next(), InvalidRequestError);
  await stream.close();
});

test("close preserves cleanup failures and stays idempotent", async () => {
  const expected = new Error("cleanup failed");
  const source = {
    [Symbol.asyncIterator]() {
      return {
        next: async () => ({ done: true, value: undefined }),
        return: async () => ({ done: true, value: undefined }),
      };
    },
  };
  const stream = new AsyncStream(source, async () => { throw expected; });

  const first = stream.close();
  const second = stream.close();
  assert.equal(first, second);
  await assert.rejects(first, expected);
});
