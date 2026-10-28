# Upstream-Update 2026-10-28 (penso/herdr-gpui → doppeltpunkt/herdr-gpui)

Laufend geführt. Branch: `merge/upstream-20261028`.

## Ausgangslage
- Fork-`main` = upstream-Stand 2026-10-25 (`3a684411`), keine eigenen Commits.
- Fork-Feature: `feat/pinned-sidebar-workspace` = 1 Commit `f4aae577` (pinned workspace row).
- `~/git-applications/herdr-work-usage` = **Worktree** dieses Repos (nicht eigener Clone),
  Branch `fix/work-account-usage` @ `30c2d78c` (2026-10-14) + uncommitted Claude-Work-Usage-Patch
  (identisch zu `~/git-applications/herdr-work-usage-claudework.patch`).
- Theme- und Browser-Annotate-Funktionen sind upstream, keine Fork-Patches.
- `/opt/homebrew/bin/herdr` 0.9.4 = Herdr-**Daemon** (Homebrew, extern), nicht dieser Fork.
  Die GUI läuft aus `/Applications/Herdr.app` (lokaler `just bundle`-Build, Bundle-ID
  `so.pen.herdr-gpui.dev`, ad-hoc signiert, 2026-10-26 02:15 = pinned-sidebar-Stand).

## Schritte
- [x] Work-Usage-Änderungen gesichert: Branch `wip/claudework-usage-backup` @ `6ede3cb0` (gepusht).
  Nicht gemergt: laut Wiki am 2026-10-16 auf Wunsch zurückgerollt („do not reapply“).
- [x] `git fetch upstream`: 37 neue Commits bis `b16a39bf` (Tag `v20261028.1`).
- [x] Merge `upstream/main` → `merge/upstream-20261028` (Basis `feat/pinned-sidebar-workspace`):
  **keine Textkonflikte**. Überlappende Dateien (sidebar.rs, sidebar/render.rs,
  config/tests.rs, sidebar/layout_tests.rs, config-gpui.example.toml) automatisch zusammengeführt.
- [x] Release-Build `PATH=/opt/homebrew/opt/rustup/bin:$PATH just bundle` → `target/release/Herdr.app`:
  grün (5:28 min, nur Future-Incompat-Warnung `block v0.1.6`). Hinweis: Homebrew-rustup legt
  `cargo` nicht in `/opt/homebrew/bin`, daher der PATH-Zusatz.
- [ ] Tests (pinned-workspace-Tests, Workspace-Tests)
- [ ] Push

## Neu upstream (Auswahl)
- Chat: Rich Rendering für ACP/CLI-Sessions (#801), Tab-Titel nach letztem Prompt (#813), Prompt-Verlauf mit ↑/↓ (#814)
- Remote Control: mehrere Computer koppeln (#815), Pairing-Adresse wählen/Tailscale bevorzugen (#850), Live-Streaming (#816), Geräte vergessen (#825)
- UI: Animationen für Menüs/Dialoge/Toasts/Sidebar (#820), Curved-Tab-Layout (#821), Geräteliste in Sidebar (#829)
- SSH: PRs/Issues auf SSH-Hosts (#839), mise-installiertes Herdr finden (#840)
- Updater zeigt Notes aller Releases seit installierter Version (#832); Review merkt „viewed“ (#834)
- Fixes: Diktat-Tools fügen Text ins Terminal ein (#841), große Bilder per Cmd-V (#818), herdr-acp/herdr-automate in Release-Paketen (#817, #822)
