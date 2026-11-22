# expose

a small companion for the league of legends client that helps you spot a troll lobby before the game starts. it does three things:

- **scouts your team** during champ select: each teammate's rank, recent games, kda and experience on their champion, with red flags (autofilled, a loss streak, a first-time champion, a bad kda, signs of a bought account) and a one-line verdict for the whole team. in ranked solo/duo it also tells you who is behind "gromp", "murk wolf" and the other hidden names.
- **auto-accepts queue ready checks**, with a toggle you can turn off.
- **dodges champ select** with two clicks, or by itself just before the lobby locks in, so you can leave a lobby without force-quitting the whole client.

on macos it lives in the dock; on windows in the notification area. no account login, no riot api key, no server of its own and no in-game overlay: everything comes from the league and riot clients on your own computer.

## risk & terms of use

expose talks to riot's local **lcu** and **riot client** apis. these are unofficial and unsupported — riot can change or remove them at any time, and using them is not endorsed.

the ranked teammates feature is the part to weigh. anonymous champ select hides teammate names on purpose; expose recovers them from a separate local api that still reports them, and matches them to their slots, so it defeats a deliberate riot feature. that is a stronger terms-of-use concern than auto-accept or dodge and may carry some account risk. it is on by default and can be turned off under settings → **reveal and match ranked teammates**. auto-accept and dodge are ordinary client conveniences, but are still automation of client actions.

there is no way to know riot's enforcement intent for certain. **use expose at your own risk.**

## features in detail

**champ select.** when champ select starts, expose comes forward with one row per player, you included, in pick order:

- the slot: champion, role, and the hidden name in ranked ("gromp", "krug", …) or the riot id elsewhere.
- who they are: the riot id behind a hidden name, marked exact or "likely".
- their solo/duo rank, the last 10 results as dots, kda, and games and mastery on the champion they picked.
- red flags in red (strong) and amber (mild), or a quiet "looks fine". click a row for the last 10 games, the role split, level and season record, and a link to op.gg or u.gg.

above the rows, a summary card reads the whole team ("3 red flags on 2 teammates", "no red flags") and holds the dodge controls. it never sounds surer than the data: while names or stats are still loading it says so, and a teammate whose match history could not be loaded does not count toward an all-clear. a **scout** button opens your whole team on op.gg or u.gg at once.

the flags, from the last 20 summoner's rift games in ranked and normal queues:

| flag | when |
| --- | --- |
| autofilled | the slot is autofilled, or (with 8 or more games) the role is not one of the player's two most played |
| lost last n | 3 or more losses in a row |
| first time on a champion | no mastery and no recent games on it |
| new to a champion (mild) | under 10,000 mastery and no recent games on it |
| low win rate (mild) | under 40% over at least 10 games |
| bad kda (mild) | under 1.5 over the last 10 games (at least 5) |
| on tilt (mild) | 5 or more games in the last 6 hours with more losses than wins |
| fresh account (mild) | under level 60 while ranked gold or above, so possibly bought or boosted |
| no match history | the client did not return one |

**ranked matching.** riot scrambles each hidden teammate's id with one key per lobby; expose works that key out from the names the riot client already lists and only accepts it when it fits at least three teammates and nothing else fits as well, so those matches are exact. whoever is left is matched by elimination, or guessed from their recent roles, summoner spells and champions and marked "likely" with a confidence. a player the riot client itself does not list (for example riot's streamer mode) is never revealed. enemy identities are never available (riot zeroes them) and are out of scope.

**auto-accept.** when a ready check pops and the toggle is on, expose accepts it. the toggle state is saved between launches.

**dodge.** the dodge button takes two clicks (arm, then confirm within 3 seconds) so it can't fire by accident, and leaves champ select through the client api without closing the client. the **last second** switch arms a dodge for the current lobby that fires when the final countdown reaches 1.5 seconds (1 to 3, in settings), with a visible countdown; it disarms itself when the lobby changes or ends. the normal dodge penalty (lp loss, queue lockout) still applies; expose only saves you the force-quit.

**settings.** a gear in the header opens settings: launch at login, auto-accept, reveal and match ranked teammates, auto-open the scout site when champ select begins, the scout site (op.gg or u.gg), the last-second lead time, the version with an update link, and a button that opens the logs folder. failures (an unreachable riot client, a dodge that could not fire) surface as a brief notice rather than failing silently.

the window is 460 by 680 by default, can be resized down to 420 by 600, and remembers its size and position.

