use anyhow::Result;

use crate::{
    cli::{LangSpecs, Options, WriterFormat},
    dict::{Dictionary, Intermediate},
    path::PathManager,
};

mod yomitan;
use yomitan::write_yomitan;

/// WARN:
/// - The issues: we want to write both for tests
/// - Decoupling IR writing and dict writing is hard
///   - behaviour against a bunch of flags (this should be easy to fix)
/// - Multiple writers require clone
///   - but irs.write() doesnt???? (because of references)
///     - >>> this is the convention, we should do the same!
///     - hard because every to_yomitan implementor uses the owned data
///       to avoid clones
/// - [ ] Sort out the flags that became useless (and therefore the docs too...)
///   - [ ] Maybe separate WriteFormat with WriteFormatSpec

/// WARN:
/// The issue: we want to write both Yomitan and IRS for tests
/// 1. Decoupling IR writing and dict writing is hard
/// 2. Multiple writers require clone
///   - but irs.write() doesnt???? (because of references)
///     - >>> this is the convention, we should do the same!
///     - hard because every to_yomitan implementor uses the owned data
///       to avoid clones
/// - [ ] Sort out the flags that became useless (and therefore the docs too...)
///   - [ ] Maybe separate WriteFormat with WriteFormatSpec (same issue as 2.)

// we impl write over the cli enum
impl WriterFormat {
    /// Write using this format (requires dictionary D to support all formats)
    pub fn write<D: Dictionary>(
        &self,
        dict: D,
        langs: LangSpecs,
        opts: &Options,
        pm: &PathManager,
        irs: D::I,
    ) -> Result<()> {
        match self {
            WriterFormat::Yomitan => YomitanWriter.write(dict, langs, opts, pm, irs),
            WriterFormat::None => IrWriter.write(dict, langs, opts, pm, irs),
        }
    }
}

/// Trait for dictionaries that can be written in a specific format
trait SupportsFormat<D: Dictionary> {
    fn write(
        &self,
        dict: D,
        langs: LangSpecs,
        opts: &Options,
        pm: &PathManager,
        irs: D::I,
    ) -> Result<()>;
}

struct YomitanWriter;
struct IrWriter;

// Update YomitanWriter to only require ToYomitan (not full Writer trait)
impl<D: Dictionary> SupportsFormat<D> for YomitanWriter {
    fn write(
        &self,
        dict: D,
        langs: LangSpecs,
        opts: &Options,
        pm: &PathManager,
        irs: D::I,
    ) -> Result<()> {
        let labelled_entries = dict.to_yomitan(langs, irs);
        write_yomitan(langs.source, langs.target, opts, pm, labelled_entries)
    }
}

// IrWriter works with any Dictionary
impl<D: Dictionary> SupportsFormat<D> for IrWriter {
    fn write(&self, _: D, _: LangSpecs, _: &Options, pm: &PathManager, irs: D::I) -> Result<()> {
        irs.write(pm)
    }
}
