# Commands

All commands produce machine-readable output with the `--json` flag.

| Command | Description |
|---|---|
| `idocs` | Check all docs, show valid/stale |
| `idocs <file>` | Filter docs tracking a specific source file |
| `idocs init` | Initialize `.idocs` directory |
| `idocs add <name> <sources...>` | Register a doc tracking source files |
| `idocs rm <name>` | Remove a doc and its `.md` file |
| `idocs info <name>` | Show doc details with source status |
| `idocs up <name>` | Re-hash sources after review |
| `idocs stale` | List only stale docs |
| `idocs read <name>` | Print doc content |
| `idocs edit <name> --set "..."` | Replace entire doc content |
| `idocs edit <name> --lines N-M --text "..."` | Replace a range of lines |
| `idocs edit <name> --replace "x" --with "y"` | Find-and-replace in doc |
| `idocs edit <name> --rehash` | Update source hashes after editing |
| `idocs edit <name>` (stdin pipe) | Read new content from stdin |
