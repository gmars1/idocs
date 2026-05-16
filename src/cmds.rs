use std::collections::BTreeMap;
use std::io::{self, Read};

use anyhow::{bail, Context, Result};

use crate::check::{check_all, file_sha256, DocState};
use crate::index::*;

// ── Commands ────────────────────────────────────────────────────────

pub fn cmd_init() -> Result<()> {
    let root = std::env::current_dir()?;
    std::fs::create_dir_all(root.join(IDOCS_DIR).join(DOCS_DIR))?;
    let p = root.join(IDOCS_DIR).join(INDEX_FILE);
    if !p.exists() {
        save_index(
            &root,
            &Index {
                docs: BTreeMap::new(),
            },
        )?;
    }
    eprintln!("  \x1b[32m\u{2713}\x1b[0m .idocs initialized");
    Ok(())
}

pub fn cmd_add(name: &str, sources: &[String], content: Option<&str>) -> Result<()> {
    let root = project_root()?;
    let mut idx = load_index(&root)?;

    let id = doc_id(name);
    let doc_dir = root.join(IDOCS_DIR).join(DOCS_DIR);
    std::fs::create_dir_all(&doc_dir)?;
    let doc_path = doc_dir.join(format!("{}.md", id));

    if let Some(c) = content {
        std::fs::write(&doc_path, c)?;
    } else if !doc_path.exists() {
        std::fs::write(&doc_path, format!("# {}\n\n", name))?;
    }

    let rel = doc_path
        .strip_prefix(&root)
        .unwrap_or(&doc_path)
        .display()
        .to_string();

    let mut src_map = match idx.docs.get(&id) {
        Some(existing) => existing.sources.clone(),
        None => BTreeMap::new(),
    };
    for s in sources {
        let full = root.join(s);
        match file_sha256(&full) {
            Ok(h) => {
                src_map.insert(s.clone(), h);
            }
            Err(_) => eprintln!("  warning: '{}' not found, skipped", s),
        }
    }

    let count = src_map.len();
    idx.docs.insert(
        id,
        DocEntry {
            name: name.to_string(),
            file: rel,
            sources: src_map,
        },
    );

    save_index(&root, &idx)?;
    eprintln!(
        "  \x1b[32m\u{2713}\x1b[0m added '{}' tracking {} source(s)",
        name, count
    );
    Ok(())
}

pub fn cmd_up(name: &str) -> Result<()> {
    let root = project_root()?;
    let mut idx = load_index(&root)?;
    let id = doc_id(name);
    let entry = idx
        .docs
        .get_mut(&id)
        .with_context(|| format!("doc '{}' not found", name))?;

    let mut changed = 0;
    for s in entry.sources.keys().cloned().collect::<Vec<_>>() {
        let full = root.join(&s);
        match file_sha256(&full) {
            Ok(h) => {
                if entry.sources.get(&s) != Some(&h) {
                    entry.sources.insert(s.clone(), h);
                    changed += 1;
                }
            }
            Err(_) => {
                entry.sources.remove(&s);
                changed += 1;
                eprintln!("  warning: '{}' no longer exists, removed", s);
            }
        }
    }

    save_index(&root, &idx)?;
    eprintln!(
        "  \x1b[32m\u{2713}\x1b[0m updated hashes for '{}' ({} source(s))",
        name, changed
    );
    Ok(())
}

pub fn cmd_rm(name: &str) -> Result<()> {
    let root = project_root()?;
    let mut idx = load_index(&root)?;
    let id = doc_id(name);
    idx.docs
        .remove(&id)
        .with_context(|| format!("doc '{}' not found", name))?;
    let doc_path = root
        .join(IDOCS_DIR)
        .join(DOCS_DIR)
        .join(format!("{}.md", id));
    if doc_path.exists() {
        std::fs::remove_file(&doc_path)?;
    }
    save_index(&root, &idx)?;
    eprintln!("  \x1b[32m\u{2713}\x1b[0m removed '{}'", name);
    Ok(())
}

