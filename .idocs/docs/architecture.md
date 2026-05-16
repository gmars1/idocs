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
Two-panel layout: valid docs (left) and stale docs (right).
Navigate with arrows/Tab, press Enter to read a doc with markdown
rendering (headings, lists, inline code, bold).

Keybindings: ↑↓/jk navigate, Tab switch panel, Enter read, r refresh, q quit.

## TUI mode

Available via `idocs -i`. Built with ratatui + crossterm.
Two-panel layout: valid docs (left) and stale docs (right).
Press Enter on a doc to open it in $EDITOR (falls back to vi).

Keybindings: ↑↓/jk navigate, Tab switch panel, Enter open in editor, r refresh, q quit.
