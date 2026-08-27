import test from "node:test";
import assert from "node:assert/strict";
import { scenarioFor, scenarios } from "../src/replay.mjs";

test("recorded demo contains all three real gate states", () => {
  assert.deepEqual(Object.keys(scenarios), ["clean", "repeat", "partial"]);
  assert.equal(scenarios.clean.status, "safe");
  assert.equal(scenarios.repeat.status, "safe");
  assert.equal(scenarios.partial.status, "unsafe");
});

test("unknown scenario safely falls back to clean", () => {
  assert.equal(scenarioFor("unknown"), scenarios.clean);
});
