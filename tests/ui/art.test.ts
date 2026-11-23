// the league client's own images: which file names a tier or role maps to,
// and the address the page asks for them at on each platform.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { artUrl, crestFile, emblemFile, roleFile } from "../../src/lib/art.ts";

test("tiers map to the client's small crests, in any case", () => {
  assert.equal(crestFile("DIAMOND"), "images/ranked-mini-crests/diamond.svg");
  assert.equal(crestFile("unranked"), "images/ranked-mini-crests/unranked.svg");
});

test("an unknown tier has no file, so the drawing shows", () => {
  assert.equal(crestFile("WOOD"), null);
  assert.equal(crestFile(""), null);
  assert.equal(emblemFile("UNRANKED"), null);
});

test("emblems and role icons use the client's file names", () => {
  assert.equal(emblemFile("Grandmaster"), "images/ranked-emblem/emblem-grandmaster.png");
  assert.equal(roleFile("utility"), "svg/position-utility.svg");
  assert.equal(roleFile("middle", true), "svg/position-middle-light.svg");
});

test("the art url follows the platform's custom scheme form", () => {
  assert.equal(
    artUrl("svg/position-top.svg", "macos"),
    "league-art://localhost/svg/position-top.svg",
  );
  assert.equal(
    artUrl("svg/position-top.svg", "windows"),
    "http://league-art.localhost/svg/position-top.svg",
  );
});

test("the content security policy lets both forms load", () => {
  const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
  const img = String(conf.app.security.csp)
    .split(";")
    .find((part) => part.trim().startsWith("img-src"));
  assert.ok(img?.includes("league-art:"), img);
  assert.ok(img?.includes("http://league-art.localhost"), img);
});
