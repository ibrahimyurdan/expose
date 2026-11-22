# expose league-style restyle implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** restyle every expose screen in the league client's look (the closest version), give mac and windows their own window shells, and make the app smaller, quieter and faster, as the approved design describes.

**Architecture:** the look is plain css ported from the reference mockup into `src/styles/` (fonts, tokens, base, type roles, controls, shell, lobby, screens), used by restyled primitives in `src/components/ui/` and our own svg drawings in `src/components/art/`. the window is built in rust per platform (`src-tauri/src/window.rs`), which also tells the ui its platform and frame through an initialization script. the backend learns the champ select stage, turn and phase length, and takes the optimization changes (scan interval, unchanged-view skipping, a champion event, faster chat reads, ranks before history, release profile, ring).

**Tech stack:** tauri 2.11.2 (tao 0.35.2), rust 2021, objc2-app-kit 0.3.2 (mac), windows 0.61 + windows-version 0.1 + webview2-com 0.38 (windows), react 19, typescript 6, tailwind 4, radix, vite 8, `node --test` (node 22.18+ strips types natively) for small ui tests.

**Spec:** docs/superpowers/specs/2026-11-21-expose-league-style-design.md. reference material, local and gitignored: `.claude/design/league-style/` (read its readme). `mockup/expose.css` is the value source and wins over `style-spec.md` where they differ; `style-spec.md` covers what the mockup does not draw (tooltip, toast, empty state, details panel, motion, the welcome and idle recipes, the drawings in its section 6). reference stylesheet map: tokens 4–151, base 152–171, type roles 172–204, buttons 205–240, checkbox 241–251, switch 252–269, radios 270–276, select 277–291, chip/tag 292–307, status 308–316, frame/lips 317–325, skeleton/loader 326–334, scrollbar 335–340, shell 341–367, band 368–395, summary 396–414, dodge 416–427, rows 429–480, settings 481–499. the mockup is frozen for screenshots: animations come from `style-spec.md` section 3.

## global constraints

