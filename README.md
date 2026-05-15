# 📝 idocs

CLI tool that detects when your markdown docs become stale relative to source files.

Track which source files your documentation references. When a source changes,
idocs flags the doc as stale — re-hash after review to acknowledge it's up to date.

## Install

```sh
# if you have Rust toolchain
cargo install --git https://github.com/gmars1/idocs

# or build from source
git clone https://github.com/gmars1/idocs && cd idocs
cargo build --release && ./install.sh
```

## Quick start

```
idocs init
idocs add auth src/auth.rs src/login.rs   # register doc
idocs                                      # check all
idocs -i                                   # TUI mode
```

## Commands

`idocs add <name> <sources...>` — register a doc tracking source files<br>
`idocs up <name>` — re-hash sources after manual review<br>
`idocs edit <name> --set/replace/lines` — edit doc content<br>
`idocs stale` — list only stale docs<br>
`idocs info <name>` — doc details with source status<br>
`idocs rm <name>` — remove a doc<br>
`idocs -i` — interactive TUI (two-panel viewer)<br>
`idocs --json` — machine-readable output


