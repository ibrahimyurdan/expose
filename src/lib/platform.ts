// which shell the ui draws. rust injects the platform and who draws the
// window's edge before any page script runs (see window.rs); the browser
// preview takes ?platform=mac|windows and ?frame=os|css instead.
//
// no runtime imports, so node runs the tests in tests/ui straight from source.

export type Platform = "macos" | "windows";
// native: macos draws the edge. os: windows 11 draws its shadow and border.
// css: windows 10, where the ui draws a 1px edge itself.
export type Frame = "native" | "os" | "css";

interface PlatformEnv {
  injected?: unknown;
  search: string;
  userAgent: string;
  preview: boolean;
}

interface FrameEnv {
  injected?: unknown;
  platform: Platform;
  search: string;
  preview: boolean;
}

export function detectPlatform(env: PlatformEnv): Platform {
  if (env.preview) {
    const asked = new URLSearchParams(env.search).get("platform");
    if (asked === "mac" || asked === "macos") {
      return "macos";
    }
    if (asked === "windows" || asked === "win") {
      return "windows";
    }
  } else if (env.injected === "macos" || env.injected === "windows") {
    return env.injected;
  }
  return env.userAgent.includes("Macintosh") ? "macos" : "windows";
}

export function detectFrame(env: FrameEnv): Frame {
  if (env.platform === "macos") {
    return "native";
  }
  const asked = env.preview ? new URLSearchParams(env.search).get("frame") : env.injected;
  return asked === "os" ? "os" : "css";
}

const page = typeof window === "undefined" ? null : window;
const preview = page !== null && !("__TAURI_INTERNALS__" in page);

export const platform: Platform = page
  ? detectPlatform({
      injected: page.__EXPOSE_PLATFORM__,
      search: page.location.search,
      userAgent: page.navigator.userAgent,
      preview,
    })
  : "windows";

export const frame: Frame = page
  ? detectFrame({
      injected: page.__EXPOSE_FRAME__,
      platform,
      search: page.location.search,
      preview,
    })
  : "css";
