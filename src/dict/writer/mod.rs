use std::fmt;

use anyhow::Result;
use clap::ValueEnum;

use crate::{
    cli::{LangSpecs, Options},
    dict::{Dictionary, Intermediate},
    path::PathManager,
};

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
            Self::Tests => {
                irs.write(pm)?;
                write_yomitan_simple(opts, pm, dict.to_yomitan(langs, irs))?;
                Ok(())
            }
            Self::Skip => Ok(()),
        }
    }
}
