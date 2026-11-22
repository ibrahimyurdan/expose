# expose: the league client look, a shell per platform, and a lighter app

date: 2026-11-21. branch: v0.2 (not yet released; this restyle ships in the same 0.2.0, since 0.2.0 was never published). status: approved by the owner on 2026-11-21, in the closest version.

## why

the owner on v0.2's look: "i didnt like how it looks. could you blend in with league of legends client model? exactly the same but our app. not talking about the size but the style. also make for each mac and windows seperate ui's and all optimize." later: "i want to have as close as possible to the client."

expose is used during champ select, next to the league client. v0.2 looks like a generic dark app (graphite cards, rounded corners, a system font, frosted glass on mac). players read the client's look in an instant: navy black, thin gold frames, uppercase serif headings, gold labels that turn cream on hover. if expose looks like a panel the client opened, it feels like part of the game rather than a foreign tool, and its warnings read with the same weight as the client's own.

## goals

- every screen and control matches the client's style: palette, gold gradient frames, square corners, type roles, control shapes and states, hover and press motion, scrollbars, tooltips, toasts. expose keeps its own name, mark, layout and data.
- a shell per platform: on windows a frameless window with a league-style title bar; on macos the native window with its traffic lights hidden and the same league-style buttons in our header.
- every screen restyled: welcome, waiting, idle, lobby (including the armed and confirm dodge and the details panel), settings, toasts and tooltips.
- optimize: near-zero cpu while league is closed, a release binary of about 5 to 6 mb instead of 17.7 mb, no blank or white first frame, fewer re-renders during champ select.

## non-goals

- no change to what v0.2 does or shows, apart from the lobby band learning the champ select stage (below).
- the window keeps its size: 460 x 680 by default, 420 x 600 at the smallest.
- no splash art background, no in-game overlay, no new screens.
- not two separate apps per os: one set of screens, two shells.

## anti-goals

- a dark web app wearing league colors: rounded cards, pill buttons, frosted glass, bright flat fills. the critic's list of what still read that way in the first mockup is fixed in the reference mockup.
- shipping anything of riot's: fonts, images, sounds, svg path data, css or js. every drawing is our own; hex values and sizes are facts and may be used.
- red that stops meaning "flag": loss dots, the dodge and flags must stay distinct.
- copying the client where it hurts reading: where a client color fails contrast at expose's sizes, expose uses a brighter step of the same hue (listed below).
- anything that implies riot endorses expose.

## constraints

- riot's developer policy: products "cannot closely resemble Riot's games or products in style or function"; only listed riot assets (data dragon) may be used; riot's legal notice must be shown. the owner was told this once (2026-11-21) and decides how close to go (see "the one open decision").
- fonts: riot's beaufort and spiegel never ship and are never loaded from the league install. the free look-alikes are bundled with their licenses.
- tauri 2.11.2 (tao 0.35.2): there is no config option to hide the macos traffic lights, so expose hides them through appkit (objc2-app-kit 0.3.2 is already in the lock file). on windows an undecorated window gets a white 1px border and rounded corners from `shadow: true` unless dwm is told otherwise.
- webview2 lets the standard `scrollbar-width` and `scrollbar-color` properties override `::-webkit-scrollbar`, so those must go. webview2 ignores font smoothing.
- v0.2's 101 rust tests, mac and windows clippy, and the ui build must stay green.

## approaches

how the two platforms differ:
- native window buttons per os (windows: league-style title bar; mac: traffic lights inside a league header). most mac-like, least like the client.
- **league buttons on both: picked by the owner.** both get the client's minimize, settings and close at the top right.

how the window is framed, given league buttons on both:
- one league frame on both: expose draws the whole window on both platforms, square corners everywhere. most uniform; on mac it gives up native corners and resize.
- **keep the mac's own window, hide its buttons: picked by the owner.** mac keeps native rounded corners, shadow and resize, with the traffic lights hidden; windows drops its frame. the two differ slightly (rounded on mac, square on windows), and hiding the lights relies on appkit behavior an os update could change.

dropped: two separate apps or screen sets, one per os. the differences are small, and every screen would have to be built and kept in sync twice.