- comments, readme and committed docs are lowercase, plain, no emojis. never cite league client files (plugin names, hashes, selectors) in code, comments or committed docs.
- nothing of riot's ships: no fonts, images, sounds, svg path data, css or js from the client, communitydragon or the install. images only from data dragon. every drawing is ours.
- fonts: "Expose Display" is cinzel 700. "Expose Body" is source sans 3: weight 400 → the regular file, 700 → the semibold (600) file, 900 → the bold (700) file, plus 400 italic. woff2 files in `src/assets/fonts/` with both ofl license texts. stacks: `--font-display: "Expose Display", "Times New Roman", Times, Baskerville, Georgia, serif`; `--font-body: "Expose Body", Arial, "Helvetica Neue", Helvetica, sans-serif`.
- the window opens at 460 x 680 and shrinks to 420 x 600; every screen must fit both, on both shells.
- corners are 0 except 2px (tags, skeletons, the dark panel), 3px (count badge), 4px (toast close plate), 16px (chips) and 50% (dots, portraits, round buttons).
- colors come from the design's token table. the departures, exactly: dodge label #ff2345, kda-low #d94545, loss dots #be1e37, red chip fill #2a1015 with edge rgba(255,35,69,.2), focus = the control's hover look plus `outline: 1px solid #c8aa6e; outline-offset: 2px`.
- the band clock prints bare seconds when the phase total is under 100000 ms, otherwise m:ss.
- copy: v0.2's strings stay, sentence case in code (display roles uppercase through css). the one change is the dodge confirm label "Confirm". the only new strings are riot's notice and the font credits.
- riot's notice, word for word: "Expose isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc."
- `prefers-reduced-motion: reduce` turns off pulses, glows, intros and the shimmer.
- mac: never set `traffic_light_position`. windows: `decorations(false)`; windows 11 (build ≥ 22000) keeps the os shadow with `DWMWCP_DONOTROUND` and border colorref `0x002D281E`; windows 10 has no shadow and a 1px #1e282d css border on the sides and bottom.
- checks: `zsh .claude/design/checks.sh` from the repo root runs fmt, mac clippy, tests, windows clippy through zig, ui tests and the ui build; it must print `== result: PASS` before every commit (ui-only commits may run just `npm run test:ui && npm run build`).
- visual checks use the preview (`npm run dev`, http://localhost:1420/?screen=…&platform=mac|windows) at 460 x 680 and 420 x 600; `.claude/design/shoot.mjs` takes headless screenshots.
- commits: `git -c user.name="Ibrahim Yurdan" -c user.email="ibrahimyurdan@users.noreply.github.com" commit -m "<scope>: <lowercase subject>" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>" -- <explicit paths>`. never push or tag.

## review focus

1. lobbies that are not ranked solo/duo (blind pick, aram, customs, bots and unfilled slots): missing or odd `actions` must give teal bars and no turn marks, never a crash or a blank band. tests: task 1 `blind_pick_is_your_teams_pick`, `odd_actions_do_not_break_the_session`, `ignores_actions_that_are_not_turns`; task 11 `barModes` unknown cases; task 12 checks `?lobby=normal` and the new `?lobby=aram`.
2. long text at 420px: 16-character riot ids with 5-character tags, "Nunu & Willump", "Grandmaster 1,234 LP", three flags, the armed countdown label. nothing may push the rank, the kda or the dodge button out of place. checks: task 11 armed label at 420 x 600; task 12 `?lobby=long` at 420 x 600.
3. the mac's hidden traffic lights coming back after minimize and restore from the dock, close and dock reopen, focus changes, resizing or a move to another display: they must stay hidden and our buttons must keep working. task 10 re-applies the hide on resize and focus; task 16 runs this list on the built app.
4. the windows frame on windows 10 and 11 at 100, 125 and 150% scaling: square corners, the border, edge and grip resizing, snap, alt+f4, no taskbar button after a show. test: task 5 `frame_follows_the_platform_and_windows_build`; task 16 hands the owner this list for the exe.
5. fonts that fail or load slowly: the window must still appear (1.5 s fallback) and fall back to the stacks without breaking the layout. tests: task 5 `first_show_happens_once`; task 9 checks the gallery and lobby with the font files blocked in the preview.

## file structure

backend (`src-tauri/`):
- `src/lobby/session.rs`: + `SessionAction`, `Stage`, `Acting`, `Session::{stage, acting, is_cell_acting}`, `SessionTimer::total_time_in_phase`.
- `src/lobby/view.rs`: + `LobbyView::{stage, acting, total_ms}`, `PlayerView::acting`.
- `src/lobby/mod.rs`: turn log line, unchanged-view skipping, chat cadence and time-based patience, quick stats arrival.
- `src/lobby/stats.rs`: `fetch_quick` and `fetch_rest` split.
- `src/lcu/mod.rs`, `src/lcu/auth.rs`: scan interval.
- `src/ddragon.rs`: champion event, reload on connect.
- `src/tls.rs` (new): ring as the process crypto provider.
- `src/window.rs` (new): window building per platform, frame detection, init script, first-show gate, hide, traffic lights, dwm corners, webview memory.
- `src/main.rs`, `src/commands.rs`, `src/tray.rs`: wiring.
- `Cargo.toml`, `tauri.conf.json`, `capabilities/default.json`, `.github/workflows/{ci,release}.yml`.

frontend (`src/`):
- `styles/{fonts,tokens,base,type,controls,shell,lobby,screens}.css` (new), `index.css` (imports only).
- `assets/fonts/*` and `assets/art/*` (new).
- `lib/platform.ts`, `lib/band.ts` (new, pure, tested from `tests/ui/`).
- `components/art/*` (new): our drawings.
- `components/ui/*`: restyled primitives; new `checkbox.tsx`, `switch.tsx`, `radio-group.tsx`, `select.tsx`, `status.tsx`, `frame.tsx`, `loader.tsx`; gone `toggle.tsx`, `segmented.tsx`, `pill.tsx`, `card.tsx`, `scroll-area.tsx`, `separator.tsx`.
- `components/shell/{TitleBar,WindowEdge,RiotNotice,FontCredits}.tsx` (new), `Notices.tsx`; gone `Header.tsx`.
- `components/lobby/Band.tsx` (new) and every other lobby component; all five screens; `App.tsx`, `main.tsx`, `lib/bridge.ts`, `types/index.ts`, `hooks/{useChampions.ts,notices.tsx}`, `dev/{fixtures,preview}.ts`, `dev/Gallery.tsx` (new, preview only).
- `tests/ui/*.test.ts` (new, outside `src/` so `tsc` never sees `node:test`).

---

### task 1: the champ select stage, turn and phase length

**Files:**
- Modify: `src-tauri/src/lobby/session.rs` (structs 13–87, tests 149–269)
- Modify: `src-tauri/src/lobby/view.rs` (LobbyView 21–39, PlayerView 41–67, build_view 163–251, serialization test 913–1020)
- Modify: `src-tauri/src/lobby/mod.rs` (LobbyState 57–93, reset/enter, publish 279–313)
- Modify: `src-tauri/src/features/dodge.rs:270-273` (the `SessionTimer` literal gets `..SessionTimer::default()`)

**Interfaces:**
- Produces, in `session.rs` (all fields `#[serde(deserialize_with = "lenient")]`, structs `Default` + `#[serde(default, rename_all = "camelCase")]`):
  - `pub struct SessionAction { pub actor_cell_id: i64, pub completed: bool, pub is_ally_action: bool, pub is_in_progress: bool, #[serde(rename = "type")] pub kind: String }`
  - `Session::actions: Vec<Vec<SessionAction>>`, `SessionTimer::total_time_in_phase: i64`
  - `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)] #[serde(rename_all = "camelCase")] pub enum Stage { #[default] Unknown, Planning, Ban, Pick, Finalization }`
  - `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)] #[serde(rename_all = "camelCase")] pub enum Acting { #[default] None, Ally, Enemy, Both }`
  - `impl Session { pub fn stage(&self) -> Stage; pub fn acting(&self) -> Acting; pub fn is_cell_acting(&self, cell_id: i64) -> bool }`
- Produces, in `view.rs`: `LobbyView::{stage: Stage, acting: Acting, total_ms: i64}` (json `stage`, `acting`, `totalMs`), `PlayerView::acting: bool`.
- Rules:
  - a turn action is an in-progress action whose `kind` is "ban" or "pick"; anything else ("ten_bans_reveal", unknown) never counts.
  - `stage`: "PLANNING" → Planning; "FINALIZATION" and "GAME_STARTING" → Finalization; "BAN_PICK" → Ban if any turn action is a ban, else Pick if any is a pick, else Unknown; any other phase → Unknown.
  - `acting`: only for Ban and Pick: Both, Ally or Enemy from the turn actions' `is_ally_action`; otherwise None.
  - `is_cell_acting(cell)`: stage is Ban or Pick and an ally turn action has `actor_cell_id == cell`.
  - `total_ms = session.timer.total_time_in_phase` (0 when missing).

- [ ] **Step 1: write the failing tests.** in `session.rs` tests, a helper `fn session(phase: &str, actions: serde_json::Value) -> Session` loads the ranked fixture as a `Value`, sets `timer.phase` and `actions`, and deserializes. tests:
  - `reads_the_phase_length_and_actions`: the plain fixture has `total_time_in_phase == 30000`, `actions.len() == 2`, `actions[0][1].actor_cell_id == 1`, `actions[1][0].kind == "pick"`; every action is completed, so `stage() == Stage::Unknown` and `acting() == Acting::None`.
  - `planning_has_no_turn`: `session("PLANNING", json!([]))` → Planning, None.
  - `simultaneous_bans_are_both_teams`: BAN_PICK with in-progress bans by ally cells 0 and 1 and an enemy cell 5 → Ban, Both; cells 0 and 1 acting, 2 and 5 not.
  - `your_teams_pick`: one in-progress ally pick by cell 2 → Pick, Ally; only cell 2 acting.
  - `enemy_pick`: one in-progress enemy pick by cell 7 → Pick, Enemy; no cell acting.
  - `blind_pick_is_your_teams_pick`: in-progress ally picks by cells 0–4 → Pick, Ally; all five acting.
  - `finalization_has_no_turn`: FINALIZATION, and separately GAME_STARTING, both → Finalization, None.
  - `ignores_actions_that_are_not_turns`: BAN_PICK with an in-progress "ten_bans_reveal" → Unknown, None.
  - `odd_actions_do_not_break_the_session`: `actions` set to `null`, to `"x"` and to `[[{"type": 5}]]` each parse; Unknown, None.
  - in `view.rs`: `view_carries_the_turn` (an ally pick in progress for the second slot's cell → `view.stage == Stage::Pick`, `view.acting == Acting::Ally`, `view.total_ms == 30000`, exactly that player `acting`), and `serializes_like_the_frontend_expects` gains top-level keys `acting`, `stage`, `totalMs` and the player json `"acting": false`.
- [ ] **Step 2: run** `cargo test --manifest-path src-tauri/Cargo.toml lobby::` → fails to compile (`SessionAction`, `stage` missing).
- [ ] **Step 3: implement** the session types and methods, then fix the `SessionTimer` literal in `features/dodge.rs:270-273`.
- [ ] **Step 4: implement** the view fields and set them in `build_view`.
- [ ] **Step 5: log turn changes.** `LobbyState` gains `logged_turn: Option<(Stage, Acting)>`, cleared in `reset` and `enter`. `publish` logs `lobby: stage=<stage> acting=<acting>` (lowercase names) only when the pair changes, like `log_matches` (655–661).
- [ ] **Step 6: run** `zsh .claude/design/checks.sh` → PASS, with the new tests among 111+.
- [ ] **Step 7: commit** `lobby: read the champ select stage, turn and phase length` with the four files.

### task 2: a quieter backend

**Files:**
- Modify: `src-tauri/src/lcu/mod.rs` (supervise 65–98, wait_for_next_attempt 200–210, run_session 100–139)
- Modify: `src-tauri/src/lcu/auth.rs` (find_credentials 19–36)
- Modify: `src-tauri/src/lobby/mod.rs` (publish 279–313)
- Modify: `src-tauri/src/ddragon.rs`, `src-tauri/Cargo.toml` (reqwest features + "gzip")
- Modify: `src/hooks/useChampions.ts`, `src/lib/bridge.ts`, `src/dev/preview.ts`

**Interfaces:**
- `lcu/mod.rs`: `const SCAN_EVERY_WATCHED: Duration = 30s`, `const SCAN_EVERY: Duration = 5s`, `fn scan_due(last: Option<Instant>, now: Instant, watching: bool) -> bool`.
- `auth.rs`: `pub fn find_credentials(scan_processes: bool)` with its current return type; with `false` it reads lockfiles only.
- `lobby/mod.rs`: `fn should_publish(previous: Option<&LobbyView>, next: &LobbyView) -> bool` (`previous != Some(next)`); `publish` stores and emits only when it is true.
- `ddragon.rs`: `pub const EVENT_CHAMPIONS: &str = "expose://champions"`; `load` emits the data after storing it; `pub async fn refresh(app: AppHandle)` fetches versions.json and reloads and emits only when `needs_reload(current, latest)`; `fn needs_reload(current: Option<&str>, latest: &str) -> bool`.
- `bridge.ts`: `onChampions(callback: (data: ChampionData) => void): Promise<UnlistenFn>` (preview: never fires).
- the scan interval alone takes the scan from every 2 s to every 30 s while the lockfile is watched (7–18 ms per scan, about 0.05% of a core), so the design's libproc listing is not needed; note this in the commit body.

- [ ] **Step 1: failing tests:** `scans_rarely_while_the_lockfile_is_watched` (no last scan → due; watched: 10 s → not due, 31 s → due; unwatched: 4 s → not, 6 s → due), `identical_views_are_not_sent_again`, `reloads_only_for_a_newer_patch` (None → true; "16.23.1" vs "16.23.1" → false; vs "16.24.1" → true).
- [ ] **Step 2: run** them → fail.
- [ ] **Step 3: implement** the scan gate in `supervise` (track the last scan; pass `scan_due(..)` to `find_credentials`), `should_publish`, the champion event and `refresh`. `run_session` spawns `ddragon::refresh` once the client connects. reqwest gets `"gzip"`.
- [ ] **Step 4: ui:** `useChampions` reads `getChampions()` once, then subscribes with `onChampions`; the 1.5 s loop goes.
- [ ] **Step 5: run** checks → PASS. preview `?screen=lobby`: champion squares still show.
- [ ] **Step 6: commit** `app: scan for league less often, skip unchanged lobbies, push champion data`.

### task 3: faster champ select

**Files:**
- Modify: `src-tauri/src/lobby/mod.rs` (constants 41–50, LobbyState 57–93, update 227–247, stats_arrived 253–274, resolve 321–399, chat_due 413, read_chat 439–459, chat_arrived 463–503, tests 1027–1094)
- Modify: `src-tauri/src/lobby/stats.rs` (fetch_stats 310–333)

**Interfaces:**
- `const CHAT_EARLY_EVERY: Duration = 500ms; const CHAT_EARLY_FOR: Duration = 5s; const CHAT_EVERY: Duration = 2s; const CHAT_PATIENCE: Duration = 10s;`
- `fn chat_interval(since_first_read: Duration) -> Duration`: 500 ms before 5 s, then 2 s.
- `LobbyState::chat_started: Option<Instant>` replaces `chat_reads` (set at the first successful read); names are due while `candidates.is_empty() && !chat_failed && chat_started.map_or(true, |t| now - t < CHAT_PATIENCE)`; `chat_due` uses `chat_interval`.
- `stats.rs`: `pub struct Quick` (the summoner and ranked answers) with `pub fn stats(&self) -> PlayerStats`; `pub async fn fetch_quick(client: &LcuClient, puuid: &str, limit: &Semaphore) -> Quick`; `pub async fn fetch_rest(client: &LcuClient, puuid: &str, limit: &Semaphore, quick: Quick) -> Fetched` (adds history and mastery; `complete` as today); `fetch_stats` becomes `fetch_rest(.., fetch_quick(..).await)`. no request is made twice.
- `LobbyState::record_quick(&mut self, puuid: &str, stats: PlayerStats)` stores stats and keeps the puuid pending; `LobbyState::record_full(&mut self, puuid: &str, stats: PlayerStats)` stores and clears pending (today's move from `stats_arrived`); caching stays in `stats_arrived` and still only for complete fetches.
- `update` spawns per puuid: `fetch_quick` → `quick_arrived` (record_quick, publish) → `fetch_rest` → `stats_arrived`.

- [ ] **Step 1: failing tests:** `reads_fast_at_first_then_slower` (`chat_interval(1s) == 500ms`, `chat_interval(6s) == 2s`), `ranks_show_before_history` (after `record_quick` the built view has the rank and `stats_pending == true`; after `record_full`, `stats_pending == false` and history loaded); update `shows_matching_until_there_is_nothing_left_to_try` to set `chat_started = Some(now - 11s)` instead of a read count.
- [ ] **Step 2: run** → fail.
- [ ] **Step 3: implement.**
- [ ] **Step 4: run** checks → PASS (`partial_stats_when_history_fails` still gives `complete == false`).
- [ ] **Step 5: commit** `lobby: read names faster and show ranks before history`.

### task 4: a smaller, safer build

**Files:**
- Create: `src-tauri/src/tls.rs`
- Modify: `src-tauri/Cargo.toml`, `src-tauri/src/main.rs` (first lines of `main`, start_logging 195–210), `src-tauri/src/lcu/websocket.rs:102`, `src-tauri/tauri.conf.json` (build), `package.json` + `package-lock.json`, `.github/workflows/ci.yml` (21–22, 33–36, 56–58), `.github/workflows/release.yml` (55–58)
- Delete: `src/components/ui/scroll-area.tsx`, `src/components/ui/separator.tsx`

**Interfaces:**
- `tls.rs`: `pub fn install_crypto()`: `let _ = rustls::crypto::ring::default_provider().install_default();` (safe to call twice).
- `main.rs`: `tls::install_crypto()` and `log_panics()` run first in `main`; the hook logs `panic: {info}` with `log::error!`, flushes the logger, then calls the previous hook.
- `Cargo.toml`:
  - `reqwest = { version = "0.13", default-features = false, features = ["json", "query", "rustls-no-provider", "gzip"] }`
  - `rustls = { version = "0.23.40", default-features = false, features = ["ring", "logging", "std", "tls12"] }`
  - `tokio-rustls = { version = "0.26.4", default-features = false, features = ["ring", "logging", "tls12"] }`
  - `sysinfo = { version = "0.38.4", default-features = false, features = ["system"] }`
  - `tokio = { version = "1.47", features = ["macros", "net", "rt", "rt-multi-thread", "sync", "time"] }` (add only what `cargo check --all-targets` on both targets asks for)
  - `[profile.release]` with `lto = true`, `codegen-units = 1`, `opt-level = "s"`, `strip = true`, `panic = "abort"`.
- `tauri.conf.json`: `"build": { …, "removeUnusedCommands": true }`.

- [ ] **Step 1: failing test** in `tls.rs`: `clients_build_with_ring` (install twice, `rustls::crypto::CryptoProvider::get_default().is_some()`, `reqwest::Client::builder().build()` is ok).
- [ ] **Step 2: run** → fails (no module).
- [ ] **Step 3: implement** the crate changes, `tls.rs`, the hook and websocket.rs:102 (`rustls::crypto::ring::default_provider()`).
- [ ] **Step 4: run** checks → PASS; `cargo tree --manifest-path src-tauri/Cargo.toml -i aws-lc-rs` → "did not match any packages".
- [ ] **Step 5: measure** `npm run tauri build -- --bundles app`, then `ls -l src-tauri/target/release/expose`: about 5–6 mb (v0.2: 17,708,400 bytes). note both numbers in the commit body.
- [ ] **Step 6: js hygiene:** `npm uninstall @radix-ui/react-scroll-area @radix-ui/react-separator @tauri-apps/plugin-opener @tauri-apps/plugin-store`, delete the two dead files, `npm run build`.
- [ ] **Step 7: commands still reachable:** `npm run tauri dev`, then open settings: "Open logs folder", the scout site and auto-accept all work (they are app commands, kept by `removeUnusedCommands`).
- [ ] **Step 8: ci:** delete the nasm steps and the aws-lc comment.
- [ ] **Step 9: commit** `app: smaller release build, ring for tls, lighter dependencies`.

### task 5: the window, built in rust

the chrome does not change yet (task 10 flips it with the new title bar): both platforms keep their system title bar here, so every commit stays usable.

**Files:**
- Create: `src-tauri/src/window.rs`
- Modify: `src-tauri/src/main.rs` (setup 88–172: the windows block 134–150 and the mac vibrancy block 153–164; on_window_event 77–87; reveal_main_window 227–241), `src-tauri/src/commands.rs`, `src-tauri/src/tray.rs` (toggle_window hides through `window::hide_main`), `src-tauri/Cargo.toml` (remove window-vibrancy; windows-only deps), `src-tauri/tauri.conf.json` (windows[0]), `src-tauri/capabilities/default.json`
- Modify: `src/lib/bridge.ts` (`ready`), `src/dev/preview.ts` (a no-op `ready`), `src/App.tsx` (the ready call)
- unchanged on purpose: the lobby's own `hide_window` (`src-tauri/src/lobby/mod.rs:702`) keeps hiding without saving the window state, as in v0.2.

**Interfaces:**
- `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Frame { Native, Os, Css }` with `pub fn as_str(self) -> &'static str` ("native", "os", "css").
- `pub fn frame_for(os: &str, windows_build: Option<u32>) -> Frame`: "macos" → Native; "windows" with build ≥ 22000 → Os; any other windows → Css.
- `pub fn init_script(platform: &str, frame: Frame) -> String` → exactly `window.__EXPOSE_PLATFORM__="<platform>";window.__EXPOSE_FRAME__="<frame>";`
- `pub fn launch_shows(os: &str, args: &[String]) -> bool`: false only for "windows" with `--autostart`.
- `fn should_first_show(already_shown: bool, launch_shows: bool) -> bool`; `pub fn first_show(app: &AppHandle)` shows the main window once through `show_main_window`, guarded by `static FIRST_SHOWN: AtomicBool`.
- `pub fn build_main_window(app: &tauri::App) -> tauri::Result<tauri::WebviewWindow>`: `WebviewWindowBuilder::from_config(app.handle(), &config)?` with `.initialization_script(&init_script(..))`; mac adds `.title_bar_style(TitleBarStyle::Overlay).hidden_title(true).accept_first_mouse(true)`; windows adds `.minimizable(false)` (decorations stay on until task 10). the windows build number comes from `windows_version::OsVersion::current().build`.
- `pub fn hide_main(app: &AppHandle)`: remember the window state, then hide.
- commands: `#[tauri::command] pub fn hide_main_window(app: AppHandle)` → `window::hide_main`; `#[tauri::command] pub fn frontend_ready(app: AppHandle)` → `window::first_show`.
- `bridge.ready(): Promise<void>` → `invoke("frontend_ready")`.
- `tauri.conf.json` windows[0]: `{"label": "main", "title": "Expose", "create": false, "width": 460, "height": 680, "minWidth": 420, "minHeight": 600, "resizable": true, "fullscreen": false, "maximizable": false, "alwaysOnTop": true, "visible": false, "skipTaskbar": false, "transparent": false, "theme": "Dark", "backgroundColor": "#010a13"}`. `macOSPrivateApi` stays true (no white flash before the first paint).
- capabilities: `["core:default", "core:window:allow-start-dragging", "core:window:allow-minimize", "core:window:allow-start-resize-dragging", "core:window:deny-internal-toggle-maximize"]`.
- windows-only deps: `windows = { version = "0.61", features = ["Win32_Foundation", "Win32_Graphics_Dwm"] }`, `windows-version = "0.1"`.

- [ ] **Step 1: failing tests** in `window.rs`: `frame_follows_the_platform_and_windows_build` (macos → Native; windows 22000 and 22631 → Os; 19045 and None → Css), `init_script_names_platform_and_frame` (the exact string for windows + Css), `login_launch_stays_hidden_only_on_windows`, `first_show_happens_once` (`(false, true)` → true, `(true, true)` → false, `(false, false)` → false).
- [ ] **Step 2: run** → fail.
- [ ] **Step 3: implement** `window.rs` and the config. in setup: `window::build_main_window(app)?` before the tray; the direct launch shows go; spawn a 1.5 s fallback that calls `window::first_show`; the mac vibrancy block and `window-vibrancy` go; the windows `set_theme`, `set_minimizable` and `set_maximizable` calls go (now in config and builder). `CloseRequested` calls `window::hide_main`. register the two commands.
- [ ] **Step 4: ui:** `bridge.ready()` invokes `frontend_ready` (preview: no-op). `App.tsx` calls it once, after the first screen that is not "loading" has committed, `await document.fonts.ready` and one `requestAnimationFrame`.
- [ ] **Step 5: run** checks → PASS. `npm run tauri dev` on mac: the window appears once the first screen has painted, with no white frame; the v0.2 ui on an opaque background; close hides; the dock reopens it; the window remembers its size and position.
- [ ] **Step 6: commit** `app: build the window in rust and show it once painted`.

### task 6: the ui foundation

**Files:**
- Create: `src/assets/fonts/` (woff2 files, `OFL-Cinzel.txt`, `OFL-SourceSans3.txt`), `src/styles/{fonts,tokens,base,type}.css`, `src/lib/platform.ts`, `tests/ui/platform.test.ts`
- Modify: `src/index.css` (all 218 lines), `src/main.tsx:9-11`, `src/vite-env.d.ts`, `package.json` (script), `.github/workflows/ci.yml` (frontend job)

**Interfaces:**
- `platform.ts` (no runtime imports, so node can run it):
  - `export type Platform = "macos" | "windows"; export type Frame = "native" | "os" | "css";`
  - `export function detectPlatform(env: { injected?: unknown; search: string; userAgent: string; preview: boolean }): Platform`
  - `export function detectFrame(env: { injected?: unknown; platform: Platform; search: string; preview: boolean }): Frame`
  - `export const platform: Platform; export const frame: Frame;` (computed from `window.__EXPOSE_PLATFORM__`, `window.__EXPOSE_FRAME__`, `location.search`, `navigator.userAgent`, preview = `!("__TAURI_INTERNALS__" in window)`).
- rules: in the app the injected value wins; in the preview `?platform=mac|macos|windows|win` wins, then the user agent ("Macintosh" → macos). frame: macos → native; windows app → the injected "os" or "css", else css; windows preview → `?frame=os|css`, default css.
- `package.json`: `"test:ui": "node --test \"tests/ui/*.test.ts\""`.

- [ ] **Step 1: failing tests** in `tests/ui/platform.test.ts` (`node:test`, `node:assert/strict`, importing `../../src/lib/platform.ts`) for every rule above.
- [ ] **Step 2: run** `npm run test:ui` → fails.
- [ ] **Step 3: implement** `platform.ts` → pass.
- [ ] **Step 4: fonts.** `npm pack @fontsource/cinzel@5.3.0 @fontsource/source-sans-3@5.3.1` in the scratchpad, extract each into its own folder, and copy:
  - cinzel latin and latin-ext, 700 normal;
  - source sans 3 latin, latin-ext, cyrillic, greek and vietnamese, 400 normal;
  - source sans 3 latin and latin-ext, 400 italic, 600 normal and 700 normal;
  - each package's license as the ofl text.
  `fonts.css` has one `@font-face` per file with the package's `unicode-range`, `font-display: block`, and the metric overrides from `mockup/fonts-free.css`.
- [ ] **Step 5: tokens.css:**
  - port the reference `:root` (4–151) and the tier tints;
  - map the colors the restyle uses in `@theme inline`;
  - add legacy aliases, so v0.2's names keep rendering until task 14 removes them: `canvas`, `surface`, `surface-raised`, `border`, `border-strong`, `fg`, `fg-secondary`, `fg-muted`, `accent`, `accent-hover`, `on-accent`, `red`, `red-bg`, `amber`, `amber-bg`, `green`, `green-bg`, `tier-*`, and the radii at 0 (xs at 2px).
- [ ] **Step 6: base.css:**
  - base (152–171) and the webkit scrollbar (335–340). v0.2's `* { scrollbar-width; scrollbar-color }` goes; in webview2 it would override the webkit rules.
  - the focus rule, and v0.2's reduced-motion block.
  - `[data-platform="windows"][data-frame="css"] #root { border: 1px solid var(--border-window); border-top: 0; box-sizing: border-box }`.
  `type.css` ports 172–204 as `@utility type-*`.
- [ ] **Step 7: index.css** keeps the tailwind and tw-animate imports, imports the four files, and declares `@custom-variant mac (&:where([data-platform="macos"] *));` and `@custom-variant win (&:where([data-platform="windows"] *));`. the old tokens, base, scrollbar and vibrancy blocks go; `@utility skeleton` and the keyframes stay until task 9.
- [ ] **Step 8:** `main.tsx` sets `document.documentElement.dataset.platform` and `.frame` and drops the vibrancy class. `vite-env.d.ts` declares the two window globals.
- [ ] **Step 9: run** `npm run test:ui && npm run build`, then check the preview at `?screen=lobby&platform=windows` and `&platform=mac`: navy and gold, cinzel and source sans (`document.fonts.check('700 14px "Expose Display"')` is true). add `npm run test:ui` to the ci frontend job.
- [ ] **Step 10: commit** `ui: league tokens, fonts, type roles and platform detection`.

### task 7: our drawings and the dev gallery

**Files:**
- Create: `src/components/art/{Mark,glyphs,MetalWork,DodgeShape,Ornament,PositionGlyph,RankCrest}.tsx`, `src/assets/art/{lip-top,lip-bottom,row-diamond}.svg` (copied from the reference mockup, our own drawings), `src/dev/Gallery.tsx`
- Modify: `src/lib/bridge.ts:86` (`PreviewScreen` gains "gallery"), `src/App.tsx` (preview-only route to the gallery)

**Interfaces** (all `aria-hidden`, `currentColor` unless noted, 18 x 18 viewBox unless noted):
- `Mark(props: { size?: number; flat?: boolean; className?: string })`: the app icon's nested diamonds (geometry from `scripts/generate-icon.mjs`), ring gradient, flat #c8aa6e below 14px.
- `glyphs.tsx`: `MinimizeGlyph`, `CloseGlyph`, `GearGlyph`, `GripGlyph`, `WarnGlyph`, `FineGlyph`, `ExactGlyph`, `ExternalGlyph`, `SearchGlyph`, `ClockGlyph`, `UpDownGlyph`, `CheckGlyph`, each `(props: SVGProps<SVGSVGElement>) => JSX.Element`; `RadioGlyph(props: { checked: boolean; className?: string })`.
- `MetalWork(props: { side: "left" | "right" })`, `DodgeShape(props: { className?: string })` (rim and fill paths; the state colors come from css on `.dodge`), `Ornament(props: { className?: string })` (the welcome divider, two mirrored halves meeting at a gap), `PositionGlyph(props: { position: Position | "fill"; className?: string })`, `RankCrest(props: { tier: string; className?: string })` (fills `var(--tier-<t>)` and `var(--tier-<t>-hi)`).
- sources: the `<symbol id="i-…">` elements and the inline dodge svg in `mockup/*-free-fonts.html`; `style-spec.md` section 6 for the position glyphs, ornament and mark; the critique's "rank crest" fix for the crest.

- [ ] **Step 1: implement** the drawings and the gallery: each drawing at 1x and 2x on #010a13, in idle, hover and pressed colors.
- [ ] **Step 2: run** `npm run build`; open `?screen=gallery` and compare each drawing with the reference renders.
- [ ] **Step 3: commit** `ui: our own league-style drawings`.

### task 8: interactive controls

**Files:**
- Create: `src/styles/controls.css` (first part), `src/components/ui/{checkbox,switch,radio-group,select}.tsx`
- Modify: `src/components/ui/{button,icon-button}.tsx`, `src/dev/Gallery.tsx`, `package.json` (+ `@radix-ui/react-select`), and the call sites, only as far as the new apis need: `Settings.tsx`, `Idle.tsx`, `DodgeControl.tsx`, `LobbySummary.tsx`, `PlayerDetail.tsx`, `Welcome.tsx`, `Header.tsx`
- Delete: `src/components/ui/toggle.tsx`, `src/components/ui/segmented.tsx`

**Interfaces:**
- `type ButtonVariant = "framed" | "primary" | "thin" | "text"`; `Button(props: ComponentProps<"button"> & { variant?: ButtonVariant; external?: boolean; asChild?: boolean })`. framed and primary hold `is-clicked` for 600 ms after a click with input blocked; thin for 250 ms.
- `IconButton(props: Omit<ComponentProps<"button">, "aria-label"> & { label: string; variant?: "bare" | "round"; size?: "md" | "sm"; tooltip?: boolean })`.
- `Checkbox(props: { checked: boolean; onCheckedChange: (checked: boolean) => void; label: string; hint?: string; disabled?: boolean })`, a `button role="checkbox"`.
- `Switch(props: ComponentProps<typeof SwitchPrimitive.Root> & { danger?: boolean; glyph?: ReactNode })` (radix switch; green on, red with `danger`).
- `RadioGroup<T extends string | number>(props: { value: T; options: readonly { value: T; label: string }[]; onChange: (value: T) => void; "aria-label"?: string; "aria-labelledby"?: string; disabled?: boolean })`, keeping Segmented's roving tabindex and arrow, Home and End keys.
- `Select<T extends string | number>(props: { value: T; options: readonly { value: T; label: string }[]; onChange: (value: T) => void; "aria-label"?: string; "aria-labelledby"?: string; disabled?: boolean })`, radix select, `.select` trigger and `.select-option` items on #010a13.
- css: reference 205–291, plus the button animations from `style-spec.md` section 3 (text glow, hover glow, press).

- [ ] **Step 1: implement** the primitives and move the call sites over.
- [ ] **Step 2: gallery:** every control in idle, hover, pressed, focus, disabled and checked states.
- [ ] **Step 3: run** `npm run build`; in the gallery, tab through every control and check the focus ring and keyboard use.
- [ ] **Step 4: commit** `ui: league buttons, toggles, radios and dropdown`.

### task 9: display pieces

**Files:**
- Modify: `src/styles/controls.css` (chip, tag, count, status, frame, lips, panel, raised, rules, tooltip, toast, skeleton, loader, empty state, scroll fade), `src/components/ui/{chip,tooltip,empty-state}.tsx`, `src/components/shell/Notices.tsx`, `src/hooks/notices.tsx` (LIFETIME_MS error 7000; at most two), `src/components/shell/Header.tsx` (Pill → Status), `src/screens/{Idle,Settings}.tsx` (Card → Panel), `src/index.css` (skeleton moves to controls.css), `src/dev/Gallery.tsx`
- Create: `src/components/ui/{status,frame,loader}.tsx`
- Delete: `src/components/ui/{pill,card}.tsx`

**Interfaces:**
- `Chip(props: { tone: "red" | "amber" | "neutral" | "info" | "soft"; warn?: boolean; className?: string; children: ReactNode })`; `Tag(props: { tone: "gold" | "amber" | "teal" | "red"; children: ReactNode })`.
- `Status(props: { state: ConnectionStatus | null })`, a dot and a label, with v0.2's labels.
- `Frame(props: ComponentProps<"div"> & { bright?: "bottom" | "top"; lips?: boolean })`; `Panel(props: ComponentProps<"div">)`; `Raised(props: ComponentProps<"div">)`.
- `Loader()`: three bouncing dots, aria-hidden.
- `TooltipProvider` with `delayDuration={450}` and `skipDelayDuration={500}`; `TooltipContent(props: … & { rich?: boolean })` with our caret, `sideOffset={13}`, a 150 ms fade and no zoom or slide; rich tooltips set `delayDuration={250}` on their root.
- `EmptyState(props: { title: string; children?: ReactNode; actions?: ReactNode })` with the flat mark.
- skeleton: a transform shimmer (`::after` translating a gradient) instead of the background-position one.
- toasts: framed with lips, 320px, bottom center, 400 ms fade with `--ease-toast`, no slide.

- [ ] **Step 1: implement** and add the pieces to the gallery.
- [ ] **Step 2: run** `npm run build`. preview `?notice=1`: the toast fades in and out with no slide. tooltips show after 450 ms. block the font files in devtools and reload the gallery and `?screen=lobby`: the layout holds on the fallback stacks (review focus 5).
- [ ] **Step 3: commit** `ui: league chips, frames, tooltips and toasts`.

### task 10: the league title bar and the window chrome per platform

**Files:**
- Create: `src/components/shell/{TitleBar,WindowEdge}.tsx`, `src/styles/shell.css`
- Modify: `src/App.tsx`, `src/lib/bridge.ts`, `src/dev/preview.ts`, `src-tauri/src/window.rs`, `src-tauri/src/main.rs` (on_window_event)
- Delete: `src/components/shell/Header.tsx`

**Interfaces:**
- `TitleBar(props: { status: ConnectionStatus | null; update: UpdateInfo | null; settingsOpen: boolean; onToggleSettings: () => void; minimal?: boolean })`.
- `WindowEdge()`: windows only; renders the corner grip, which calls `bridge.startResize()`.
- `bridge.hideWindow(): Promise<void>` → `invoke("hide_main_window")`; `bridge.minimizeWindow(): Promise<void>` → mac `getCurrentWindow().minimize()`, windows `hideWindow()`; `bridge.startResize(): Promise<void>` → `getCurrentWindow().startResizeDragging("SouthEast")`. the preview notes each call.
- rust:
  - `#[cfg(target_os = "macos")] pub fn hide_traffic_lights(window: &WebviewWindow)` hides `CloseButton`, `MiniaturizeButton` and `ZoomButton` through `NSWindow::standardWindowButton(..).setHidden(true)` on the main thread. the mac-only dep is `objc2-app-kit = { version = "0.3.2", features = ["NSWindow", "NSButton", "NSControl", "NSView", "NSResponder"] }`; add features if the compiler asks.
  - `#[cfg(windows)] fn square_corners(window: &WebviewWindow)` calls `DwmSetWindowAttribute` with `DWMWA_WINDOW_CORNER_PREFERENCE` = `DWMWCP_DONOTROUND`, and `DWMWA_BORDER_COLOR` = `COLORREF(0x002D281E)`.
  - `build_main_window` now gives windows `.decorations(false).shadow(frame == Frame::Os)`, calling `square_corners` for `Os`. mac calls `hide_traffic_lights` after build, and again on `WindowEvent::Resized` and `WindowEvent::Focused(true)`.
- behavior:
  - the strip is 41px: the gold line, `Mark` and "Expose" in `type-nav`, `Status`, then minimize, settings and close.
  - update dot: on the gear when `update?.available`. the gear gets `aria-pressed` while settings is open and stays a gear.
  - drag: `data-tauri-drag-region` on the strip on both platforms, buttons excluded. the noZoom guard runs on both.
  - `App`:
    - ⌘, on mac and ctrl+, on windows toggle settings.
    - the first screen renders without the fade.
    - `WindowEdge` shows when `platform === "windows"`.

- [ ] **Step 1: implement** the ui and the rust chrome.
- [ ] **Step 2: run** checks → PASS, with the windows cross-check covering the dwm code.
- [ ] **Step 3: preview** `?platform=mac` and `?platform=windows&frame=css` at 460 x 680 and 420 x 600: all three buttons are visible, the windows border shows, the grip sits in the corner.
- [ ] **Step 4: mac app** (`npm run tauri dev`): no traffic lights; minimize goes to the dock; the dock icon brings the window back with the lights still hidden; settings toggles; close hides and the dock reopens; ⌘, toggles settings; dragging the strip moves the window; a double-click does not zoom; no white flash at launch.
- [ ] **Step 5: commit** `ui: league title bar and window chrome per platform`.

### task 11: the lobby band, verdict and dodge

**Files:**
- Create: `src/lib/band.ts`, `tests/ui/band.test.ts`, `src/components/lobby/Band.tsx`
- Modify: `src/types/index.ts` (LobbyView 30–46, PlayerView 48–71), `src/dev/fixtures.ts` (TIMERS 397–403; ranked becomes stage "ban", acting "both", total 30000, as in the mockup; a new variant "pick" with cell 1 acting), `src/dev/preview.ts` (`?lobby=pick`), `src/screens/Lobby.tsx` (PhaseClock 100–137 and useTurnLength 142–165 give way to `Band`), `src/components/lobby/{LobbySummary,DodgeControl}.tsx`, `src/styles/lobby.css` (band, summary, dodge: 368–427)

**Interfaces:**
- types: `export type Stage = "unknown" | "planning" | "ban" | "pick" | "finalization"; export type Acting = "none" | "ally" | "enemy" | "both";` `LobbyView` gains `stage: Stage; acting: Acting; totalMs: number`; `PlayerView` gains `acting: boolean`.
- `band.ts` (only `import type` imports):
  - `export type BarMode = "teal" | "red" | "hidden"`
  - `export function formatBandClock(leftMs: number, totalMs: number): string`: `total = totalMs > 0 ? totalMs : leftMs`. under 100000, `String(Math.ceil(leftMs / 1000))`. otherwise m:ss, computed inline the same way `formatClock` does.
  - `export function barModes(stage: Stage, acting: Acting, armedFinal: boolean): { left: BarMode; right: BarMode }`. bar colors per case:

    | case | left | right |
    |---|---|---|
    | `armedFinal` | red | red |
    | planning, finalization, unknown | teal | teal |
    | ban, both | red | red |
    | ban, ally | red | hidden |
    | ban, enemy | hidden | red |
    | pick, ally | teal | hidden |
    | pick, enemy | hidden | red |
    | pick, both | teal | red |
    | ban or pick, none | teal | teal |
- `Band(props: { lobby: LobbyView; receivedAt: number })`:
  - phase line: `phaseLabel(phase)` in `type-phase`, or `type-phase-turn` when the local player is acting.
  - the clock and two bars filling `left / total`, with `MetalWork` on both sides and the queue name below.
  - `is-final` (pulse) in the last 10 s.
  - while armed in finalization, a 2px red tick on each bar at `leadMs / total` from the clock end.
  - sticky, with the fade once stuck (keep `useStuck`).
- summary: `Frame` bright at the bottom, with lips.
  - head strip by tone: red is the banner gradient plus a 6px #ff4545 stripe; amber is the same in amber; neutral and clear have no strip, and clear gets the green `FineGlyph`.
  - title in `type-h5` with .02em tracking; detail in `type-body`, plus `Loader` while still checking.
  - the round scout `IconButton` with `SearchGlyph`.
  - foot: `DodgeControl`.
- dodge: `.dodge` with `DodgeShape`, labels "Dodge", "Confirm", "Dodging…", and the 3 s drain line while confirming. `CONFIRM_MS`, `CONFIRM_GUARD_MS` and the key handling stay. `Switch danger` with `ClockGlyph`. its label:
  - "Last second" in `type-label` #a09b8c;
  - while armed, the countdown style (display 12/16 700 .075em uppercase): "Dodges at …" in #c89b3c; "Dodges in …" and "Dodging now" in #ff2345.

- [ ] **Step 1: failing tests** in `tests/ui/band.test.ts`:
  - `formatBandClock`: (26000, 30000) → "26"; (26400, 30000) → "27"; (0, 30000) → "0"; (95000, 120000) → "1:35"; (5000, 0) → "5"; (125000, 0) → "2:05".
  - `barModes`: every row of the table above.
- [ ] **Step 2: run** `npm run test:ui` → fails; implement `band.ts` → passes.
- [ ] **Step 3: implement** the types, fixtures and components.
- [ ] **Step 4: run** `npm run test:ui && npm run build`. preview on both platforms at 460 and 420:
  - `?lobby=ranked`: both bars red.
  - `?lobby=pick`: left bar teal, right hidden.
  - `?lobby=normal`: finalization, teal.
  - `?lobby=normal`, then arm the last-second dodge: both bars red, the tick, the countdown label. at 420 the label never pushes the dodge button (review focus 2).
  - a full confirm and dodge with the keyboard.
- [ ] **Step 5: commit** `lobby: league band, verdict and dodge`.

### task 12: player rows and details

**Files:**
- Modify: `src/components/lobby/{PlayerRow,PlayerDetail,RankBadge,FormDots,ChampionIcon}.tsx`, `src/styles/lobby.css` (rows 429–480; the details panel from `style-spec.md` section 5), `src/dev/fixtures.ts` + `src/dev/preview.ts` (variants "long" and "aram")

**Interfaces:**
- `ChampionIcon(props: { championId: number; champions: ChampionData | null; size: "portrait" | "square"; ring?: "teammate" | "self" | "acting"; position?: Position | null; className?: string })`:
  - portrait: 44px, round, a gradient ring; with no pick yet, a #1e282d disc with the `PositionGlyph`.
  - square: 20px, with a gold-6 edge.
- `RankBadge(props: { rank: RankView | null; unranked: boolean })`: `RankCrest`, the tier in #cdbe91, the lp in #a09b8c, "Unranked" in `.rank-none`.
- `FormDots(props: { games: GameView[]; limit?: number })`: 6px dots, 2px apart; wins #0acbe6, losses #be1e37.
- `PlayerRow` and `PlayerDetail` keep their v0.2 props.
- row lines:
  1. the role, with the rank on the right;
  2. the champion in display caps with "· N games", and the dots and kda on the right;
  3. the alias and the riot id, with the exact glyph or a "Likely" `Tag`; "Name hidden" and "Matching name" in italic with `Loader`;
  4. flags (`Chip`), or "Looks fine", or your season line, and the details triangle.
- each line has a fixed height, and skeletons share it, so rows never jump. stats that are present but still pending show the rank and a dots skeleton.
- your own row: #fabe0a text. the acting bar: 3px, teal, amber on your row.
- dividers: `li::before` with `row-diamond.svg` and the fading line. the row glows on hover and while open.
- details: `Raised` with a caret toward the portrait.
  - facts (level, ranked this season, on this champion) split by vertical rules.
  - role bars for all five positions, with `PositionGlyph` and the percentage; the slot is highlighted.
  - recent games in two columns.
  - "View on {site}" as a thin `Button` with `external`.
- fixtures:
  - "long": riot id "Wwwwwwwwwwwwwwww#WWWWW", Nunu & Willump (id 20), GRANDMASTER 1234 LP, three red flags.
  - "aram": queue 450, no positions, stage "unknown", acting "none".

- [ ] **Step 1: implement.**
- [ ] **Step 2: run** `npm run build`. preview every lobby variant on both platforms at 460 x 680 and 420 x 600: ranked, guess, resolving, hidden, normal, live, pick, long and aram. check each row's line heights before and after stats land (`?lobby=live`), and open the details on one row per variant.
- [ ] **Step 3: commit** `lobby: league player rows and details`.

### task 13: waiting, idle, welcome, settings and riot's notice

**Files:**
- Create: `src/components/shell/{RiotNotice,FontCredits}.tsx`, `src/styles/screens.css` (settings 481–499 plus the welcome and idle recipes from `style-spec.md` section 3)
- Modify: `src/screens/{Waiting,Idle,Welcome,Settings}.tsx`, `README.md:157`

**Interfaces:**
- `export const RIOT_NOTICE: string`, the exact text; `RiotNotice(props: { className?: string })`.
- `FontCredits()`:
  - "Cinzel, Copyright 2020 The Cinzel Project Authors (https://github.com/NDISCOVER/Cinzel)".
  - "Source Sans 3, © 2023 Adobe (http://www.adobe.com/), with Reserved Font Name ‘Source’".
  - "Both under the SIL Open Font License 1.1."
  - a disclosure that shows both license texts, imported with `?raw`.
- settings:
  - the breadcrumb title "Expose / Settings", the current part brightest, with a 1px rule under it. section titles in `type-h5`.
  - `Checkbox` rows with v0.2's labels and hints (label tracking .025em).
  - scout site: `RadioGroup`. lead time: `Select`, with "1 s", "1.5 s", "2 s", "3 s".
  - about:
    - the version and its hint, plus a thin "Get {latest}" `Button` with `external` when an update exists;
    - "Open logs folder" as a thin button;
    - `RiotNotice` and `FontCredits`.
- welcome:
  - `Frame` with lips, 388px at most. `Mark` at 40px, "Welcome to Expose" in `type-h3`, the `Ornament` under it, the subtitle in `type-label`.
  - three feature rows: our 24px glyphs in #cdbe91 (search, check, warn), titles in `type-h5`, text in `type-body-m`.
  - "Get started": a primary button sitting on the frame's bottom edge, between 2px gold posts.
  - `RiotNotice` in small type at the bottom.
- idle:
  - the auto-accept `Frame` with lips: title, hint and a `Switch`.
  - v0.2's next-step line.
  - the last lobby: a subhead with the time ago, and a `Panel` holding the queue, the verdict `Chip` and the names.
- waiting: `EmptyState` with "Open League of Legends", v0.2's text and `Loader`.
- readme line 157 becomes the notice, case kept.

- [ ] **Step 1: implement.**
- [ ] **Step 2: run** `npm run build`. preview both platforms at 460 and 420:
  - `?screen=welcome`, `?screen=waiting`, `?screen=idle` (before and after a dodge, so the last lobby shows);
  - `?screen=settings` with `?update=0`, `?update=failed` and the default update.
- [ ] **Step 3: commit** `ui: league welcome, waiting, idle and settings, with riot's notice`.

### task 14: drop v0.2's leftovers

**Files:**
- Modify: `src/styles/tokens.css` (the legacy aliases go), any component still using legacy classes, `package.json` (`lucide-react`, `tw-animate-css` and `class-variance-authority` go if nothing imports them)

- [ ] **Step 1:** grep `src/` for the legacy names (`bg-canvas`, `text-fg`, `border-border`, `bg-accent`, `rounded-card`, `rounded-control`, `text-md`, `bg-surface`, `lucide-react`, `animate-in`, `cva(`), and replace each with the new classes and drawings.
- [ ] **Step 2:** remove the aliases and the unused packages. run `npm run build`, and note the js bundle size before and after.
- [ ] **Step 3:** preview every screen once more on both platforms to confirm nothing lost its style.
- [ ] **Step 4: commit** `ui: drop the v0.2 tokens and unused packages`.

### task 15: windows webview memory while hidden

**Files:**
- Modify: `src-tauri/src/window.rs`, `src-tauri/src/main.rs` (reveal_main_window), `src-tauri/Cargo.toml` (windows-only `webview2-com = "0.38"`)

**Interfaces:**
- `#[cfg(windows)] pub fn set_memory_target(window: &WebviewWindow, low: bool)`: through `with_webview`, the controller's `CoreWebView2` is cast to `ICoreWebView2_19`, then `SetMemoryUsageTargetLevel(COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW or _NORMAL)`. errors are logged and ignored.
- `window::hide_main` and the lobby's hide call it with `true`; `reveal_main_window` with `false`.

- [ ] **Step 1: implement.**
- [ ] **Step 2: run** checks → PASS (the windows cross-check compiles it).
- [ ] **Step 3: commit** `app: lower webview memory while hidden on windows`.

### task 16: verify everything

- [ ] **Step 1:** `zsh .claude/design/checks.sh` → PASS.
- [ ] **Step 2: screenshots** with `.claude/design/shoot.mjs`, adapted to add `&platform=`, into the scratchpad: every screen and lobby variant on both platforms at 460 x 680 and 420 x 600. compare them with the reference renders and fix what differs.
- [ ] **Step 3: mac release build.**
  - `npm run tauri build -- --bundles app`; record the binary size.
  - launch: no white flash.
  - with league closed, `ps -o %cpu= -p <pid>` sampled for 60 s stays about 0.
  - review focus 3: minimize and restore from the dock, close and reopen from the dock, focus away and back, resize, move to another display. the lights stay hidden and our buttons work.
- [ ] **Step 4: live league on the mac, if the owner runs a game there:**
  - tls through ring: the status reaches "connected" and lobby events arrive.
  - in champ select, the band colors match the stage, and the log shows `lobby: stage=… acting=…`.
- [ ] **Step 5:** write the owner's windows checklist (review focus 4) into the final report.
- [ ] **Step 6:** a whole-branch review by a fresh reviewer; fix what it confirms; rerun steps 1–2.
- [ ] **Step 7:** update the memory note (`expose-v0.2.md`) with the restyle's state and the unverified live items.

## self-review

- spec coverage: look (tasks 6–9, 11–14), shells (5, 10), stage and turn (1, 11), every screen (10–13), optimization table (2, 3, 4, 5's first show, 12's steady rows, 15), policy (13's notice and credits, the global constraints), checks (16).
- departures from the design, both small: the libproc listing is dropped because the 30 s interval already takes the scan to about 0.05% of a core (task 2), and the mac and windows chrome flips in task 10 rather than task 5, so every commit stays usable.
- frontend tasks rely on the build, the two pure-logic test files and visual checks, as in v0.2.
