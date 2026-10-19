# First launch after the rename

What OraType moves from Typelite on its first launch. Part of [rename-oratype](index.md).
The code is `src-tauri/src/rename_migration.rs`; it runs right after logging starts, before the
Tauri app is built, and does nothing when there is nothing to move.

| What | From | To |
|---|---|---|
| Settings, database, models | `~/Library/Application Support/dev.typelite.mac` | `…/dev.oratype.mac` (a rename, so models are not copied) |
| The database file | `typelite.db` (+ `-wal`, `-shm`) | `oratype.db` |
| The windows' local storage | `~/Library/WebKit/dev.typelite.mac` | `…/dev.oratype.mac` |
| Launch at login | `~/Library/LaunchAgents/Typelite.plist` | removed; OraType registers `OraType.plist` from the setting |
| The app bundle | `Typelite.app` (an in-place update keeps the folder name) | `OraType.app`, then started again with `open -n` |
| API keys | keychain service "Typelite" | "OraType", on first use (`credentials.rs`) |
| The output-mute crash flag | `muted-by-typelite` | `muted-by-oratype` (`audio/output_mute.rs`) |

The development build moves `dev.typelite.mac.dev`, "Typelite Dev" and `Typelite Dev.app` the
same way.

- When both the old and the new data folder exist, OraType has run before: its own folder wins
  and Typelite's is left in place.
- The bundle is not renamed when something called `OraType.app` is already there, or when macOS
  runs a downloaded copy from a read-only place (App Translocation). The app then keeps running
  from where it is.
- Logs start fresh in `~/Library/Logs/OraType/oratype.log`; Typelite's stay where they were.
- Cannot move: macOS permissions (Accessibility, Microphone, Automation of System Events). They
  belong to the bundle identifier, so OraType asks for them like a new app; What's New says so.
- After the update, the old login item and a Dock icon point to the old path. The login item is
  replaced; a Dock icon has to be added again.
