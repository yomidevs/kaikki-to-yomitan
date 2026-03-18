//! Dictionary release metadata.
//!
//! Scans the release `dict/` folder and produces a `release_metadata.json` summarizing
//! the size of each dictionary type, source language, and target language.
//!
//! Metadata is written to the `docs/` folder to be used in the downloads page.

use std::{collections::BTreeMap, path::Path};

use anyhow::Result;
use serde::ser::SerializeStruct;

use crate::utils::human_size;

#[derive(Debug, Default)]
struct SourceInfo {
    size: u64,
    count: u64,
    targets: BTreeMap<String, u64>,
}

#[derive(Debug, Default)]
struct TypeInfo {
    size: u64,
    count: u64,
    sources: BTreeMap<String, SourceInfo>,
}

type DictInfo = BTreeMap<String, TypeInfo>;

#[derive(Debug, Default)]
struct Metadata {
    size: u64,
    count: u64,
    dicts: DictInfo,
}

impl serde::Serialize for SourceInfo {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut state = s.serialize_struct("SourceInfo", 2)?;
        state.serialize_field("size", &human_size(self.size as f64))?;
        state.serialize_field("count", &self.count)?;
        state.serialize_field(
            "targets",
            &self
                .targets
                .iter()
                .map(|(k, v)| (k, human_size(*v as f64)))
                .collect::<BTreeMap<_, _>>(),
        )?;
        state.end()
    }
}

impl serde::Serialize for TypeInfo {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut state = s.serialize_struct("TypeInfo", 2)?;
        state.serialize_field("size", &human_size(self.size as f64))?;
        state.serialize_field("count", &self.count)?;
        state.serialize_field("sources", &self.sources)?;
        state.end()
    }
}

impl serde::Serialize for Metadata {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut state = s.serialize_struct("Metadata", 3)?;
        state.serialize_field("size", &human_size(self.size as f64))?;
        state.serialize_field("count", &self.count)?;
        state.serialize_field("dicts", &self.dicts)?;
        state.end()
    }
}

fn classify_dict(name: &str) -> &str {
    if name.ends_with("-ipa") {
        // wty-afb-en-ipa
        if name.chars().filter(|c| *c == '-').count() == 3 {
            "ipa"
        // wty-afb-ipa
        } else {
            "ipa-merged"
        }
    } else if name.ends_with("-gloss") {
        "glossary"
    } else {
        "main"
    }
}

fn scan_and_group(root_dir: &Path) -> Result<Metadata> {
    let mut meta = Metadata::default();

    for entry in walkdir::WalkDir::new(root_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "zip"))
    {
        let path = entry.path();

        // Expect <root>/<source>/<target>/<dict-name>.zip
        let rel = path.strip_prefix(root_dir)?;
        let parts: Vec<_> = rel.components().collect();
        if parts.len() != 3 {
            continue;
        }

        let source = parts[0].as_os_str().to_string_lossy().into_owned();
        let target = parts[1].as_os_str().to_string_lossy().into_owned();
        let filename = parts[2].as_os_str().to_string_lossy();

        let dict_type = classify_dict(filename.trim_end_matches(".zip"));
        let size = path.metadata()?.len();

        let type_entry = meta.dicts.entry(dict_type.to_string()).or_default();
        let src = type_entry.sources.entry(source).or_default();

        if !src.targets.contains_key(&target) {
            src.size += size;
            src.count += 1;
            src.targets.insert(target, size);
            type_entry.size += size;
            type_entry.count += 1;
            meta.size += size;
            meta.count += 1;
        }
    }

    Ok(meta)
}

pub fn write_dict_metadata(root_dir: &Path) -> Result<()> {
    let dict_dir = root_dir.join("dict");
    let metadata = scan_and_group(&dict_dir)?;
    let json = serde_json::to_string_pretty(&metadata)?;
    let out_path = Path::new("docs/release_metadata.json");
    std::fs::write(out_path, &json)?;
    println!("[meta] Dict metadata written to {}", out_path.display());
    Ok(())
}
