# expose: the real rank emblems and role icons

status: approved by the owner on 2026-11-22 ("evet, böyle yap" to reading them from the league client). a short design: a small change on branch `v0.2`, on top of the league-style restyle.

## intent

the owner wants expose to show league's real icons, the way op.gg does: the rank emblems (iron, bronze, silver, gold, platinum, emerald, diamond, master, grandmaster, challenger) instead of expose's own drawn two-tone shields. the point is that a rank reads at a glance, as it does in the client. the role icons (top, jungle, mid, bot, support) are the other place where expose draws its own stand-in, so they switch too (my proposal in the conversation, not objected to). champion pictures already come from riot's data dragon and stay as they are.

## goals

- each player row's rank (line one, 14px) shows the client's small rank crest for the tier, the one the client draws for small sizes.
- a player's details show the tier's full emblem, larger, with the ranked season.
- an unranked player shows the client's unranked crest beside "unranked".
- the role icons on an empty portrait (no pick yet) and in the details' role bars are the client's position icons; the slot's own role is the light variant.

## non-goals

- no change to the rank data, the rank text, the lp or the layout of the four row lines.
- champion squares stay on data dragon.
- no other client art (borders, banners, regalia).

## anti-goals

- riot's art inside expose: nothing of riot's goes into the repo or the bundle (the restyle spec's guardrail stands; riot's developer policy allows only press kit assets in products).
- a third-party image host.
- a broken image, an empty gap or a layout shift when league is closed or a file is missing: expose's own drawings stay as the fallback, the same size.
- a way for the page to reach anything beyond these images: the protocol reads only the listed files from league's archive and never talks to the client.

## approaches

1. **read them from the player's own league at runtime (picked by the owner).** nothing of riot's ships with expose, and the art always matches the installed client. first planned through the client's local server; a live check showed it doesn't serve these files, so rust reads them from the client's own archive on disk (see below). costs: a small archive reader in rust.
2. bundle the emblem files in the app. works offline, but ships riot's art in a public repo and every download, which the spec ruled out and riot's developer policy does not allow.
3. load them from communitydragon at runtime, as champion squares load from data dragon. the simplest code, but it depends on a community host serving riot's files, which the spec's guardrails excluded.

## what was decided

- the files, from the client's static assets plugin (`rcp-fe-lol-static-assets`), names checked against its file tree:
  - small crests: `images/ranked-mini-crests/<tier>.svg`, all ten tiers plus `unranked.svg` (emerald has only the svg, so svg throughout).
  - full emblems: `images/ranked-emblem/emblem-<tier>.png`.
  - role icons: `svg/position-<top|jungle|middle|bottom|utility>.svg`, with `-light` variants.
- the source, corrected after a live check on 2026-11-22: the client's local server does not serve its interface files (every `/fe/...` path is a 404, "invalid uri format"), so expose reads them from league's own archive on disk instead: `<league folder>/Plugins/rcp-fe-lol-static-assets/assets.wad`, the folder that holds league's lockfile. the archive is riot's wad format, version 3, with entries keyed by the xxh64 hash of the lowercased path `plugins/rcp-fe-lol-static-assets/global/default/<file>` and stored raw or zstd-compressed. all 31 files were found and unpacked on the owner's mac. it is still the player's own league that supplies the art, and it now works with league closed as long as it is installed.
- the path: the page asks a custom protocol registered in rust; rust checks the path against the exact list of files above, reads it from the archive, and answers with the bytes and their content type. get only. anything else, no league install, or a file the archive lacks is a 404, and the page shows the fallback drawing.
- a small in-memory cache per app run, keyed by path, so each file is fetched once.
- the image urls are built per platform with tauri's `convertFileSrc` (macos `<scheme>://localhost/...`, windows `http://<scheme>.localhost/...`), and the content security policy's `img-src` gains both forms.
- `RankCrest` and `PositionGlyph` stay, as the fallbacks and in the browser preview, which has no rust side.

## left to the builder

- the protocol's name.
- the full emblem's size and its exact spot in the details (it must fit at 420px).
- how the fallback swaps in (an image error handler, without a flash).
- whether the cache needs a size cap (the whole set is under 2 mb).

## checks

- unit tests: the path check (allowed files pass; other folders, `..`, encoded characters, query strings and other methods fail) and the content type per extension.
- the preview: with no rust side every icon falls back to the drawings, with no layout change.
- live, on the mac: the three kinds of files answer through the protocol, and the row crest, the details emblem and the role icons show in a champ select. with no league install (or a renamed archive), the drawings show.
