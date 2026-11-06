import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "@/App";
import "@/index.css";

// macos renders a native frosted vibrancy behind the window; opt the body into
// translucency there so the blur shows. other platforms stay fully opaque.
if (navigator.userAgent.includes("Macintosh")) {
  document.documentElement.classList.add("vibrancy");
}

// the webview is a browser underneath: in release builds, hide its page menu
// (back, refresh, save as, print) and the reload, print and find shortcuts, so
// the panel behaves like an app. dev builds keep them for debugging.
if (import.meta.env.PROD) {
  window.addEventListener("contextmenu", (event) => event.preventDefault());
  window.addEventListener(
    "keydown",
    (event) => {
      const key = event.key.toLowerCase();
      const modifier = event.ctrlKey || event.metaKey;
      // event.code is the physical key, so non-latin keyboard layouts match too.
      const letter =
        ["r", "p", "f", "g"].includes(key) || ["KeyR", "KeyP", "KeyF", "KeyG"].includes(event.code);
      if (key === "f5" || key === "f3" || (modifier && letter)) {
        event.preventDefault();
      }
    },
    { capture: true },
  );
}

// when expose loses focus, drop focus from whatever control had it. otherwise the
// next time the window takes focus (a tray click, the champ-select pop on macos)
// a space or enter meant for league could flip a switch that still has it.
window.addEventListener("blur", () => {
  if (document.activeElement instanceof HTMLElement) {
    document.activeElement.blur();
  }
});

const container = document.getElementById("root");
if (!container) {
  throw new Error("expose: #root element is missing from index.html");
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
