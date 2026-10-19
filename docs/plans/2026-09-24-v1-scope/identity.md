# Identity and build

Names, identifiers and build settings for OraType. Back to [index](index.md).

## Identity

| Where | Value |
|---|---|
| App name, window titles, tray, UI text | OraType |
| Bundle id (`tauri.conf.json`) | `dev.oratype.mac` |
| Crate and library names (`Cargo.toml`) | `oratype`, `oratype_lib` |
| npm package (`package.json`) | `oratype` |
| Data folder, database, keychain service | `dev.oratype.mac`, `oratype.db`, `OraType` |
| Env vars and thread names | `ORATYPE_*`, `oratype-*` |
| Icons | `src-tauri/icons/source/*.svg` |

## Licence

MIT. `LICENSE` carries every required copyright notice.

## Build and signing

- `npx tauri build --bundles app`.
- `rust-toolchain.toml` pins a current stable Rust, so builds do not depend on the machine's
  global default.
- Every build should be signed with one stable certificate, so macOS permissions survive
  rebuilds.

## Updates

There is no auto-updater until OraType publishes its own releases with its own signing key.
