use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::Result;
use pangloss::{
    AltEntry, AltMap, DataEntry, Definition, Entry, Glossary, GlossaryInfo, Writer,
    formats::mdict::MdictFormat,
};

use crate::{
    cli::Options,
    dict::writer::renderer::Renderer,
    models::yomitan::{DetailedDefinition, YomitanDict},
    path::PathManager,
};

mod renderer;
use renderer::MdictRenderer;

pub fn write_mdict(
    source: crate::lang::Lang,
    target: crate::lang::Lang,
    _: &Options,
    pm: &PathManager,
    ydict: YomitanDict,
) -> Result<PathBuf> {
    let dir_in_stage = pm.dir_in_stage("mdict");
    _ = fs::create_dir_all(&dir_in_stage);

    let dict_name = format!("wty-{source}-{target}");
    let mdx_path = dir_in_stage.join(format!("{dict_name}.mdx"));
    let glossary = build_glossary(&dict_name, ydict);
    debug_assert_eq!(glossary.css_files().count(), 1);

    MdictFormat::default().write(&mdx_path, &glossary)?;

    Ok(dir_in_stage)
}

fn build_glossary(dict_name: &str, ydict: YomitanDict) -> Glossary {
    // Aren't these duplicated in entries?
    let mut alt_map = AltMap::new();
    for entry in &ydict.term_bank_form {
        for def in &entry.definitions {
            let DetailedDefinition::Inflection((from, _tags)) = def else {
                panic!("forms must be made from inflections");
            };
            alt_map
                .entry(from.clone())
                .or_default()
                .push(AltEntry::only_term(entry.term.clone()));
        }
    }

    let entries: Vec<Entry> = ydict
        .into_iter_flat()
        .map(|entry| {
            Entry::new(
                entry.term().to_string(),
                Definition::Html(MdictRenderer::render_entry(&entry).into_string()),
            )
        })
        .collect();

    // In theory we could call this in pangloss
    const EXTRA_CSS_SC: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/styles_html.css"
    ));
    let data_entries = vec![DataEntry::new("styles.css", EXTRA_CSS_SC.to_vec())];

    let mut info = GlossaryInfo::new();
    info.insert("name", dict_name.to_string());

    Glossary {
        entries,
        data_entries,
        alt_map,
        info,
        ..Default::default()
    }
}
