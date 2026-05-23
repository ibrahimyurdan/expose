# expose

a small companion for the league of legends client. it does three things:

- **shows your teammates** during champ select. in ranked, the client hides teammate names (anonymous champ select); expose surfaces them anyway.
- **auto-accepts queue ready checks**, with a toggle you can turn off.
- **dodges champ select** with one click, so you can bail a lobby without force-quitting the whole client.

on macos it lives in the dock; on windows in the notification area. no account login and no in-game overlay.

## features in detail

**teammates.** in normal, draft, and bot games the champ-select session carries real puuids, so expose shows a full card per teammate: role, current champion, level, and a region-correct op.gg link. in ranked, riot's anonymous champ select strips identities from that session, so expose instead reads the riot client chat service (a separate local api that still knows who is who) and surfaces each teammate's name and op.gg link — names only, because the chat gives no way to map a name back to a champ-select cell (so no role or champion). enemy identities are never available (riot zeroes them) and are out of scope. a **scout all** button opens every teammate on op.gg at once via a multi-search.

**auto-accept.** when a ready check pops and the toggle is on, expose accepts it. the toggle state is saved between launches.

**dodge.** a dodge button appears during champ select. it takes two clicks (arm, then confirm) so it can't fire by accident, and leaves champ select through the client api without closing the client. the normal dodge penalty (lp loss, queue lockout) still applies; expose only saves you the force-quit.

see [scope](#scope) below for what is intentionally left out.

## install

prebuilt installers are published on the [releases page](https://github.com/ibrahimyurdan/expose/releases):

- macos: download the `.dmg`, open it, and drag expose to applications.
- windows: download the `.msi` and run it.

the app is not code signed yet, so the first launch shows a warning:

- macos: right click the app and choose open, or run `xattr -dr com.apple.quarantine "/Applications/Expose.app"`.
- windows: on the smartscreen prompt choose "more info" then "run anyway".

after launch, expose lives in the tray. left click the tray icon to show or hide the window; right click for a small menu.

## build from source

prerequisites:

- rust (stable) and cargo
- node 20 or newer and npm
- the platform tauri prerequisites: xcode command line tools on macos, the webview2 runtime and the msvc build tools on windows. see the tauri prerequisites guide for details.

then:

```bash
npm install            # install frontend deps and the tauri cli
npm run tauri dev      # run the app in development
npm run tauri build    # produce a release bundle for the current platform
```

to run the backend checks:

```bash
npm run build                                   # type check and bundle the frontend
cargo test --manifest-path src-tauri/Cargo.toml # run the rust unit tests
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
```

## release & signing

tagging `v*.*.*` triggers `.github/workflows/release.yml`, which builds the macos universal `.dmg` and the windows `.msi` and attaches them to a draft github release.

to skip the macos "unidentified developer" warning, set these repository secrets and the workflow will sign + notarize automatically:

- `APPLE_CERTIFICATE` — base64-encoded `Developer ID Application` `.p12`
- `APPLE_CERTIFICATE_PASSWORD` — the p12 password
- `APPLE_SIGNING_IDENTITY` — for example `Developer ID Application: Your Developer Name (TEAMID)`
- `APPLE_ID` — your apple id email
- `APPLE_PASSWORD` — an app-specific password from appleid.apple.com
- `APPLE_TEAM_ID` — your apple developer team id

without these secrets the workflow still runs and ships an unsigned build. an in-panel notice appears in the app when a newer github release is available.

## how it works

the rust backend connects to two local apis: the **league client** (lcu) for champ select, matchmaking, and dodging, and the **riot client** for the chat service that still reports real names during anonymous ranked champ select. each advertises a random port and an auth token via a lockfile and the league process command line; expose finds them either way and connects over the self-signed https and websocket endpoints (accepted only because the host is always 127.0.0.1).

a single supervisor task owns the connection lifecycle: it detects the client (a 2 second poll plus a file watcher on the lockfile directory), subscribes to two events, dispatches them to the features, and reconnects when the client comes and goes.

```
src-tauri/src/
  main.rs            builder, plugins, tray, window, background tasks
  error.rs           one error type for the whole backend
  models.rs          serde shapes for lcu payloads and emitted events
  state.rs           shared state (toggle, caches, status)
  commands.rs        frontend commands (status, toggle, dodge, open links)
  ddragon.rs         champion id to name and icon lookup
  tray.rs            windows notification-area icon (macos uses the dock)
  lcu/
    auth.rs          credential discovery for the league + riot clients
    client.rs        https client (league client and riot client chat)
    websocket.rs     event subscription and parsing
    mod.rs           connection supervisor
  features/
    teammates.rs     resolve teammates (session, or riot chat when hidden)
    auto_accept.rs   accept ready checks
```

the frontend is react, typescript, vite, tailwind, and shadcn/ui, styled as a dark hextech panel. it subscribes to backend events and renders the teammate cards and the auto-accept toggle. champion icons come from riot's data dragon cdn.

## scope

dodging is manual (a button you press), never automatic. deliberately out of scope: enemy team identities (riot does not expose them), draft or matchup analytics, win probability, any automation of in-game actions, and platforms other than macos and windows.

## disclaimer

this project is not affiliated with or endorsed by riot games.
