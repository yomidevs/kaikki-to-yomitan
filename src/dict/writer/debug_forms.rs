use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Result;

use crate::{
    cli::Options,
    models::yomitan::{DetailedDefinition, YomitanDict},
    path::PathManager,
};

pub fn write_debug_forms(_: &Options, pm: &PathManager, ydict: YomitanDict) -> Result<()> {
    let dname = pm.dict_name_expanded();
    let filepath = format!("html/test-{}.txt", dname);
    let filename = Path::new(&filepath);
    if let Some(parent) = filename.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    // NOTE: we are undoing work and going back to irs... but we can't replace
    // the yomitan dict with irs, since then we should assume that the irs can be
    // of ANY dictionary (and we only care about the main one)
    // ... and in theory, there is no guarantee that the irs format of the main
    // dictionary won't change, while this logic remains the same.
    let mut grouped_by: HashMap<String, Vec<String>> = HashMap::new();

    for entry in ydict.term_info_form {
        let term = entry.term.to_string();
        for def in entry.definitions {
            let DetailedDefinition::Inflection((from, _tags)) = def else {
                panic!("forms must be made from inflections");
            };
            let tos = grouped_by.entry(from).or_default();
            if !tos.contains(&term) {
                tos.push(term.clone());
            }
        }
    }

    for (from, tos) in &grouped_by {
        writer.write_all(from.as_bytes())?;
        writer.write_all(b"\n")?;
        for to in tos {
            writer.write_all(to.as_bytes())?;
            writer.write_all(b"\n")?;
        }
        writer.write_all(b"\n")?;
    }

    crate::utils::pretty_println_at_path("Wrote debug file", filename);

    Ok(())
}
