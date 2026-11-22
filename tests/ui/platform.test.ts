// which shell the ui draws: the platform and who draws the window's edge.
import assert from "node:assert/strict";
import { test } from "node:test";

import { detectFrame, detectPlatform } from "../../src/lib/platform.ts";

const MAC = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15";
const WINDOWS = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Edg/131.0";

test("the app trusts the platform rust injected", () => {
  assert.equal(
    detectPlatform({ injected: "windows", search: "", userAgent: MAC, preview: false }),
    "windows",
  );
  assert.equal(
    detectPlatform({ injected: "macos", search: "", userAgent: WINDOWS, preview: false }),
    "macos",
  );
});

test("the app falls back to the user agent and ignores ?platform=", () => {
  assert.equal(detectPlatform({ search: "", userAgent: MAC, preview: false }), "macos");
  assert.equal(detectPlatform({ search: "", userAgent: WINDOWS, preview: false }), "windows");
  assert.equal(
    detectPlatform({ injected: "windows", search: "?platform=mac", userAgent: MAC, preview: false }),
    "windows",
  );
});

test("the preview takes ?platform= first, then the user agent", () => {
  for (const value of ["mac", "macos"]) {
    assert.equal(
      detectPlatform({ search: `?platform=${value}`, userAgent: WINDOWS, preview: true }),
      "macos",
    );
  }
  for (const value of ["windows", "win"]) {
    assert.equal(
      detectPlatform({ search: `?screen=lobby&platform=${value}`, userAgent: MAC, preview: true }),
      "windows",
    );
  }
  assert.equal(detectPlatform({ search: "?platform=linux", userAgent: MAC, preview: true }), "macos");
  assert.equal(detectPlatform({ search: "", userAgent: WINDOWS, preview: true }), "windows");
});

test("macos draws its own window edge", () => {
  assert.equal(detectFrame({ injected: "css", platform: "macos", search: "", preview: false }), "native");
  assert.equal(detectFrame({ platform: "macos", search: "?frame=os", preview: true }), "native");
});

test("windows in the app uses the injected frame, css by default", () => {
  assert.equal(detectFrame({ injected: "os", platform: "windows", search: "", preview: false }), "os");
  assert.equal(detectFrame({ injected: "css", platform: "windows", search: "", preview: false }), "css");
  assert.equal(detectFrame({ platform: "windows", search: "?frame=os", preview: false }), "css");
});

test("windows in the preview takes ?frame=, css by default", () => {
  assert.equal(detectFrame({ platform: "windows", search: "?frame=os", preview: true }), "os");
  assert.equal(detectFrame({ platform: "windows", search: "?frame=css", preview: true }), "css");
  assert.equal(detectFrame({ platform: "windows", search: "", preview: true }), "css");
});
