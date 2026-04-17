use std::fmt;

use anyhow::Result;
use clap::ValueEnum;

use crate::{
    cli::{LangSpecs, Options},
    dict::{Dictionary, Intermediate, main::html::render_entry},
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
            // Self::Html => todo!(),
            Self::Html => {
                use std::fs::File;
                use std::io::{BufWriter, Write};
                use std::path::Path;
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

                let labelled_entries = dict.to_yomitan(langs, irs);
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
                crate::utils::pretty_println_at_path("Wrote html", filename);
                Ok(())
            }
            Self::Tests => {
                irs.write(pm)?;
                write_yomitan_simple(opts, pm, dict.to_yomitan(langs, irs))?;
                Ok(())
            }
            Self::Skip => Ok(()),
        }
    }
}

// This is scuffed and introduces whitespace where it shouldn't, which then
// makes the css white-space: pre-line do some insane mangling.
// Stick with this for tests, without that css line, but remember to add it
// back in the final version.
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
