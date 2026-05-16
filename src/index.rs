use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

pub const IDOCS_DIR: &str = ".idocs";
pub const INDEX_FILE: &str = "sources.json";
pub const DOCS_DIR: &str = "docs";

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    pub docs: BTreeMap<String, DocEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocEntry {
    pub name: String,
    pub file: String,
    pub sources: BTreeMap<String, String>,
}

#[derive(Serialize)]
pub struct JsonOutput {
    pub valid: Vec<JsonDoc>,
    pub stale: Vec<JsonDoc>,
}

#[derive(Serialize)]
pub struct JsonDoc {
    pub name: String,
    pub bad: Vec<JsonBad>,
}

#[derive(Serialize)]
pub struct JsonBad {
    pub file: String,
    pub status: String,
}

pub fn project_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let mut p = Some(cwd.as_path());
    while let Some(d) = p {
        if d.join(IDOCS_DIR).exists() {
            return Ok(d.to_path_buf());
        }
        p = d.parent();
    }
    bail!("no .idocs found (run 'idocs init' first)");
}

pub fn load_index(root: &Path) -> Result<Index> {
    let p = root.join(IDOCS_DIR).join(INDEX_FILE);
    if !p.exists() {
        return Ok(Index {
            docs: BTreeMap::new(),
        });
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(&p)?)?)
}

pub fn save_index(root: &Path, idx: &Index) -> Result<()> {
    std::fs::write(
        root.join(IDOCS_DIR).join(INDEX_FILE),
        serde_json::to_string_pretty(idx)?,
    )?;
    Ok(())
}

pub fn doc_id(name: &str) -> String {
    name.to_lowercase().replace(' ', "_").replace('/', "_")
}
