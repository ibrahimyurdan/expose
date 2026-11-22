// a row's flags share one line: as many chips as fit, then a "+N" chip for
// the rest, so the row never grows when its flags arrive.
import assert from "node:assert/strict";
import { test } from "node:test";

import { fitCount } from "../../src/lib/fit.ts";

test("every chip shows when they all fit", () => {
  assert.equal(fitCount([150, 90, 140], 4, 30, 400), 3);
});

test("chips that just fit need no room for the count", () => {
  assert.equal(fitCount([100, 100], 4, 30, 204), 2);
});

test("leaves room for the count when some chips do not fit", () => {
  // 150 + 4 + 30 fits in 260; adding the 90 chip would not.
  assert.equal(fitCount([150, 90, 140], 4, 30, 260), 1);
  // 150 + 4 + 90 + 4 + 30 fits in 300.
  assert.equal(fitCount([150, 90, 140], 4, 30, 300), 2);
});

test("the first chip always shows, cut short when it must be", () => {
  assert.equal(fitCount([300, 50], 4, 30, 200), 1);
  assert.equal(fitCount([300], 4, 30, 200), 1);
});

test("a fraction of a pixel over still fits", () => {
  assert.equal(fitCount([100, 100.4], 4, 30, 204), 2);
});

test("no chips, nothing to show", () => {
  assert.equal(fitCount([], 4, 30, 200), 0);
});
