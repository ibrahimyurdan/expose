# Real rank emblems and role icons Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** show the league client's own rank crests, rank emblems and role icons in expose, read at runtime from the player's running client, with expose's drawings as the fallback.

**Architecture:** a custom protocol, `league-art`, registered in rust, answers the page's image requests for a fixed list of the client's static asset files: it fetches each from the connected client's local server with the existing credentials and keeps it in memory for the app's run. the ui builds each image url per platform and shows its own drawing until the image has loaded, or for good when it can't.

**Tech Stack:** tauri 2.11 (`register_asynchronous_uri_scheme_protocol`), rust with reqwest (the existing `LcuClient`), react 19 and typescript, node's test runner for the pure ui helpers.

**Spec:** docs/superpowers/specs/2026-11-22-expose-real-rank-emblems-design.md

## Global Constraints

- nothing of riot's goes into the repo or the bundle: the images come only from the player's running client, through `league-art`.
- the page never sees the client's port or token, and `league-art` answers get requests for the allowed files only; everything else is a 404 without contacting the client.
- with league closed, or a file the client doesn't serve, every icon shows expose's drawing at the same size: no broken image, no gap, no layout shift.
- the four row lines keep their heights (14, 20, 16 and 22 px).
- code comments and docs: lowercase, plain, no emojis.

## Review Focus

1. league closed or disconnected while a lobby is shown: every crest, emblem and role icon falls back to the drawing, with no broken-image marks. checks: task 2's preview audit (the preview has no rust side, so every icon falls back) and task 4 step 3 (close league: the drawings come back).
2. a tier string the app doesn't know (a new tier, "NONE", an empty string): no file, so the drawing shows; rust refuses the name too. tests: task 2 `an unknown tier has no file`, task 1 `refuses_everything_else`.
3. paths that try to reach more of the client (`..`, percent-encoded dots, upper case, other folders, a trailing slash) or another method: a 404 with no request to the client. task 1 `refuses_everything_else` and `only_get_is_served`.
4. the windows url form (`http://league-art.localhost/...`) and the content security policy: an image the policy blocks would quietly stay on the fallback forever. tests: task 2 `the art url follows the platform` and `the content security policy lets both forms load`.
5. the details' rank line at 420 px with the longest text ("Grandmaster 1234 LP", "Ranked solo/duo · 212W 187L · 53%"): nothing cut or pushed out. task 3's preview check with `?lobby=long`.

---

### Task 1: the league-art protocol

**Files:**
- Create: `src-tauri/src/art.rs`
- Modify: `src-tauri/src/lcu/client.rs`, `src-tauri/src/state.rs`, `src-tauri/src/main.rs`, `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces:
  - `pub const SCHEME: &str = "league-art";` the page asks for `league-art://localhost/<file>` on macos and `http://league-art.localhost/<file>` on windows.
  - the allowed `<file>`s, exactly:
    - `images/ranked-mini-crests/<t>.svg` for t in iron, bronze, silver, gold, platinum, emerald, diamond, master, grandmaster, challenger, unranked;
    - `images/ranked-emblem/emblem-<t>.png` for the ten tiers (no unranked);
    - `svg/position-<p>.svg` and `svg/position-<p>-light.svg` for p in top, jungle, middle, bottom, utility.
  - `pub struct Art { pub lcu_path: String, pub content_type: &'static str }` and `pub fn route(get: bool, path: &str) -> Option<Art>`: `path` is the request uri's path (leading slash); `lcu_path` is `/fe/lol-static-assets/<file>`; svg is `image/svg+xml`, png `image/png`.
  - `pub fn serve(ctx: tauri::UriSchemeContext<'_, tauri::Wry>, request: tauri::http::Request<Vec<u8>>, responder: tauri::UriSchemeResponder)`.
  - `LcuClient::static_asset(&self, path: &str) -> Result<Vec<u8>>`.
  - `AppState::art: Mutex<HashMap<String, Vec<u8>>>` (parking_lot, like the other fields), keyed by `lcu_path`.

- [ ] **Step 1: write the failing tests in `src-tauri/src/art.rs`**

