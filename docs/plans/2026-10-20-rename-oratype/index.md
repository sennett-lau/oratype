# Rename to OraType

Typelite is renamed OraType in 1.2.0. "Typelite" is hard to find in search: an older,
unrelated project called TypeLITE ranks for it. Everything is renamed: the app, its bundle
identifier, the crate, the website and the repository. Installed copies update to OraType by
themselves and keep their settings, models, dictionary and API keys. Old links (the website, the
repository, its releases and the update feed) lead to the new name.

Status: done (2026-10-20)

## Goals and non-goals

- Every name a user or contributor sees says OraType: the app, menus, windows, notices, release
  files, the website, the README and the docs.
- An update from Typelite keeps everything the user set up. The only thing to redo is granting
  Accessibility and Microphone once, because macOS ties them to the identifier.
- Old URLs keep working and pass their search ranking on: `typelite.sennettlau.me` and
  `github.com/sennett-lau/typelite` (pages, releases, the update feed).
- Searching for Typelite finds OraType: "formerly Typelite" on the site, in its metadata and in
  the README.
- Not a goal: a new icon. The text-cursor mark fits the new name.

## Key decisions

| Decision | Reason |
|---|---|
| The bundle identifier changes too (`dev.typelite.mac` → `dev.oratype.mac`, the dev build `….dev`) | The owner chose a complete rename. The cost is one new permission grant per user, said in What's New. |
| On the first launch OraType moves Typelite's folders (`rename_migration.rs`) before anything reads them | macOS keys the data folder, the web storage and the login item by identifier and name; moving keeps settings, the database and gigabytes of models without a copy. See [migration.md](migration.md). |
| API keys move on first use: a key missing under the keychain service "OraType" is read from "Typelite" and copied | Reading another app's keychain item may ask the user once; doing it at first use, not at launch, asks only for keys that exist. |
| A bundle still called `Typelite.app` renames itself `OraType.app` and starts again | The updater replaces the bundle's contents in place, so the folder would keep the old name. |
| Files exported by Typelite (presets, dictionary) still import; new files carry the new format names | Users have shared and saved such files. |
| The signing certificate keeps its name, "Typelite Signing" | Only `codesign` shows it. A new certificate means new repository secrets and a new keychain item on the maintainer's Mac for no visible gain. |
| The GitHub repository is renamed `oratype` | GitHub redirects every old repository URL, release downloads included, so installed Typelite copies find the 1.2.0 update through the old feed URL. A new repository must never take the name `typelite`, or the redirects stop. |
| The site moves from GitHub Pages to Cloudflare Workers, which also answers `typelite.sennettlau.me` with a permanent redirect (301) to the same page on `oratype.sennettlau.me` | GitHub Pages serves one custom domain per site, so the redirect needed a separate Cloudflare rule; a Worker does both in code, and Cloudflare keeps both domains' DNS and certificates. A 301 carries the old domain's ranking over. See [web.md](web.md). |
| Recorded data (benchmark and research JSON) keeps the old name | It is evidence of past runs; rewriting it would change what was measured. |

## Parts

| File | Covers |
|---|---|
| [migration.md](migration.md) | What moves on the first launch, in which order, and what cannot move |
| [web.md](web.md) | Repository rename, the website's domain and redirect, search, the release order |

## Open questions

- Should the old GitHub releases' notes get a line pointing to OraType? The redirects already
  work; the notes would only help a reader who lands on an old release page.
