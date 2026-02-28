use anyhow::{Context, Ok, Result};
use serde::{Deserialize, Serialize};

use std::borrow::Cow;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use crate::Map;
use crate::cli::Options;
use crate::dict::writer::write_yomitan;
use crate::lang::{Edition, EditionSpec, Lang};
use crate::models::kaikki::WordEntry;
use crate::models::yomitan::YomitanEntry;
use crate::path::{PathKind, PathManager};
use crate::utils::pretty_print_at_path;
use crate::utils::skip_because_file_exists;

const CONSOLE_PRINT_INTERVAL: i32 = 10000;

// pub type E = Box<dyn Iterator<Item = YomitanEntry>>;
pub type E = Vec<YomitanEntry>;

// Used in tests to write separate files for lemmas/forms.
pub struct LabelledYomitanEntry {
    pub label: &'static str,
    pub entries: E,
}

impl LabelledYomitanEntry {
    pub fn new(
        label: &'static str,
        // entries: impl IntoIterator<Item = YomitanEntry> + 'static,
        entries: Vec<YomitanEntry>,
    ) -> Self {
        Self {
            label,
            // entries: Box::new(entries.into_iter()),
            entries,
        }
    }
}

/// Trait for Intermediate representation. Used for postprocessing (merge, etc.) and debugging via snapshots.
///
/// The simplest form is a Vec<YomitanEntry> if we don't want to do anything fancy, cf. `DGlossary`
pub trait Intermediate: Default {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// How to write `Self::I` to disk.
    ///
    /// Only called if `opts.save_temps` is set and `Dictionary::write_ir` returns true.
    #[allow(unused_variables)]
    fn write(&self, pm: &PathManager) -> Result<()> {
        Ok(())
    }
}

impl<T> Intermediate for Vec<T>
where
    T: Serialize,
{
    fn len(&self) -> usize {
        Self::len(self)
    }

    fn write(&self, pm: &PathManager) -> Result<()> {
        let writer_path = pm.dir_tidy().join("tidy.jsonl");
        let writer_file = File::create(&writer_path)?;
        let writer = BufWriter::new(&writer_file);
        if pm.opts.pretty {
            serde_json::to_writer_pretty(writer, self)?;
        } else {
            serde_json::to_writer(writer, self)?;
        }
        if !pm.opts.quiet {
            pretty_print_at_path("Wrote tidy", &writer_path);
        }
        Ok(())
    }
}

/// Trait to abstract the process of making a dictionary.
pub trait Dictionary {
    type A: TryInto<PathManager, Error = anyhow::Error>;
    type I: Intermediate;

    /// Whether to keep or not this entry.
    fn keep_if(&self, source: Lang, entry: &WordEntry) -> bool;

    /// Whether we can quickly probe a jsonline to avoid a full deserialization.
    ///
    /// Only used for the main dictionary. It probes on source Lang.
    fn supports_probe(&self) -> bool {
        false
    }

    // NOTE: Maybe we can get rid of this (blocked by mutable behaviour of the main dictionary).
    //
    /// How to preprocess a `WordEntry`. Everything that mutates `entry` should go here.
    #[allow(unused_variables)]
    fn preprocess(&self, langs: Langs, entry: &mut WordEntry, opts: &Options, irs: &mut Self::I) {}

    /// How to transform a `WordEntry` into intermediate representation.
    ///
    /// Most dictionaries only make *at most one* `Self::I` from a `WordEntry`.
    fn process(&self, langs: Langs, entry: &WordEntry, irs: &mut Self::I);

    /// Console message for found irs. It is customized for the main dictionary.
    #[allow(unused_variables)]
    fn found_ir_message(&self, key: &LangsKey, irs: &Self::I) {
        println!("Found {} irs", irs.len());
    }

    /// Whether to write or not `Self::I` to disk.
    ///
    /// Compare to `save_temp`, that rules if `Self::I` AND the `term_banks` are written to disk.
    ///
    /// This is mainly a debug function, in order to allow not writing the ir `Self::I` to disk for
    /// minor dictionaries in the testsuite. It is only set to true in the main dictionary.
    fn write_ir(&self) -> bool {
        false
    }

