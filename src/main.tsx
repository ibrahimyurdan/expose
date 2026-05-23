import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "@/App";
import "@/index.css";

// macos renders a native frosted vibrancy behind the window; opt the body into
// translucency there so the blur shows. other platforms stay fully opaque.
if (navigator.userAgent.includes("Macintosh")) {
  document.documentElement.classList.add("vibrancy");
}

const container = document.getElementById("root");
if (!container) {
  throw new Error("expose: #root element is missing from index.html");
}

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
