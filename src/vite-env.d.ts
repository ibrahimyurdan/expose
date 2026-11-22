/// <reference types="vite/client" />

// set by rust before any page script runs (see src-tauri/src/window.rs).
interface Window {
  __EXPOSE_PLATFORM__?: string;
  __EXPOSE_FRAME__?: string;
}
