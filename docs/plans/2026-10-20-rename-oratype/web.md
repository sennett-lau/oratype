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
3. Move the website to Cloudflare (below).
4. Cut 1.2.0 as usual (`docs/releasing.md`). The release files are `OraType_1.2.0_aarch64.dmg`
   and friends. Installed Typelite copies see 1.2.0 through the redirected feed, update in
   place, and the first launch moves their data (migration.md).

## Website

The site moves from GitHub Pages to Cloudflare Workers with static assets, in the Cloudflare
account that holds the `sennettlau.me` zone (`site/wrangler.jsonc`, `site/worker/index.ts`):

- `oratype.sennettlau.me` serves the built page.
- `typelite.sennettlau.me` answers every path with a 301 to the same path (and query) on
  `oratype.sennettlau.me`. A 301 tells search engines the page moved for good, so its ranking
  moves with it.
- Both are Workers Custom Domains: Cloudflare creates their DNS records and certificates. The
  old `typelite` record (a `CNAME` to GitHub Pages) is replaced by the Worker's.
- `.github/workflows/site.yml` deploys on every push to `main` that touches `site/`, with the
  repository secrets `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`; `pages.yml` and GitHub
  Pages are retired.

## Search

- The page title, description and Open Graph text say "OraType (formerly Typelite)", the
  structured data names `Typelite` as `alternateName`, and the hero and footer say it too.
- The README says OraType was called Typelite until 1.2.0.
- On GitHub, add `typelite` to the repository's topics so its search finds the renamed
  repository.
