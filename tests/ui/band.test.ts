// the champ select band: its clock and which bars show in which color.
import assert from "node:assert/strict";
import { test } from "node:test";

import { armedFinal, barModes, formatBandClock } from "../../src/lib/band.ts";

test("short phases print bare seconds, rounded up", () => {
  assert.equal(formatBandClock(26000, 30000), "26");
  assert.equal(formatBandClock(26400, 30000), "27");
  assert.equal(formatBandClock(0, 30000), "0");
});

test("phases of 100 s or more print minutes and seconds", () => {
  assert.equal(formatBandClock(95000, 120000), "1:35");
});

test("an unknown phase length goes by the time left", () => {
  assert.equal(formatBandClock(5000, 0), "5");
  assert.equal(formatBandClock(125000, 0), "2:05");
});

test("an armed dodge in finalization turns both bars red", () => {
  assert.deepEqual(barModes("finalization", "none", true), { left: "red", right: "red" });
});

test("planning, finalization and an unknown stage are teal", () => {
  for (const stage of ["planning", "finalization", "unknown"] as const) {
    assert.deepEqual(barModes(stage, "none", false), { left: "teal", right: "teal" });
  }
});

test("bans are red on the side that is banning", () => {
  assert.deepEqual(barModes("ban", "both", false), { left: "red", right: "red" });
  assert.deepEqual(barModes("ban", "ally", false), { left: "red", right: "hidden" });
  assert.deepEqual(barModes("ban", "enemy", false), { left: "hidden", right: "red" });
});

test("picks show only the picking team's bar", () => {
  assert.deepEqual(barModes("pick", "ally", false), { left: "teal", right: "hidden" });
  assert.deepEqual(barModes("pick", "enemy", false), { left: "hidden", right: "red" });
  assert.deepEqual(barModes("pick", "both", false), { left: "teal", right: "red" });
});

test("a ban or pick stage with nobody on the clock stays teal", () => {
  assert.deepEqual(barModes("ban", "none", false), { left: "teal", right: "teal" });
  assert.deepEqual(barModes("pick", "none", false), { left: "teal", right: "teal" });
});

test("an armed dodge counts down while the lobby locks in", () => {
  assert.equal(
    armedFinal({ phase: "FINALIZATION", stage: "finalization", dodge: { armed: true } }),
    true,
  );
});

test("game starting is past the dodge: no countdown and no red bars", () => {
  // the backend fires only in FINALIZATION; GAME_STARTING shares its stage.
  assert.equal(
    armedFinal({ phase: "GAME_STARTING", stage: "finalization", dodge: { armed: true } }),
    false,
  );
});

test("an unarmed dodge has no final seconds", () => {
  assert.equal(
    armedFinal({ phase: "FINALIZATION", stage: "finalization", dodge: { armed: false } }),
    false,
  );
});