    /// How to postprocess the intermediate representation.
    ///
    /// This can be implemented to merge entries from different edition, to postprocess tags etc.
    #[allow(unused_variables)]
    fn postprocess(&self, irs: &mut Self::I) {}

    /// How to convert `Self::I` into one or more yomitan entries.
    fn to_yomitan(&self, langs: Langs, irs: Self::I) -> Vec<LabelledYomitanEntry>;
}

fn rejected(entry: &WordEntry, opts: &Options) -> bool {
    opts.reject.iter().any(|(k, v)| k.field_value(entry) == v)
        || !opts.filter.iter().all(|(k, v)| k.field_value(entry) == v)
}

use crate::dict::{DGlossary, DGlossaryExtended, DIpa, DIpaMerged, DMain};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct LangsKey {
    pub edition: EditionSpec,
    pub source: Lang,
    pub target: Lang,
}

/// Maps an iteration Langs to its aggregation key.
///
/// Used by merged dictionaries to combine data across editions.
pub trait AggregationKey {
    fn langs_to_key(&self, langs: Langs) -> LangsKey {
        LangsKey {
            edition: EditionSpec::One(langs.edition),
            source: langs.source,
            target: langs.target,
        }
    }
}

impl AggregationKey for DMain {}
impl AggregationKey for DIpa {}
impl AggregationKey for DGlossary {}

impl AggregationKey for DIpaMerged {
    // Collapse all editions into one logical key
    fn langs_to_key(&self, langs: Langs) -> LangsKey {
        LangsKey {
            edition: EditionSpec::All,
            source: langs.source,
            target: langs.target,
        }
    }
}

impl AggregationKey for DGlossaryExtended {
    // Collapse all editions into one logical key
    fn langs_to_key(&self, langs: Langs) -> LangsKey {
        LangsKey {
            edition: EditionSpec::All,
            source: langs.source,
            target: langs.target,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Langs {
    pub edition: Edition,
    pub source: Lang,
    pub target: Lang,
}

impl Langs {
    pub const fn new(edition: Edition, source: Lang, target: Lang) -> Self {
        Self {
            edition,
            source,
            target,
        }
    }
}

impl fmt::Debug for Langs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Langs")
            .field(&self.edition)
            .field(&self.source)
            .field(&self.target)
            .finish()
    }
}

pub fn find_or_download_jsonl(
    edition: Edition,
    lang: Option<Lang>,
    pm: &PathManager,
) -> Result<PathBuf> {
    let paths_candidates = pm.dataset_paths(edition, lang);
    let kinds_to_check = [PathKind::Unfiltered, PathKind::Filtered];
    let of_kind: Vec<_> = paths_candidates
        .inner
        .iter()
        .filter(|p| kinds_to_check.contains(&p.kind))
        .collect();

    if !pm.opts.redownload
        && let Some(existing) = of_kind.iter().find(|p| p.path.exists())
    {
        if !pm.opts.quiet {
            skip_because_file_exists("download", &existing.path);
        }
        return Ok(existing.path.clone());
    }

    let path = &of_kind
        .iter()
        .next_back()
        .unwrap_or_else(|| {
            panic!(
                "No path available, \
             for edition={edition:?} and lang={lang:?} | {paths_candidates:?}"
            )
        })
        .path;

    // TODO: remove this once it's done: it prevents downloading in the testsuite
    // anyhow::bail!(
    //     "Downloading is disabled but JSONL file was not found @ {}",
    //     path.display()
    // );

    #[cfg(feature = "html")]
    crate::download::download_jsonl(edition, path, false)?;

    Ok(path.clone())
}

pub fn iter_datasets(pm: &PathManager) -> impl Iterator<Item = Result<(Edition, PathBuf)>> + '_ {
    let (edition_pm, source_pm, _) = pm.langs();

    edition_pm.variants().into_iter().map(move |edition| {
        let path_jsonl = find_or_download_jsonl(edition, Some(source_pm), pm)?;
        tracing::debug!("edition: {edition}, path: {}", path_jsonl.display());

        Ok((edition, path_jsonl))
    })
}

