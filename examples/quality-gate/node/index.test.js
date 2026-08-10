import { test } from "node:test";
import assert from "node:assert";
import { add } from "./index.js";

test("add(2, 3) returns 5", () => {
  assert.strictEqual(add(2, 3), 5);
});