```rust
#[test]
fn serves_the_known_crests_emblems_and_role_icons() {
    let crest = route(true, "/images/ranked-mini-crests/gold.svg").expect("a crest");
    assert_eq!(crest.lcu_path, "/fe/lol-static-assets/images/ranked-mini-crests/gold.svg");
    assert_eq!(crest.content_type, "image/svg+xml");
    let emblem = route(true, "/images/ranked-emblem/emblem-diamond.png").expect("an emblem");
    assert_eq!(emblem.content_type, "image/png");
    assert!(route(true, "/images/ranked-mini-crests/unranked.svg").is_some());
    assert!(route(true, "/svg/position-utility-light.svg").is_some());
}

#[test]
fn refuses_everything_else() {
    for path in [
        "", "/", "/lol-summoner/v1/current-summoner",
        "/images/ranked-mini-crests/../../lol-summoner/v1/current-summoner",
        "/images/ranked-mini-crests/%2e%2e/gold.svg", "/images/ranked-mini-crests/Gold.svg",
        "/images/ranked-mini-crests/gold.png", "/images/ranked-mini-crests/gold.svg/",
        "/images/ranked-emblem/emblem-unranked.png", "/images/ranked-emblem/emblem-wood.png",
        "/svg/position-fill.svg", "/svg/position-top-red.svg",
    ] {
        assert!(route(true, path).is_none(), "{path}");
    }
}

#[test]
fn only_get_is_served() {
    assert!(route(false, "/images/ranked-mini-crests/gold.svg").is_none());
}
```

- [ ] **Step 2: run them to see them fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml art::`
Expected: FAIL to compile, `route` is not defined.

- [ ] **Step 3: implement `route` in `src-tauri/src/art.rs`** as an exact match against the three file forms and their name lists (no regex, no decoding: anything not spelled exactly is refused).

- [ ] **Step 4: implement `serve`**: it spawns on `tauri::async_runtime`, answers 404 when `route` refuses or no client is connected (`AppState::client`), 200 from `AppState::art` when cached, else fetches with `static_asset`, caches and answers 200 with `Content-Type` and `Cache-Control: max-age=86400`. a failed fetch is a 404 and one `log::warn!("art: {lcu_path} not served: {error}")`. add `static_asset` to `client.rs` beside `get_json`, with the same status check, returning the body's bytes.

- [ ] **Step 5: register it and open the policy**: `.register_asynchronous_uri_scheme_protocol(art::SCHEME, art::serve)` on the builder in `main.rs` (with `mod art;`); in `tauri.conf.json`, `img-src 'self' data: https://ddragon.leagueoflegends.com league-art: http://league-art.localhost`.

- [ ] **Step 6: run the tests and the checks**

Run: `cargo test --manifest-path src-tauri/Cargo.toml art::` then `zsh .claude/design/checks.sh`
Expected: the three art tests pass; `== result: PASS`.

- [ ] **Step 7: commit** `app: serve the league client's rank and role art to the page`.

### Task 2: the client's crests and role icons in the ui

**Files:**
- Create: `src/lib/art.ts`, `src/components/art/LeagueArt.tsx`, `tests/ui/art.test.ts`
- Modify: `src/lib/bridge.ts`, `src/dev/preview.ts`, `src/components/lobby/RankBadge.tsx`, `src/components/lobby/ChampionIcon.tsx`, `src/components/lobby/PlayerDetail.tsx` (the role bars), `src/styles/lobby.css`

**Interfaces:**
- Consumes: task 1's url forms and file names.
- Produces:
  - `src/lib/art.ts` (only type imports, so node tests it): `crestFile(tier: string): string | null`, `emblemFile(tier: string): string | null`, `roleFile(position: Position, light?: boolean): string`, `artUrl(file: string, platform: Platform): string`.
  - `bridge.leagueArt(file: string): string | null`: `artUrl` for the app's platform; the preview's bridge returns null.
  - `artUrl` builds the url itself rather than with tauri's `convertFileSrc` (the spec's suggestion): `convertFileSrc` percent-encodes the slashes, and task 1's `route` matches exact, unencoded paths.
  - `LeagueArt({ file, className, fallback }: { file: string | null; className?: string; fallback: ReactNode })`: shows `fallback` until the image has loaded, and keeps it when `file` or the url is null or the image fails.

- [ ] **Step 1: write `tests/ui/art.test.ts`**