## how close: closest, picked by the owner

everything above and below is the same either way except a few signature pieces of the client. the reference mockup includes them:
1. the dodge button in the shape of the client's ban button: flat top, sides slanting out, a bottom that bows down, a red rim.
2. gold metal-work around the clock: thin lines that curl into brackets at the bars' outer ends.
3. trim lines just outside the top and bottom edges of framed cards (the summary card, welcome, toasts).
4. row dividers that start with a small hollow diamond.
5. a bracketed ornament beside the welcome title.

- **closest (as mocked):** keep all five, drawn by us. this is what the owner asked for. the cost is the policy risk: riot could ask for changes or a takedown.
- **plainer:** the dodge becomes a framed rectangular button with a red edge, the clock row keeps its bars without metal-work, frames lose the trim lines, dividers are plain fading hairlines, the welcome title gets a plain divider. still clearly league-styled, less of a copy.

the owner picked closest on 2026-11-21, after seeing both versions side by side. all five pieces are built.

## the look

### principles

- surfaces are opaque navy black (#010a13). raised controls step up to #1e2328. groups are set apart by thin gold frames, not lighter cards.
- corners are square. circles only for portraits, status dots and round icon buttons; diamonds for marks and radios.
- gold means "you can use this": labels and glyphs rest in pale gold (#cdbe91) or warm grey (#a09b8c) and turn cream (#f0e6d2) on hover.
- headings, buttons and small captions are uppercase serif with wide tracking; reading text is a quiet warm-grey sans.
- motion is quiet: hovers arrive fast, glows fire once, only a live state loops. `prefers-reduced-motion` turns the pulses, glows and intro motion off.

### tokens

all colors live as css variables on `:root` in `src/index.css` and are exposed to tailwind through `@theme inline`. the radius scale collapses to 0 except where noted. the full set, with gradients, shadows and motion curves, is in the reference (style-spec.md section 1 and mockup/expose.css). the core:

| token | value | use |
|---|---|---|
| bg | #010a13 | window, band, frames, toasts |
| surface | #1a1c21 | details panel, tooltips |
| surface-raised | #1e2328 | button faces, chips, switch knob |
| surface-well | #1e282d | empty portrait, round-button face |
| row hover | linear-gradient(180deg, rgba(200,170,111,0) 25%, rgba(200,170,111,.2)) | player row hover and open |
| border-window | #1e282d | windows window edge, title rule |
| line-gold | #785a28 | 2px line across the top of the window |
| line-header | rgba(240,230,209,.25) | header bottom hairline |
| gold-1 / text | #f0e6d2 | headings, values, hover text |
| gold-2 | #cdbe91 | idle control labels and glyphs |
| gold-3 / accent | #c8aa6e | text buttons, focus ring, bright frame stop |
| gold-4 | #c89b3c | checkbox check, update dot, soft warnings |
| gold-5 | #785a28 | pressed text, scrollbar thumb, frame bright end |
| gold-6 | #463714 | frame dark end, dividers |
| text-secondary | #a09b8c | body text, idle window glyphs |
| text-disabled | #5c5b57 | disabled text and borders |
| win | #0acbe6 | win dots, "W" |
| teal | #0ac8b9 | exact-match check, teal tags |
| status-game | #16cae5 | "champ select" status |
| red | #ff2345 | red flags, errors, dodge label |
| red-strong | #be1e37 | loss dots, red bars |
| red-stripe | #ff4545 | stripe on the red verdict strip |
| amber | #fabe0a | amber flags, your own row |
| green | #00a741 | "connected", "looks fine" check |
| kda-high / kda-low | #35b138 / #d94545 | kda coloring |
| frame | 2px border-image, #785a28 fading to #463714, bright on the edge that faces the actions or the anchor | cards, toasts, tooltips |
| tier tints | two tones per tier (iron through challenger; emerald is ours) | rank crest only; tier text stays #cdbe91 |

where expose departs from the client for reading (all are the client's hue, one step brighter or darker):
- dodge label: #ff2345 at rest (the client's #be1e37 is 2.6:1 on its fill).
- kda-low: #d94545 (the client's #c83535 is 3.8:1 at 12px).
- loss dots: #be1e37, so the brighter #ff2345 means "flag" only.
- red chips: a dark red fill (#2a1015) with a faint red edge, so red and amber chips differ by more than hue.
- focus: the hover look plus a 1px #c8aa6e outline 2px out. the client has no focus ring; keyboard users need one.

### type

- display: cinzel (ofl), weights 400, 500, 700, 900, with metric overrides so it sits like the client's display face. cinzel has no lowercase (its lowercase are small caps) and no italic, so display text is uppercase, and mixed-case or italic text uses the body face.
- body: source sans 3 (ofl), remapped so expose can use the client's weights: 400 is regular, 700 maps to semibold (600), 900 maps to bold (700). the size and metric overrides are in the reference (mockup/fonts-free.css).
- both ship as woff2 in `src/assets/fonts/` with their license texts. the csp already allows `font-src 'self'`.
- roles, as tailwind utilities (`type-h3`, `type-h4`, `type-h5`, `type-h6`, `type-micro`, `type-nav`, `type-phase`, `type-queue`, `type-button`, `type-body`, `type-label`, `type-role`, `type-value`, `type-rank`, `type-tag`, `type-tiny`, and a few more), with size, line height, weight, tracking, case and color each from the reference (style-spec.md section 2). examples: section titles are display 14/18 700, .075em, uppercase, #f0e6d2; body text is 12/16 400, .025em, #a09b8c; labels are 12/16 400, .1em.
- riot ids and slot names use the body face, as the client's champ select rows do. one tracking for the whole name line.
- numbers that line up in columns may use `tabular-nums` (source sans 3 has them).

### components

each v0.2 primitive in `src/components/ui/` keeps its role and takes the client's look. the exact css for each is in mockup/expose.css or style-spec.md section 3.

- **framed button** (primary and default): 2px gold gradient edge, #1e2328 face (primary: dark bronze gradient), display 14px uppercase gold-2 label. hover swaps to the bright edge, fades in a warm fill over 400ms, and pulses a text glow once. pressed darkens the edge and the label. clicks lock for 600ms, as the client does.
- **thin button** (secondary, the default for small actions such as "open logs folder"): 1px gold edge, body 12px semibold label, 200ms crossfade.
- **text button** (ghost): display 14px uppercase in gold-3, cream on hover.
- **icon buttons**: bare 18px glyphs in 28px hit boxes, #a09b8c, cream on hover, #5b5a56 pressed, no plate and no red close hover. a round ring button (gold ring around a #1e282d face) for the scout button and modal closes.
- **checkbox** for every on/off setting: 12px box with a 1px #785a28 edge and a gold check. the client's settings use checkboxes, never switches.
- **switch** for the two live modes only (idle auto-accept, last-second dodge): the client's lobby toggle look, a 70x29 gold-rimmed track with a round gold-rimmed knob that overhangs it. on fill is green; the dodge switch's on fill is red (ours).
- **radio diamonds** for two-choice settings (scout site): an 18px gold diamond ring; selected adds a dark field and a cream center diamond.
- **framed dropdown** for multi-value settings (last-second lead time): 234px wide, gold gradient edge, #a09b8c value, our own up and down chevrons, a #010a13 option list.
- **chip** (flags): the client's lobby notice pill, 22px tall, #1e2328 fill, faint edge, 16px corners, colored text with our warning glyph. red, amber, neutral.
- **tag** ("likely"): 16px tall, 2px corners, gold or amber tones.
- **status** (header): a dot and a label, no container. connected: green; champ select: teal; waiting: grey.
- **frame** (cards), **panel** (quiet container), **raised** (details panel); **dividers**: hairline, fading, vertical, center-fading.
- **tooltip**: #1a1c21 with a 2px gold frame bright toward the target, our own caret, 13px from the target. labels wait 450ms and rich tooltips 250ms, 150ms fade, no slide; a second tooltip within 500ms shows at once.
- **toast**: a framed #010a13 card, 320px, bottom center, at most two, 400ms fade with no slide; info stays 4s and errors 7s.
- **scrollbar**: 11px, transparent track, #785a28 thumb with 3px inset, #c8aa6e on hover, #463714 while dragging. an 18px edge fade only on the edge that has more content.
- **loading**: a shimmer bar for pending values, and the client's three-dot bounce for "still checking" text.
- **empty state**: our mark as a flat #3c3c41 silhouette, a display title, warm-grey text, no icon circle.

### marks and glyphs

all drawn by us as svg in `currentColor` or our gradients, per the shapes in style-spec.md section 6: minimize, close, gear and resize grip; the frame trim; checkbox check; radio diamonds; switch check; tooltip caret; warning triangle; external-link arrow; exact-match check; portrait rings; row divider diamond; dodge shape; metal-work; welcome ornament; position glyphs (top, jungle, mid, bottom, support, fill); the rank crest (a two-tone shield in the tier tints, never a gem, which reads as diamond tier).

the brand mark is the app icon's two nested diamonds, filled with the gold ring gradient (flat #c8aa6e below 14px). it replaces v0.2's eye in the header, the welcome screen and the empty state, so one mark appears everywhere (my call; the mockup still shows an eye).

## the screens

the reference mockup shows the lobby on windows and mac and settings on windows (`.claude/design/league-style/mockup/*-free-fonts.png`). the other screens follow the recipes in style-spec.md section 3.

### header (both platforms)

- 41px: a 2px #785a28 line across the top, a 38px row, a 1px rgba(240,230,209,.25) line below, opaque #010a13.
- left (15px in): the mark and "EXPOSE" in the nav type.
- right: the status, then minimize, settings and close as bare glyphs, 28px hit boxes 5px apart, the close glyph's right edge 11px from the window edge.
- settings open: the gear stays a gear, lit cream, `aria-pressed`. update available: a 6px #c89b3c dot at the gear's top right; the "get x.y.z" action lives in settings.
- the whole strip is the drag region, its buttons excluded. a double-click never maximizes.
- minimize and close show on every screen, the welcome screen included.

### lobby

top to bottom: the band, the summary card, the team.

- **band** (sticky, with a fade under it once stuck):
  - the phase line, centered: italic sentence case in the body face ("pick and ban"); on the local player's turn, upright uppercase display in cream.
  - the clock between two bars that drain toward it. the clock prints bare seconds ("26") unless the phase lasts 100 seconds or more, then m:ss, as the client does. in the final 10 seconds the clock and bars pulse.
  - bar colors follow the client: teal in planning and finalization, red during bans; during picks only the acting team's bar shows. while the last-second dodge is armed, the bars turn red in finalization with a tick at the moment it fires.
  - gold metal-work under each bar (signature piece 2).
  - the queue name below in small display caps.
- **summary card**: a #010a13 frame with trim (signature piece 3).
  - head: the verdict ("3 red flags on 2 teammates") in display caps with the detail line under it ("still checking 1 teammate" plus the bouncing dots), and the round scout button on the right. with red flags the head is a 44px strip with a dark red gradient and a 6px #ff4545 stripe at its left edge; amber flags use the same strip in amber; with no flags, a green check and no strip.
  - foot: the last-second switch with its label, and the dodge button on the right.
- **dodge button** (signature piece 1): 128x36 in the ban-button shape, red rim, dark fill, display label in #ff2345. hover and the confirm step brighten the fill to a dark red; the confirm step also runs a 3 second drain line under the label. pressed and leaving: dim rim and #802626 text. disabled: grey. below 32px of height it falls back to a framed button with a red edge. the confirm step's label reads "confirm": v0.2's "confirm dodge" is about 132px in this type and does not fit the 128px shape. the hint under the card still says "press again to dodge. the usual dodge penalty applies."
- **last-second switch**: red when on. its label reads "last second" at rest; while armed it switches to the client's countdown style, "dodges at 0:01.5" in #c89b3c, then "dodges in 0:03" in #ff2345 during finalization.
- **player row**, no card fill, a row-glow on hover and when open:
  - a 44px round portrait in a gold gradient ring: brighter gold on your own row, teal while that teammate is acting; a dark disc with the slot's position glyph before a pick.
  - line 1: the role in pale gold; the rank crest, tier and lp on the right.
  - line 2: the champion in display caps with "· 7 games"; results dots (newest left) and kda on the right.
  - line 3: the alias and the real riot id (cream, tagline grey) with the teal exact-match check or a "likely" tag; "name hidden" and "matching name" in italic.
  - line 4: flag chips, or a green check and "looks fine", or the season line when there is nothing to flag; the details toggle (a small triangle) at the right.
  - your own row is in #fabe0a text instead of a "you" chip. a 3px teal bar at the left edge marks a teammate who is picking or banning now (amber on your row).
  - rows are separated by a 9px strip: a small hollow diamond and a fading line (signature piece 4).
- **details panel**: the client's scouting card look, #1a1c21 with a gold frame bright at the top and a caret pointing at the portrait. sections (rank and lp, champion experience, roles, recent games) are split by fading dividers. role bars for all five positions, with the slot's position highlighted so an autofilled player's short bar stands out. recent games in two columns: champion square, W or L, kda, time ago. the scout link is a thin button with the external arrow.

new data the band needs (backend): the lobby view gains the champ select stage (planning, ban, pick, finalization) and who is acting (your team, theirs, or both, plus which of your cells), read from the session's `actions`. it also gains the phase's total time (`timer.totalTimeInPhase`), which the clock's 100-second rule and the bars' fill need; today the view only carries the time left. until that data arrives on a session, the band uses teal bars and no acting marks. the repo's one recorded session is a trimmed pick-and-ban snapshot with no turn in progress, so the parsing is tested on session fixtures written in that recorded session's own shape: each action has `actorCellId`, `type`, `isAllyAction`, `isInProgress` and `completed`, and the session carries `hasSimultaneousBans` and `hasSimultaneousPicks`. the fixtures cover planning, simultaneous bans, your team's pick, the enemy's pick, finalization and a blind pick lobby. the lobby logs the stage and acting side it derives whenever they change, so the first live champ select confirms the parsing from the log.

### waiting

the empty state: the flat mark, "open league of legends" in display caps, a warm-grey line that expose connects on its own, the bouncing dots.

### idle

- auto-accept: a framed card with trim, a display title, a one-line hint and the switch on the right.
- last lobby: a small caps header with how long ago, above a quiet panel holding the queue, the verdict chip and the names.

### welcome (first run)

a framed card with trim, 388px wide at most: the mark at 40px, the title ("welcome to expose") in large display caps, a one-line subtitle, three feature rows (our 24px glyph in pale gold, a display title, body text), and a primary "get started" button that sits on the frame's bottom edge between two short gold posts. riot's notice in small type at the bottom.

the title is too wide for the bracketed ornament (signature piece 5) to flank it in a 388px card, which needs about 64px free on each side. so the ornament sits centered under the title as its divider, its two mirrored halves meeting at a small gap.

### settings

- a breadcrumb title bar, "expose / settings", the current part brightest, with a 1px rule under it. no cards and no group boxes.
- sections general, champ select, dodge, about, each with a display caps title.
- checkbox rows with a hint under each label; label tracking .025em as in the client's settings.
- scout site as two radio diamonds; last-second lead time as the framed dropdown, with v0.2's values.
- about: the version with "get x.y.z" as a thin button when an update exists, "open logs folder", riot's notice and the font licenses (names, copyright lines, a link that shows the bundled license text).

### copy

v0.2's wording stays throughout ("bot", not "bottom"), including the armed dodge's countdown label ("dodges at 0:01.5", "dodges in 0:03"). the one change is the dodge button's confirm step, which reads "confirm". the only new strings are the riot notice and the font credits.

## the two shells

| | windows | macos |
|---|---|---|
| window frame | none (frameless) | native, with rounded corners |
| traffic lights | n/a | hidden through appkit |
| shadow and edge | windows 11: os shadow with square corners and a #1e282d os border. windows 10: no shadow, a 1px #1e282d css border on the sides and bottom | native shadow and outline |
| resize | the edges, plus an 18px corner grip at the bottom right | native edges |
| minimize button | hides to the tray (the window has no taskbar button) | minimizes to the dock |
| close button and alt+f4 | hides to the tray | hides; clicking the dock icon reopens it |
| quit | tray menu | ⌘q, or quit from the dock icon's menu |
| tray or menu-bar icon | the tray icon, as in v0.2 | none, as in v0.2 (expose lives in the dock) |
| start at login | starts hidden in the tray, as in v0.2 | opens the window, as in v0.2 |
| settings shortcut | ctrl+, | ⌘, |
| first click on the inactive window | goes through already | `acceptFirstMouse`, so a click on dodge while league has focus goes through |
| background | opaque #010a13 | opaque #010a13; the frosted glass and window-vibrancy are removed |

how it is wired:
- the main window is built in rust during setup from the shared config, with each platform's builder calls behind `cfg`: windows gets `decorations(false)`, `maximizable(false)` and its shadow choice; macos gets the overlay title bar, a hidden title, `maximizable(false)` and `accept_first_mouse(true)`, then hides its close, miniaturize and zoom buttons through the `NSWindow`'s standard window buttons. the hide is re-applied after resize and focus changes in case appkit brings them back. the mac builder must not set `traffic_light_position`: when a position is set, tao 0.35.2 moves the buttons on every redraw; without one, nothing touches them. building in rust avoids the json merge patch trap of `tauri.windows.conf.json`, which replaces the whole windows array.
- windows 11 is told to keep square corners and use the #1e282d border color through dwm window attributes; windows 10 (no such attributes) gets `shadow(false)`. rust tells the ui which case it is in, so the css border shows only on windows 10.
- rust injects `window.__EXPOSE_PLATFORM__` ("macos" or "windows") through the window builder's `initialization_script`, before any page script runs. the ui sets `<html data-platform>` from it, with tailwind variants `mac:` and `win:`. the browser preview takes `?platform=mac|windows` and draws the windows bar exactly, and on mac where the hidden lights would be, nothing.
- one `hide_main_window` command saves the window's size and position and hides it; windows' minimize and close and mac's close all use it. mac's minimize calls the window's minimize.
- capabilities gain minimize, start-resize-dragging and a deny on the internal maximize toggle; start-dragging stays.
- the window is shown only once its first screen has painted: the ui calls `frontend_ready` after the first real screen, `document.fonts.ready` and one animation frame; rust shows the window then, with a 1.5 second fallback. v0.2's launch rules stay: on windows a login autostart stays hidden in the tray; on macos the window opens at every launch.

## optimization

| change | expected gain |
|---|---|
| stop the idle process scan: with league closed, scan processes every 30 s while the lockfile watcher is active (it wakes the loop when league starts), every 5 s without it; on macos list processes through libproc and read arguments only for league's pids | idle cpu from about 2% of one core to near zero (measured cost 7–18 ms per scan every 2 s) |
| release profile: `lto = true`, `codegen-units = 1`, `opt-level = "s"`, `strip = true`, `panic = "abort"`, plus a panic hook that logs the panic before the default hook | 17.7 mb to about 6.2 mb (measured on mac); release builds get slower (about 99 s clean) |
| ring instead of aws-lc-rs for tls, installed as the default provider on the first line of `main` | about 6.2 to 5.1 mb, faster clean builds, nasm leaves ci. tls with the live league and riot clients must be checked |
| drop window-vibrancy | comes with the opaque look; removes a duplicate objc class and unblocks full lto |
| show the window once painted | no empty or white first frame, no font swap |
| skip publishing a lobby view equal to the last one; push champions as an event instead of the 1.5 s poll; reload data dragon when league connects (patch day); gzip for data dragon | fewer re-renders and wakeups; champion.json 159 kb to 35 kb per launch |
| faster champ select: read riot client chat every 500 ms for the first seconds of a lobby, then every 2 s, with a time-based patience limit; publish summoner and ranked before history and mastery | names and ranks up to about 1.5 s sooner; still only complete stats are cached |
| rows keep their height when stats land; the shimmer animates a transform | no jumping rows, no full repaint while loading |
| dependency hygiene: sysinfo with only its `system` feature, tokio with only the features used, the unused js packages (`@tauri-apps/plugin-opener`, `@tauri-apps/plugin-store`, radix scroll-area and separator) and the dead `scroll-area.tsx` and `separator.tsx` removed, `removeUnusedCommands` on | faster builds, smaller bundle |
| windows only: webview2 memory target low while hidden, normal on show | lower memory while in the tray (best effort) |

not done: lazy-loading screens (under 5 kb), per-platform bundles, replacing radix tooltip.

## policy guardrails

- nothing of riot's in the repo or the bundle: no fonts, images, sounds, svg path data, css or js from the client, communitydragon or the install. code comments and committed docs never cite client files.
- champion squares and other images only from data dragon, as today.
- no riot or league logos or trademarks in expose's name, icon, wordmark or store text; no riot vocabulary ("hextech") as branding.
- riot's notice, word for word, in settings → about, at the bottom of the welcome screen, and in the readme in place of the current shorter line:

  > Expose isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.

- the font credits next to it: cinzel and source sans 3 under the sil open font license 1.1, with their copyright lines and the bundled license text viewable.

## my calls (the owner can overrule any)

- display font cinzel rather than spectral (the research's first pick): its capitals are much closer to the client's, and the client sets its display text in capitals anyway. mixed case goes to the body face.
- the brand mark is the app icon's nested diamonds everywhere, replacing v0.2's eye.
- windows corners: square with the os shadow on windows 11, no shadow with a css border on windows 10. windows has no help button; minimize also hides to the tray.
- `panic = "abort"` with a logging hook: a crash quits and leaves a log line instead of leaving a half-working app.
- a flat lobby background, no splash art: rows and flags stay readable at 460px.
- our own rank crest and position glyphs rather than riot's developer-portal emblem files, whose license scope is unread.
- settings use checkboxes and the live modes use the switch, as the client does; lead time uses the framed dropdown.
- toasts stay at the bottom (the client puts them top right).
- the lobby band learns the stage and who is acting, so its colors and acting marks match the client. its tests use hand-written session fixtures, and a log line confirms the parsing in the first live champ select, so the owner does not need to capture sessions first.
- the welcome ornament sits under the title as a divider, because "welcome to expose" leaves no room beside it; the dodge's confirm step reads "confirm".
- macos keeps v0.2's dock-only behavior: no new menu-bar icon, and the window opens at every launch.

## left to the builder

- how the css is organized (tokens in `@theme`, components as classes or utilities) and which radix primitives stay (tooltip stays; switch, checkbox and radio may be radix or native as long as keyboard and screen reader behavior holds).
- the exact svg drawings, within the shapes and colors in the reference.
- the optional client polish: button click sheen and flares, the row intro motion, the acting spinner and wash, the band's top glow on your turn.
- how rust detects windows 10 versus 11, and how it tells the ui.
- kda thresholds stay v0.2's.

## how it is checked

- unit tests: the stage and acting parsing on the hand-written session fixtures, the clock format (bare seconds under 100 s), platform detection; all v0.2 tests stay green; mac and windows clippy; the ui build.
- visual: every screen in the preview on both shells at 460 x 680 and 420 x 600, compared with the reference mockup. the mockup lobby already reflows at 420 x 600 without overflow; the armed countdown label is the one to watch there, since it is wider than "last second" and must not push the dodge button; the armed dodge, confirm step, details panel, toasts, tooltips, welcome, waiting and idle checked one by one; text contrast checked against the surfaces.
- the mac app, built and run: lights hidden on launch and after resize and focus changes, all three buttons, drag, native resize, minimize to the dock, close and reopen, ⌘q, no white flash, near-zero cpu with league closed, binary size.
- windows: the exe built by ci, tested by the owner on their pc: corners, border and shadow, edge and grip resize, snap, alt+f4, no taskbar button after a show, a dark first frame.
- live league: tls through ring against the league and riot clients, and a champ select to see the band's stage colors.

## reference material

local and not committed (`.claude/design/league-style/`, see its readme): the per-component value sheet (style-spec.md), the mockup stylesheet with the critic's fixes (mockup/expose.css, which wins where the two differ), the mockup pages and renders, and the critique.
