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
    models::yomitan::{DetailedDefinition, YomitanDict, YomitanEntry},
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
    let fname = format!("mdict-{source}-{target}");
    let dir_in_stage = pm.dir_in_stage(&fname);
    fs::create_dir_all(&dir_in_stage)?;

    let mdx_path = dir_in_stage.join("wty.mdx");
    let dict_name = "wty-ja-ja";
    let glossary = build_glossary(dict_name, ydict);
    debug_assert_eq!(glossary.css_files().count(), 1);

    MdictFormat::default().write(&mdx_path, &glossary)?;

    Ok(dir_in_stage)
}

fn build_glossary(dict_name: &str, ydict: YomitanDict) -> Glossary {
    let entries: Vec<Entry> = ydict
        .term_info
        .into_iter()
        .map(YomitanEntry::TermInfo)
        .chain(ydict.term_meta.into_iter().map(YomitanEntry::TermMeta))
        .map(|entry| {
            Entry::new(
                entry.term().to_string(),
                Definition::Html(MdictRenderer::render_entry(&entry).into_string()),
            )
        })
        .collect();

    // Aren't these duplicated in entries?
    let mut alt_map = AltMap::new();
    for entry in ydict.term_info_form {
        let term = entry.term;
        for def in entry.definitions {
            let DetailedDefinition::Inflection((from, _tags)) = def else {
                panic!("forms must be made from inflections");
            };
            alt_map
                .entry(from)
                .or_default()
                .push(AltEntry::only_term(term.clone()));
        }
    }

    // In theory we could call this in pangloss
    const EXTRA_CSS_SC: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/styles_html.css"
    ));
    let data_entries = vec![DataEntry::new(
        "structured-content.css",
        EXTRA_CSS_SC.to_vec(),
    )];

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
