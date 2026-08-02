import assert from "node:assert/strict";
import test from "node:test";

import { EXCHANGES, FEATURES } from "../dist/generated/contract.js";
import { Exchange, Feature } from "../dist/models.js";

test("generated exchange and feature inventories match the public models", () => {
  assert.deepEqual(EXCHANGES, Exchange.values.map((value) => value.id));
  assert.deepEqual(FEATURES, Feature.values.map((value) => value.id));
});
