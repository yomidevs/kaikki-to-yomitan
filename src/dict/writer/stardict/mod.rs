use std::{
    fs::{self},
    path::PathBuf,
};

use anyhow::Result;
use pangloss::{
    AltEntry, AltMap, Definition, Entry, Glossary, GlossaryInfo, Writer,
    formats::stardict::StardictFormat,
};

use crate::{
    cli::Options,
    dict::writer::renderer::Renderer,
    lang::Lang,
    models::yomitan::{DetailedDefinition, YomitanDict, YomitanEntry},
    path::PathManager,
};

mod renderer;
use renderer::StardictRenderer;

pub fn write_stardict(
    source: Lang,
    target: Lang,
    _: &Options,
    pm: &PathManager,
    ydict: YomitanDict,
) -> Result<PathBuf> {
    let dir_in_stage = pm.dir_in_stage("stardict");
    _ = fs::create_dir_all(&dir_in_stage);

    let dict_name = format!("wty-{source}-{target}");
    let ifo_path = dir_in_stage.join(format!("{dict_name}.ifo"));
    let glossary = build_glossary(&dict_name, ydict);

    StardictFormat.write(&ifo_path, &glossary)?;

    Ok(dir_in_stage)
}

// Build a Glossary out of the html rendered by StardictRenderer
fn build_glossary(dict_name: &str, ydict: YomitanDict) -> Glossary {
    let entries: Vec<Entry> = ydict
        .term_info
        .into_iter()
        .map(YomitanEntry::TermInfo)
        .chain(ydict.term_meta.into_iter().map(YomitanEntry::TermMeta))
        .map(|entry| {
            Entry::new(
                entry.term().to_string(),
                Definition::Html(StardictRenderer::render_entry(&entry).into_string()),
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

    let mut info = GlossaryInfo::new();
    info.insert("name", dict_name.to_string());
    info.insert("sametypesequence", "h".to_string());

    Glossary {
        entries,
        alt_map,
        info,
        ..Default::default()
    }
}
