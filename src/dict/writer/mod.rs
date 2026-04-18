use std::{
    collections::HashSet,
    fmt,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Result;
use clap::ValueEnum;

use crate::{
    cli::{LangSpecs, Options},
    dict::{Dictionary, Intermediate, core::LabelledYomitanEntries, main::html::render_entry},
    models::yomitan::{DetailedDefinition, YomitanEntry},
    path::PathManager,
};

// mod html;
mod yomitan;
use yomitan::{write_yomitan, write_yomitan_simple};

#[derive(ValueEnum, Debug, Default, Clone, Copy)]
pub enum WriterFormat {
    // Yomitan zipped
    #[default]
    Yomitan,
    // Yomitan unzipped (json) + no metadata (index.json etc.)
    YomitanSimple,
    // Write irs as json
    Ir,
    // Simple html that matches the Yomitan structure
    Html,
    // Text file that can be build into mdict (via, f.e. mdict-utils)
    MdictText,
    // Stardict format
    Stardict,
    // Self::YomitanSimple + Self::Ir
    #[value(skip)]
    Tests,
    // Skip writing (for benchmarking etc.)
    Skip,
}

impl fmt::Display for WriterFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Yomitan => "yomitan",
            Self::YomitanSimple => "yomitan-simple",
            Self::Ir => "ir",
            Self::Html => "html",
            Self::MdictText => "mdict-text",
            Self::Stardict => "stardict",
            Self::Tests => "tests",
            Self::Skip => "skip",
        })
    }
}

impl WriterFormat {
    pub fn write<D: Dictionary>(
        self,
        dict: &D,
        langs: LangSpecs,
        opts: &Options,
        pm: &PathManager,
        irs: &D::I,
    ) -> Result<()> {
        match self {
            Self::Yomitan => write_yomitan(
                langs.source,
                langs.target,
                opts,
                pm,
                dict.to_yomitan(langs, irs),
            ),
            Self::YomitanSimple => write_yomitan_simple(opts, pm, dict.to_yomitan(langs, irs)),
            Self::Ir => irs.write(pm),
            Self::Html => write_html(opts, pm, dict.to_yomitan(langs, irs)),
            Self::MdictText => write_mdict_text(opts, pm, dict.to_yomitan(langs, irs)),
            Self::Stardict => write_stardict(opts, pm, dict.to_yomitan(langs, irs)),
            Self::Tests => {
                irs.write(pm)?;
                write_yomitan_simple(opts, pm, dict.to_yomitan(langs, irs))?;
                Ok(())
            }
            Self::Skip => Ok(()),
        }
    }
}

