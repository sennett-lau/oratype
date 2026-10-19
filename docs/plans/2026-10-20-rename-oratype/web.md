# Repository, website and search

How old links keep working and how search finds the new name. Part of
[rename-oratype](index.md).

## Order

The new app reads its update feed and language presets from `sennett-lau/oratype`, so the
repository is renamed before 1.2.0 is tagged:

1. Merge the rename to `main`.
2. Rename the repository on GitHub to `oratype` (Settings → General, or `gh repo rename oratype`).
   GitHub then redirects `github.com/sennett-lau/typelite/…`: pages, `git` remotes, release
   downloads and `releases/latest/download/latest.json`, which installed Typelite copies poll
   for updates. Local clones keep working; `git remote set-url origin
   git@github.com:sennett-lau/oratype.git` makes it explicit.
3. Point the website at the new domain (below).
4. Cut 1.2.0 as usual (`docs/releasing.md`). The release files are `OraType_1.2.0_aarch64.dmg`
   and friends. Installed Typelite copies see 1.2.0 through the redirected feed, update in
   place, and the first launch moves their data (migration.md).

## Website

The site stays on GitHub Pages; the DNS for `sennettlau.me` is on Cloudflare.

1. Cloudflare DNS: a `CNAME` record `oratype` → `sennett-lau.github.io`, DNS only (grey cloud),
   as `typelite` has today.
2. GitHub → Settings → Pages → Custom domain: `oratype.sennettlau.me`, then Enforce HTTPS once
   the certificate is issued.
3. Cloudflare: switch the `typelite` record to Proxied (orange cloud), and add a Redirect Rule
   (Rules → Redirect Rules → Single Redirect): when the hostname equals
   `typelite.sennettlau.me`, a dynamic redirect to
   `concat("https://oratype.sennettlau.me", http.request.uri.path)`, status 301, keep the query
   string. A 301 tells search engines the page moved for good, so its ranking moves with it.

## Search

- The page title, description and Open Graph text say "OraType (formerly Typelite)", the
  structured data names `Typelite` as `alternateName`, and the hero and footer say it too.
- The README says OraType was called Typelite until 1.2.0.
- On GitHub, add `typelite` to the repository's topics so its search finds the renamed
  repository.