see [scope](#scope) below for what is intentionally left out.

## install

prebuilt installers are published on the [releases page](https://github.com/ibrahimyurdan/expose/releases):

- macos: download the `.dmg`, open it, and drag expose to applications.
- windows: download `Expose_<version>_x64-setup.exe` and run it. it installs for your user only (no admin prompt) and adds a start menu entry. if the pc is missing microsoft's webview2 runtime, the installer fetches it.

the app is not code signed yet, so the first launch shows a warning:

- macos: right click the app and choose open, or run `xattr -dr com.apple.quarantine "/Applications/Expose.app"`.
- windows: on the smartscreen prompt choose "more info" then "run anyway".

on macos expose lives in the dock. on windows it opens its window when you start it and lives in the notification area by the clock: left click the tray icon to show or hide the window, right click it for a small menu with quit. closing the window only hides it. windows 11 may tuck a new tray icon behind the `^` arrow; drag it onto the taskbar to keep it in view.

## build from source

prerequisites:

- rust (stable) and cargo
- node 22 (or 20.19+) and npm
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

tagging `v*.*.*` triggers `.github/workflows/release.yml`, which builds the macos universal `.dmg` and the windows setup `.exe`, attaches them to a github release, and makes it public once both are uploaded, so anyone can download them without a github account. a tag with a suffix, like `v0.2.0-beta.1`, is published as a pre-release. if either build fails, the release stays a draft hidden from the public. ci also builds the windows setup `.exe` on every push to main and every pull request, and keeps it as a download on the run page.

to skip the macos "unidentified developer" warning, set these repository secrets and the workflow will sign + notarize automatically:

- `APPLE_CERTIFICATE` — base64-encoded `Developer ID Application` `.p12`
- `APPLE_CERTIFICATE_PASSWORD` — the p12 password
- `APPLE_SIGNING_IDENTITY` — for example `Developer ID Application: Your Developer Name (TEAMID)`
- `APPLE_ID` — your apple id email
- `APPLE_PASSWORD` — an app-specific password from appleid.apple.com
- `APPLE_TEAM_ID` — your apple developer team id

without these secrets the workflow still runs and ships an unsigned build. an in-panel notice appears in the app when a newer github release is available.

## how it works

the rust backend connects to two local apis: the **league client** (lcu) for champ select, matchmaking, and dodging, and the **riot client** for the chat service that still reports real names during anonymous ranked champ select. each advertises a random port and an auth token via a lockfile and the league process command line; expose finds them either way (on windows it also reads riot's install metadata, so league can live on any drive) and connects over the self-signed https and websocket endpoints (accepted only because the host is always 127.0.0.1).

a single supervisor task owns the connection lifecycle: it detects the client (a 2 second poll plus a file watcher on the lockfile directory), subscribes to two events, dispatches them to the features, and reconnects when the client comes and goes. every champ select update rebuilds one lobby view, which goes to the frontend at once and again as each player's stats arrive (four requests per player, at most four at a time, reused for ten minutes). the log records which matching step placed each ranked teammate, never their names or ids.

```
src-tauri/src/
  main.rs            builder, plugins, tray, window, background tasks
  error.rs           one error type for the whole backend
  models.rs          serde shapes for lcu payloads and emitted events
  state.rs           shared state (toggle, caches, status)
  commands.rs        frontend commands (status, toggle, dodge, open links)
  ddragon.rs         champion id to name and icon lookup
  scout.rs           multi-search url builder per scout site
  update.rs          github release version check
  tray.rs            windows notification-area icon (macos uses the dock)
  lcu/
    auth.rs          credential discovery for the league + riot clients
    client.rs        https client (league client and riot client chat)
    websocket.rs     event subscription and parsing
    mod.rs           connection supervisor (poll, heartbeat, reconnect)
  lenient.rs         tolerant field parsing for client payloads
  lobby/
    session.rs       the champ select session model
    aliases.rs       the hidden names, given out the way the client does
    identity.rs      matching names to slots: exact steps, then a guess
    stats.rs         rank, match history and mastery, with a cache
    flags.rs         red flags and the team summary
    view.rs          the lobby view sent to the frontend
    mod.rs           ties it together for each session update
  features/
    auto_accept.rs   accept ready checks
    dodge.rs         the two-click dodge and the last-second dodge
```

the frontend is react, typescript, vite and tailwind with radix primitives, in its own dark graphite design. it subscribes to backend events and renders the screens: waiting for league, idle (auto-accept and the last lobby), champ select, settings and a first-run welcome. champion icons come from riot's data dragon cdn. `npm run dev` without the tauri shell serves the ui with sample data (for example `http://localhost:1420/?screen=lobby&lobby=ranked`) so it can be reviewed without league running.

## scope

dodging always starts with you: the button, or the last-second dodge you arm for one lobby. expose never dodges because of a flag. deliberately out of scope: enemy team identities (riot does not expose them), draft or matchup analytics, win probability, any automation of in-game actions, and platforms other than macos and windows.

## screenshots

coming with the first public post.

## disclaimer

Expose isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.
