use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::{
    cli::Options,
    dict::writer::renderer::Renderer,
    lang::Lang,
    models::yomitan::{DetailedDefinition, YomitanDict, YomitanEntry},
    path::PathManager,
};

mod renderer;
use renderer::StardictRenderer;

type Definitions = Vec<(String, String)>;
type Synonyms = Vec<(String, Vec<String>)>;

pub fn write_stardict(
    source: Lang,
    target: Lang,
    _: &Options,
    pm: &PathManager,
    ydict: YomitanDict,
) -> Result<PathBuf> {
    let fname = format!("stardict-{source}-{target}");
    let dir_in_stage = pm.dir_in_stage(fname);
    _ = fs::create_dir_all(&dir_in_stage);

    let (definitions, synonyms) = extract_definitions_and_synonyms(ydict);
    write_to_stardict(&dir_in_stage, definitions, synonyms)?;

    Ok(dir_in_stage)
}

fn extract_definitions_and_synonyms(ydict: YomitanDict) -> (Definitions, Synonyms) {
    let definitions = ydict
        .term_info
        .into_iter()
        .map(YomitanEntry::TermInfo)
        .chain(ydict.term_meta.into_iter().map(YomitanEntry::TermMeta))
        .map(|entry| {
            (
                entry.term().to_string(),
                StardictRenderer::render_entry(&entry).into_string(),
            )
        })
        .collect();

    let synonyms = ydict
        .term_info_form
        .into_iter()
        .map(|entry| {
            let redirects_to = entry.term;
            let redirects_from = entry
                .definitions
                .into_iter()
                .map(|def| {
                    let DetailedDefinition::Inflection((from, _tags)) = def else {
                        panic!("forms must be made from inflections");
                    };
                    from
                })
                .collect::<Vec<_>>();
            (redirects_to, redirects_from)
        })
        .collect();

    (definitions, synonyms)
}

// In my machine (ubuntu, flatpak), dict should be @
// ~/.var/app/rocks.koreader.KOReader/config/koreader/data/dict
fn write_to_stardict(
    opath: &Path,
    mut definitions: Definitions,
    mut synonyms: Synonyms,
) -> Result<()> {
    let output_base = opath.join("wty");
    let dict_path = output_base.with_extension("dict");
    let idx_path = output_base.with_extension("idx");
    let syn_path = output_base.with_extension("syn");
    let ifo_path = output_base.with_extension("ifo");

    definitions.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    synonyms.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    let word_to_index: HashMap<&str, u32> = definitions
        .iter()
        .enumerate()
        .map(|(i, (word, _))| (word.as_str(), i as u32))
        .collect();

    let index_data = write_dict_file(&dict_path, &definitions)?;
    write_idx_file(&idx_path, &index_data)?;
    let syn_count = write_syn_file(&syn_path, &synonyms, &word_to_index)?;
    let idxfilesize = std::fs::metadata(&idx_path)?.len();
    write_ifo_file(&ifo_path, definitions.len(), syn_count, idxfilesize)?;

    let wordcount = definitions.len();

    tracing::debug!("Definitions: {wordcount}");
    tracing::debug!("Synonyms: {syn_count}");
    // println!("Files created:");
    // println!(" - {}", dict_path.display());
    // println!(" - {}", idx_path.display());
    // println!(" - {}", syn_path.display());
    // println!(" - {}", ifo_path.display());

    Ok(())
}

fn write_dict_file(dict_path: &Path, definitions: &Definitions) -> Result<Vec<Vec<u8>>> {
    let dict_file = File::create(dict_path)?;
    let mut dict_writer = BufWriter::new(dict_file);
    let mut index_data: Vec<Vec<u8>> = Vec::new();
    let mut offset: u32 = 0;

    for (word, definition) in definitions {
        let encoded_def = definition.as_bytes();
        dict_writer.write_all(encoded_def)?;
        let length = encoded_def.len() as u32;
        let mut entry = Vec::new();
        entry.extend_from_slice(word.as_bytes());
        entry.push(0u8);
        entry.extend_from_slice(&offset.to_be_bytes());
        entry.extend_from_slice(&length.to_be_bytes());
        index_data.push(entry);
        offset += length;
    }

    Ok(index_data)
}

fn write_idx_file(idx_path: &Path, index_data: &[Vec<u8>]) -> Result<()> {
    let idx_file = File::create(idx_path)?;
    let mut idx_writer = BufWriter::new(idx_file);
    for item in index_data {
        idx_writer.write_all(item)?;
    }
    Ok(())
}

fn write_syn_file(
    syn_path: &Path,
    synonyms: &Synonyms,
    word_to_index: &HashMap<&str, u32>,
) -> Result<u32> {
    let syn_file = File::create(syn_path)?;
    let mut syn_writer = BufWriter::new(syn_file);
    let mut seen = HashSet::new();
    let mut syn_count: u32 = 0;

    for (alias, targets) in synonyms {
        for target in targets {
            let Some(&idx) = word_to_index.get(target.as_str()) else {
                continue;
            };
            if !seen.insert((alias, idx)) {
                continue;
            }
            syn_writer.write_all(alias.as_bytes())?;
            syn_writer.write_all(&[0u8])?;
            syn_writer.write_all(&idx.to_be_bytes())?;
            syn_count += 1;
        }
    }

    Ok(syn_count)
}

fn write_ifo_file(
    ifo_path: &Path,
    wordcount: usize,
    syn_count: u32,
    idxfilesize: u64,
) -> Result<()> {
    let bookname = "wty";
    let ifo_content = format!(
        "StarDict's dict ifo file\nversion=3.0.0\nbookname={bookname}\nwordcount={wordcount}\nsynwordcount={syn_count}\nidxfilesize={idxfilesize}\nsametypesequence=h\n"
    );
    let mut ifo_file = File::create(ifo_path)?;
    ifo_file.write_all(ifo_content.as_bytes())?;
    Ok(())
}