pub fn cmd_info(name: &str, json: bool) -> Result<()> {
    let root = project_root()?;
    let idx = load_index(&root)?;
    let id = doc_id(name);
    let entry = idx
        .docs
        .get(&id)
        .with_context(|| format!("doc '{}' not found", name))?;

    if json {
        let mut sources = serde_json::Map::new();
        for (src, old_hash) in &entry.sources {
            let full = root.join(src);
            let status: &str = match file_sha256(&full) {
                Ok(h) if h == *old_hash => "valid",
                Ok(_) => "modified",
                Err(_) => "deleted",
            };
            sources.insert(
                src.clone(),
                serde_json::json!({"status": status, "stored_hash": old_hash}),
            );
        }
        let out = serde_json::json!({
            "name": entry.name,
            "file": entry.file,
            "sources": sources,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("  name:    {}", entry.name);
    println!("  file:    {}", entry.file);
    println!("  sources:");
    for (src, old_hash) in &entry.sources {
        let full = root.join(src);
        let status = match file_sha256(&full) {
            Ok(h) if h == *old_hash => format!("\x1b[32mvalid\x1b[0m"),
            Ok(_) => format!("\x1b[31mmodified\x1b[0m"),
            Err(_) => format!("\x1b[31mdeleted\x1b[0m"),
        };
        println!("    {}  {}", status, src);
    }
    Ok(())
}

pub fn cmd_read(name: &str) -> Result<()> {
    let root = project_root()?;
    let idx = load_index(&root)?;
    let id = doc_id(name);
    let entry = idx
        .docs
        .get(&id)
        .with_context(|| format!("doc '{}' not found", name))?;
    print!("{}", std::fs::read_to_string(root.join(&entry.file))?);
    Ok(())
}

pub fn cmd_edit(
    name: &str,
    set: Option<&str>,
    lines: Option<&str>,
    text: Option<&str>,
    replace: Option<&str>,
    with: Option<&str>,
    rehash: bool,
) -> Result<()> {
    let root = project_root()?;
    let idx = load_index(&root)?;
    let id = doc_id(name);
    let entry = idx
        .docs
        .get(&id)
        .with_context(|| format!("doc '{}' not found", name))?;
    let path = root.join(&entry.file);

    let new_content: String = if let Some(c) = set {
        c.to_string()
    } else if let (Some(range), Some(replacement)) = (lines, text) {
        let content = std::fs::read_to_string(&path)?;
        let lines_vec: Vec<&str> = content.split('\n').collect();
        let (start, end) = parse_range(range, lines_vec.len())?;
        let start_idx = start - 1;
        let end_idx = end.min(lines_vec.len());
        let mut new = Vec::new();
        for (_, line) in lines_vec.iter().enumerate().take(start_idx) {
            new.push(*line);
        }
        for l in replacement.lines() {
            new.push(l);
        }
        for i in end_idx..lines_vec.len() {
            new.push(lines_vec[i]);
        }
        new.join("\n")
    } else if let (Some(old), Some(new_text)) = (replace, with) {
        std::fs::read_to_string(&path)?.replace(old, new_text)
    } else if !atty::is(atty::Stream::Stdin) {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        if buf.is_empty() {
            bail!("empty content from stdin (pipe content or use --set/--lines/--replace)");
        }
        buf
    } else {
        bail!("no edit mode specified. Use --set, --lines/--text, --replace/--with, or pipe content via stdin");
    };

    std::fs::write(&path, &new_content)?;
    if rehash {
        cmd_up(name)?;
    }
    Ok(())
}

fn parse_range(s: &str, total: usize) -> Result<(usize, usize)> {
    if let Some(end_str) = s.split('-').nth(1) {
        let start: usize = s.split('-').next().unwrap().parse()?;
        let end = if end_str.is_empty() {
            total
        } else {
            end_str.parse()?
        };
        if start < 1 || end > total || start > end {
            bail!("invalid range '{}' (file has {} lines)", s, total);
        }
        Ok((start, end))
    } else {
        let n: usize = s.parse()?;
        if n < 1 || n > total {
            bail!("line {} out of range (file has {} lines)", n, total);
        }
        Ok((n, n))
    }
}

pub fn cmd_default(filter: Option<&str>, json: bool) -> Result<()> {
    let root = project_root()?;
    let (valid, stale) = check_all(&root, filter)?;

    if filter.is_some() {
        if json {
            let out: Vec<&DocState> = valid.iter().chain(stale.iter()).collect();
            let j: Vec<serde_json::Value> = out
                .iter()
                .map(|d| {
                    serde_json::json!({
                        "name": d.name,
                        "status": if d.bad.is_empty() { "valid" } else { "stale" },
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&j)?);
        } else {
            for d in &valid {
                println!("  \x1b[32m\u{2713}\x1b[0m {}", d.name);
            }
            if valid.is_empty() {
                println!("  no valid docs for '{}'", filter.unwrap());
            }
        }
        return Ok(());
    }

    if json {
        let out = JsonOutput {
            valid: valid
                .into_iter()
                .map(|d| JsonDoc {
                    name: d.name,
                    bad: vec![],
                })
                .collect(),
            stale: stale
                .into_iter()
                .map(|d| JsonDoc {
                    bad: d
                        .bad
                        .into_iter()
                        .map(|(f, st)| JsonBad {
                            file: f,
                            status: st,
                        })
                        .collect(),
                    name: d.name,
                })
                .collect(),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    if valid.is_empty() && stale.is_empty() {
        println!("  no docs  (\u{2190}  run \x1b[1midocs add\x1b[0m)");
        return Ok(());
    }

    for d in &valid {
        println!("  \x1b[32m\u{2713}\x1b[0m {}", d.name);
    }
    for d in &stale {
        println!("  \x1b[31m\u{2717}\x1b[0m {}", d.name);
        for (sf, why) in &d.bad {
            println!("      \x1b[31m{}\x1b[0m: {}", why, sf);
        }
    }
    println!("  {} valid  |  {} stale", valid.len(), stale.len());
    Ok(())
}

pub fn cmd_stale(json: bool) -> Result<()> {
    let root = project_root()?;
    let (_, stale) = check_all(&root, None)?;

    if stale.is_empty() {
        if json {
            println!("[]");
        } else {
            println!("  no stale docs");
        }
        return Ok(());
    }

    if json {
        let out: Vec<JsonDoc> = stale
            .into_iter()
            .map(|d| JsonDoc {
                name: d.name,
                bad: d
                    .bad
                    .into_iter()
                    .map(|(f, st)| JsonBad {
                        file: f,
                        status: st,
                    })
                    .collect(),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        for d in &stale {
            println!("  \x1b[31m\u{2717}\x1b[0m {}", d.name);
            for (sf, why) in &d.bad {
                println!("      \x1b[31m{}\x1b[0m: {}", why, sf);
            }
        }
    }
    Ok(())
}
