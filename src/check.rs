use std::path::Path;

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::index::load_index;

pub struct DocState {
    pub name: String,
    pub bad: Vec<(String, String)>,
}

pub fn file_sha256(path: &Path) -> Result<String> {
    let mut h = Sha256::new();
    h.update(&std::fs::read(path)?);
    Ok(format!("{:x}", h.finalize()))
}

pub fn check_all(root: &Path, filter: Option<&str>) -> Result<(Vec<DocState>, Vec<DocState>)> {
    let idx = load_index(root)?;
    let mut valid = Vec::new();
    let mut stale = Vec::new();

    for (_, entry) in &idx.docs {
        let mut bad: Vec<(String, String)> = Vec::new();

        for (src_path, old_hash) in &entry.sources {
            if let Some(pf) = filter {
                if !src_path.starts_with(pf) {
                    continue;
                }
            }
            let full = root.join(src_path);
            let current = match file_sha256(&full) {
                Ok(h) => h,
                Err(_) => {
                    bad.push((src_path.clone(), "deleted".into()));
                    continue;
                }
            };
            if current != *old_hash {
                bad.push((src_path.clone(), "modified".into()));
            }
        }

        if filter.is_some()
            && bad.is_empty()
            && !entry.sources.keys().any(|s| s.starts_with(filter.unwrap()))
        {
            continue;
        }

        let ds = DocState {
            name: entry.name.clone(),
            bad,
        };
        if ds.bad.is_empty() {
            valid.push(ds);
        } else {
            stale.push(ds);
        }
    }

    Ok((valid, stale))
}