// TODO: move this
fn write_html(
    opts: &Options,
    pm: &PathManager,
    labelled_entries: Vec<LabelledYomitanEntries>,
) -> Result<()> {
    let dname = pm.dict_name_expanded();
    let filepath = format!("html/test-{}.html", dname);
    let filename = Path::new(&filepath);
    if let Some(parent) = filename.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(
        br#"<!DOCTYPE html>
            <html>
            <head>
            <meta charset="utf-8">
            <link rel="stylesheet" href="styles.css">
            </head>
            <body>
            "#,
    )?;

    // we don't care about labels for html
    for lentry in labelled_entries {
        for entry in lentry.entries {
            let html = render_entry(&entry).into_string();
            if opts.pretty {
                let pretty = prettify_html(&html);
                writer.write_all(pretty.as_bytes())?;
            } else {
                writer.write_all(html.as_bytes())?;
            }
        }
    }
    writer.write_all(br#"</body></html>"#)?;
    crate::utils::pretty_println_at_path("Wrote file", filename);

    Ok(())
}

// TODO: move this
fn write_mdict_text(
    opts: &Options,
    pm: &PathManager,
    labelled_entries: Vec<LabelledYomitanEntries>,
) -> Result<()> {
    let dname = pm.dict_name_expanded();
    let filepath = format!("html/test-{}.txt", dname);
    let filename = Path::new(&filepath);
    if let Some(parent) = filename.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    // we don't care about labels for html
    for lentry in labelled_entries {
        for entry in lentry.entries {
            writer.write_all(entry.term().as_bytes())?;
            writer.write_all(b"\n")?;

            let html = render_entry(&entry).into_string();
            // Requires a css include in each entry!
            writer.write_all(b"<link rel='stylesheet' href='styles.css' type='text/css'>")?;
            if opts.pretty {
                let pretty = prettify_html(&html);
                writer.write_all(pretty.as_bytes())?;
            } else {
                writer.write_all(html.as_bytes())?;
            }
            writer.write_all(b"\n")?;
            writer.write_all(b"</>")?;
            writer.write_all(b"\n")?;
        }
    }
    writer.write_all(br#"</body></html>"#)?;
    crate::utils::pretty_println_at_path("Wrote html", filename);

    Ok(())
}

// This is scuffed and introduces whitespace where it shouldn't, which then
// makes the css white-space: pre-line do some insane mangling.
// Stick with this for tests, without that css line, but remember to add it
// back in the final version.
// TODO: move this
fn prettify_html(html: &str) -> String {
    let mut result = String::new();
    let mut indent: usize = 0;
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => {
                if !in_tag {
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent)); // main offender
                }
                in_tag = true;
                result.push(ch);
            }
            '>' => {
                result.push(ch);
                in_tag = false;

                // Adjust indent based on tag type
                let last_tag = result.split('<').last().unwrap_or("");
                if last_tag.starts_with('/') {
                    indent = indent.saturating_sub(1);
                } else if !last_tag.ends_with('/') && !last_tag.starts_with('!') {
                    indent += 1;
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

use std::collections::HashMap;

type Definitions = Vec<(String, String)>;
type Synonyms = Vec<(String, Vec<String>)>;

fn write_stardict(
    _: &Options,
    _: &PathManager,
    labelled_entries: Vec<LabelledYomitanEntries>,
) -> Result<()> {
    let mut lemmas = vec![];
    let mut forms = vec![];
    // HACK:
    for lentry in labelled_entries {
        match lentry.label {
            "lemma" => lemmas = lentry.entries,
            "form" => forms = lentry.entries,
            other => unimplemented!("Didn't recognize label {}", other),
        }
    }

    let opath = Path::new("wty-stardict");
    let _ = fs::create_dir(&opath);

    let definitions = lemmas
        .into_iter()
        // the into below is for PreEscapedString > String
        .map(|entry| (entry.term().to_string(), render_entry(&entry).into()))
        .collect();
    let synonyms = forms
        .into_iter()
        .map(|entry| {
            let redirects_to = entry.term().to_string();
            let redirects_from = {
                let YomitanEntry::TermInfoForm(t) = entry else {
                    panic!("synonyms must be made from forms");
                };
                t.definitions
                    .into_iter()
                    .map(|def| {
                        let DetailedDefinition::Inflection((from, _tags)) = def else {
                            panic!("forms must be made from inflections");
                        };
                        from
                    })
                    .collect::<Vec<_>>()
            };
            (redirects_to, redirects_from)
        })
        .collect();
    write_to_stardict(opath, definitions, synonyms)?;
    Ok(())
}

// In my machine (ubuntu, flatpak), dict should be @
// ~/.var/app/rocks.koreader.KOReader/config/koreader/data/dict
//
// NOTE: (KOReader) Furigana is not supported. Nor are backlinks.
//
// TODO: Fix synonyms
fn write_to_stardict(
    opath: &Path,
    mut definitions: Definitions,
    mut synonyms: Synonyms,
) -> std::io::Result<()> {
    let output_base = opath.join("output");

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

    // Write .dict and build index
    let mut index_data: Vec<Vec<u8>> = Vec::new();
    let mut offset: u32 = 0;

    {
        let dict_file = File::create(&dict_path)?;
        let mut dict_writer = BufWriter::new(dict_file);

        for (word, definition) in &definitions {
            let encoded_def = definition.as_bytes();
            dict_writer.write_all(encoded_def)?;
            let length = encoded_def.len() as u32;

            let mut entry = Vec::new();
            entry.extend_from_slice(word.as_bytes());
            entry.push(0u8); // null terminator
            entry.extend_from_slice(&offset.to_be_bytes());
            entry.extend_from_slice(&length.to_be_bytes());

            index_data.push(entry);
            offset += length;
        }
    }

    // Write .idx
    {
        let idx_file = File::create(&idx_path)?;
        let mut idx_writer = BufWriter::new(idx_file);
        for item in &index_data {
            idx_writer.write_all(item)?;
        }
    }

    // Write .syn
    let mut syn_count: u32 = 0;
    {
        let syn_file = File::create(&syn_path)?;
        let mut syn_writer = BufWriter::new(syn_file);
        // We don't want to add redirections twice
        let mut seen = HashSet::new();
        // alias redirects to target!
        for (alias, targets) in &synonyms {
            for target in targets {
                let Some(&idx) = word_to_index.get(target.as_str()) else {
                    continue;
                };
                if !seen.insert((alias, idx)) {
                    continue;
                }
                // tracing::error!("Added {alias} | {target}");
                syn_writer.write_all(alias.as_bytes())?;
                syn_writer.write_all(&[0u8])?;
                syn_writer.write_all(&idx.to_be_bytes())?;
                syn_count += 1;
            }
        }
    }

    // Write .ifo
    let wordcount = definitions.len();
    let idxfilesize = std::fs::metadata(&idx_path)?.len();
    let bookname = "wty";

    let ifo_content = format!(
        "StarDict's dict ifo file\nversion=3.0.0\nbookname={bookname}\nwordcount={wordcount}\nsynwordcount={syn_count}\nidxfilesize={idxfilesize}\nsametypesequence=h\n"
    );

    {
        let mut ifo_file = File::create(&ifo_path)?;
        ifo_file.write_all(ifo_content.as_bytes())?;
    }

    println!("Done!");
    println!("Definitions: {wordcount}");
    println!("Synonyms: {syn_count}");
    println!("Files created:");
    println!(" - {}", dict_path.display());
    println!(" - {}", idx_path.display());
    println!(" - {}", syn_path.display());
    println!(" - {}", ifo_path.display());

    Ok(())
}