```ts
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
  assert.equal(artUrl("svg/position-top.svg", "macos"), "league-art://localhost/svg/position-top.svg");
  assert.equal(artUrl("svg/position-top.svg", "windows"), "http://league-art.localhost/svg/position-top.svg");
});
test("the content security policy lets both forms load", () => {
  const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
  const img = conf.app.security.csp.split(";").find((part: string) => part.trim().startsWith("img-src"));
  assert.ok(img.includes("league-art:") && img.includes("http://league-art.localhost"));
});
```

- [ ] **Step 2: run them to see them fail**

Run: `node --test tests/ui/art.test.ts`
Expected: FAIL, `src/lib/art.ts` does not exist.

- [ ] **Step 3: implement `src/lib/art.ts`, the bridge entry (and its preview twin), and `LeagueArt`.**

- [ ] **Step 4: use them**:
  - `RankBadge`: the crest is `LeagueArt` of `crestFile(tier)` with `RankCrest` as the fallback, both 14 px (`.rank-crest`); "Unranked" gets the unranked crest before it, with no fallback.
  - `ChampionIcon`'s empty disc and the details' role bars: `LeagueArt` of `roleFile(position)` with `PositionGlyph` as the fallback; in the role bars the slot's own role uses `roleFile(position, true)`.

- [ ] **Step 5: run the tests, the build and the preview audit**

Run: `npm run test:ui && npm run build`, then the screenshot audit (`node <scratchpad>/shoot-all.mjs <out> <profile>`).
Expected: all ui tests pass; the build passes; 72/72 shots ok, and every crest and role icon in the preview is the drawing (no rust side), each row 94 px.

- [ ] **Step 6: commit** `lobby: the client's own rank crests and role icons`.

### Task 3: the rank emblem in a player's details

**Files:**
- Modify: `src/components/lobby/PlayerDetail.tsx`, `src/styles/lobby.css`

**Interfaces:**
- Consumes: `LeagueArt`, `emblemFile`, `crestFile` (task 2); `tierName`, `isApexTier`, `formatPercent` (`src/lib/format.ts`).

- [ ] **Step 1: add the rank line as the details' first section**: `LeagueArt` of `emblemFile(tier)` in a 40 px box (fallback: `RankCrest` at 28 px, centered), beside two lines:
  - ranked: `{tierName} {division} · {lp} LP` in `type-value-strong` (no division for master and above), then `Ranked solo/duo · {wins}W {losses}L · {percent}`, or `Ranked solo/duo · No games`, in `type-body`;
  - unranked: the unranked crest (no fallback), `Unranked`, then `Ranked solo/duo`;
  - unknown (no rank, not unranked): no art, `Rank unknown`, then `Ranked solo/duo`.
- [ ] **Step 2: the facts row keeps `Level` and `On {champion}`**; its "Ranked this season" fact goes, since its numbers moved up.
- [ ] **Step 3: run** `npm run build`; preview `?lobby=ranked|long|aram|hidden` at 420x600 and 460x680 with the details open: nothing cut in the rank line or the facts; the shoot-all audit stays 72/72.
- [ ] **Step 4: commit** `lobby: the rank emblem in a player's details`.

### Task 4: check it against a running client

needs the owner: league open on the mac, signed in (a custom game's champ select is enough for the rank crest and emblem; role icons need a lobby with positions).

- [ ] **Step 1:** with the lockfile (`/Applications/League of Legends.app/Contents/LoL/lockfile`) read the port and token, and fetch every allowed file from `https://127.0.0.1:<port>/fe/lol-static-assets/<file>`.
  Expected: 200 for all 31, with svg and png bodies. if the prefix is wrong, find the right one, change `route`'s mapping and its test, and rerun task 1's tests.
- [ ] **Step 2:** fetch them into the scratchpad (never the repo) and render them on the app's background at their sizes (crest 14 px, emblem 40 px, role icons as drawn) in the browser pane; compare with the drawings.
- [ ] **Step 3:** run `npm run tauri dev` with league open; in the custom game's champ select, the log has no `art: … not served` lines, and the owner sees the real crest and emblem. close league: the drawings come back. also check the restyle's open items: the status reaches connected (tls through ring), and the log shows `lobby: stage=… acting=…`.
- [ ] **Step 4:** record what was seen; commit only if a change was needed (`app: the client's art prefix`).