#[derive(Deserialize)]
#[serde(default)]
struct LangCodeProbe<'a> {
    #[serde(borrow)]
    lang_code: Cow<'a, str>,
}

impl Default for LangCodeProbe<'_> {
    fn default() -> Self {
        Self {
            lang_code: Cow::Borrowed(""),
        }
    }
}

pub fn make_dict<D: Dictionary + AggregationKey>(dict: D, raw_args: D::A) -> Result<()> {
    let pm: &PathManager = &raw_args.try_into()?;
    let (_, source_pm, target_pm) = pm.langs();
    let opts = &pm.opts;

    pm.setup_dirs()?;

    let capacity = 256 * (1 << 10); // default is 8 * (1 << 10) := 8KB
    let mut line = Vec::with_capacity(1 << 10);
    // (source, target) -> D::I
    let mut irs_map: Map<LangsKey, D::I> = Map::default();

    for pair in iter_datasets(pm) {
        let (edition, path_jsonl) = pair?;

        let reader_file = File::open(&path_jsonl)?;
        let mut reader = BufReader::with_capacity(capacity, reader_file);

        let mut line_count = 0;
        let mut accepted_count = 0;

        loop {
            line.clear();
            if reader.read_until(b'\n', &mut line)? == 0 {
                break; // EOF
            }

            line_count += 1;

            if !opts.quiet && line_count % CONSOLE_PRINT_INTERVAL == 0 {
                print!("Processed {line_count} lines...\r");
                std::io::stdout().flush()?;
            }

            if dict.supports_probe() {
                let probe: LangCodeProbe = serde_json::from_slice(&line)
                    .with_context(|| "Error decoding JSON @ make_dict (lang_code prefilter)")?;
                if source_pm.as_ref() != probe.lang_code.as_ref() {
                    continue;
                }
            }

            let mut entry: WordEntry =
                serde_json::from_slice(&line).with_context(|| "Error decoding JSON @ make_dict")?;

            if rejected(&entry, opts) {
                continue;
            }

            accepted_count += 1;
            if accepted_count == opts.first {
                break;
            }

            let langs = Langs {
                edition,
                source: source_pm,
                target: target_pm,
            };

            if dict.keep_if(langs.source, &entry) {
                let key = dict.langs_to_key(langs);
                let irs = irs_map.entry(key).or_default();
                dict.preprocess(langs, &mut entry, opts, irs);
                dict.process(langs, &entry, irs);
            }
        }

        if !opts.quiet {
            println!("Processed {line_count} lines. Accepted {accepted_count} lines.");
        }

        // tracing::debug!(
        //     "After {edition}: irs_map has {} keys, {} total entries",
        //     irs_map.len(),
        //     irs_map.values().map(|ir| ir.len()).sum::<usize>()
        // );
    }

    if irs_map.len() > 1 {
        tracing::debug!("Matrix ({}): {:?}", irs_map.len(), irs_map.keys());
    }

    for (key, mut irs) in irs_map {
        if !opts.quiet {
            dict.found_ir_message(&key, &irs);
        }

        if irs.is_empty() {
            continue;
        }

        dict.postprocess(&mut irs);

        if opts.save_temps && dict.write_ir() {
            irs.write(pm)?;
        }

        if !opts.skip_yomitan {
            let mut pm2 = pm.clone();
            let source = key.source;
            let target = key.target;
            pm2.set_source(source);
            pm2.set_target(target);
            pm2.setup_dirs()?;
            tracing::trace!("calling to_yomitan with (source={source}, target={target})",);
            let labelled_entries = match key.edition {
                EditionSpec::All => {
                    // HACK: we don't use the edition for IpaMerged: use a dummy for now
                    let langs = Langs::new(Edition::Zh, key.source, key.target);
                    dict.to_yomitan(langs, irs)
                }
                EditionSpec::One(edition) => {
                    let langs = Langs::new(edition, key.source, key.target);
                    dict.to_yomitan(langs, irs)
                }
            };
            write_yomitan(source, target, opts, &pm2, labelled_entries)?;
        }
    }

    Ok(())
}
// TODO: rename this to make_dicts when done, and keep the original
