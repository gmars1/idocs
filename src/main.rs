mod check;
mod cmds;
mod index;
mod tui;

use std::io::{self, Read};

use anyhow::Result;
use clap::{Parser, Subcommand};

use cmds::*;

#[derive(Parser)]
#[command(
    name = "idocs",
    disable_help_subcommand = true,
    about = "Track which source files your docs reference and detect staleness.\n\n  idocs                           check all docs\n  idocs src/auth.rs               valid docs for a source file\n  idocs init                      initialize .idocs\n\n  idocs add \"auth\" src/auth.rs    register doc\n  idocs rm auth                   remove doc\n  idocs info auth                 show doc details\n\n  idocs up auth                   re-hash sources after review\n  idocs stale                     list only stale docs\n\n  idocs edit auth --set \"..\"                   replace content\n  idocs edit auth --lines 2-4 --text \"..\"      replace line range\n  idocs edit auth --replace \"x\" --with \"y\"    find-and-replace\n  idocs edit auth --rehash                     update hashes after edit\n\n  idocs -i                        interactive TUI (two-panel viewer)\n  idocs --json                    machine-readable output"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(
        value_name = "FILE",
        help = "Show valid docs that track this source file (e.g. src/auth.rs)"
    )]
    path: Option<String>,

    #[arg(short, long, global = true, help = "Interactive TUI mode")]
    interactive: bool,

    #[arg(long, global = true, help = "Machine-readable JSON output")]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize .idocs directory
    Init,
    /// Register a doc tracking source files
    Add { name: String, sources: Vec<String> },
    /// Re-hash source files for a doc
    Up { name: String },
    /// Print doc content
    Read { doc: String },
    /// Edit doc content (--set, --lines/--text, --replace/--with, or pipe)
    Edit {
        doc: String,
        #[arg(short, long)]
        set: Option<String>,
        #[arg(short, long)]
        lines: Option<String>,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        replace: Option<String>,
        #[arg(long)]
        with: Option<String>,
        #[arg(long)]
        rehash: bool,
    },
    /// Show doc details and source status
    Info { doc: String },
    /// Remove a doc
    Rm { doc: String },
    /// List only stale docs
    Stale,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.interactive {
        return tui::run();
    }

    match cli.command {
        Some(Commands::Init) => cmd_init()?,
        Some(Commands::Add { name, sources }) => {
            let content = if !atty::is(atty::Stream::Stdin) {
                let mut buf = String::new();
                io::stdin().read_to_string(&mut buf)?;
                if buf.is_empty() {
                    None
                } else {
                    Some(buf)
                }
            } else {
                None
            };
            cmd_add(&name, &sources, content.as_deref())?;
        }
        Some(Commands::Up { name }) => cmd_up(&name)?,
        Some(Commands::Read { doc }) => cmd_read(&doc)?,
        Some(Commands::Edit {
            doc,
            set,
            lines,
            text,
            replace,
            with,
            rehash,
        }) => cmd_edit(
            &doc,
            set.as_deref(),
            lines.as_deref(),
            text.as_deref(),
            replace.as_deref(),
            with.as_deref(),
            rehash,
        )?,
        Some(Commands::Info { doc }) => cmd_info(&doc, cli.json)?,
        Some(Commands::Rm { doc }) => cmd_rm(&doc)?,
        Some(Commands::Stale) => cmd_stale(cli.json)?,
        None => cmd_default(cli.path.as_deref(), cli.json)?,
    }

    Ok(())
}
