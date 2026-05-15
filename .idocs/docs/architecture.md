# Architecture

`idocs` is a Rust CLI tool that tracks which source files your documentation
references and detects when those sources become stale.

## Module layout

- **`src/index.rs`** — data model (`Index`, `DocEntry`), JSON persistence
  (`load_index`/`save_index`), `project_root` discovery, `doc_id` sanitization.
- **`src/check.rs`** — staleness detection. `file_sha256` hashes a file,
  `check_all` compares stored hashes against current filesystem state.
- **`src/cmds.rs`** — all command implementations: `cmd_add`, `cmd_up`,
  `cmd_rm`, `cmd_info`, `cmd_read`, `cmd_edit`, `cmd_stale`, `cmd_default`.
- **`src/main.rs`** — CLI entry point. Defines the `Cli` struct and
  `Commands` enum via clap derive, routes subcommands to `cmds::*`.
- **`src/tui.rs`** — Interactive TUI mode (`idocs -i`). ratatui + crossterm.
  Two-panel layout (valid/stale), $EDITOR integration, keyboard navigation.

## Data flow

1. `idocs add <name> <sources...>` stores source file SHA-256 hashes in
   `.idocs/sources.json`.
2. `idocs` (default) loads the index, re-hashes every source, compares
   against stored hashes. Reports valid/stale per doc.
3. `idocs up <name>` re-hashes and updates stored hashes after manual review.
4. `idocs edit` modifies doc `.md` content via `--set`, `--lines`/`--text`,
   `--replace`/`--with`, or piped stdin.
5. `--json` flag produces machine-readable output for all commands.

## TUI mode

Available via `idocs -i`. Built with ratatui + crossterm.
Two-panel layout: valid docs (left, green) and stale docs (right, red).
Pressing Enter on a doc suspends the TUI, opens the doc in $EDITOR
(parsing args like `subl -w`), and resumes after the editor closes.
`terminal.clear()` is called after re-entering alternate screen to
ensure the full UI (including bottom help bar) redraws correctly.

### Data flow for editor open

1. TUI draws two-panel layout with help bar at bottom
2. Enter key → `disable_raw_mode` + `LeaveAlternateScreen`
3. Editor process runs (blocking)
4. Editor exits → `enable_raw_mode` + `EnterAlternateScreen` + `terminal.clear()`
5. Full redraw via `terminal.draw()`, sources re-checked

Keybindings: ↑↓/jk navigate, Tab switch panel, Enter open in $EDITOR, r refresh, q quit.
