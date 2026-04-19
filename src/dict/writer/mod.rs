use std::fmt;

use anyhow::Result;
use clap::ValueEnum;

use crate::{
    cli::{LangSpecs, Options},
    dict::{Dictionary, Intermediate},
    path::PathManager,
};

mod debug_forms;
mod html;
mod mdict_text;
mod renderer;
mod stardict;
mod yomitan;

use debug_forms::write_debug_forms;
use html::write_html;
use mdict_text::write_mdict_text;
use stardict::write_stardict;
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
    // Debug inflections (only useful for the main dict)
    DebugForms,
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
            Self::DebugForms => "debug-forms",
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
            Self::DebugForms => write_debug_forms(opts, pm, dict.to_yomitan(langs, irs)),
            Self::Tests => {
                irs.write(pm)?;
                write_yomitan_simple(opts, pm, dict.to_yomitan(langs, irs))?;
                Ok(())
            }
            Self::Skip => Ok(()),
        }
    }
}
