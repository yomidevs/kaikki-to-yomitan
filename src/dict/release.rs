//! Build a dictionary release.
//!
//! Index extracting and publishing are done in python via release.py.
//!
//! Command to limit memory usage (linux):
//! systemd-run --user --scope -p MemoryMax=24G -p MemoryHigh=24G cargo run -r -- release -v

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::Result;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use rkyv::Archived;
use rusqlite::{Connection, params};

use crate::{
    cli::{
        DictName, GlossaryArgs, GlossaryExtendedArgs, GlossaryExtendedLangs, GlossaryLangs,
        IpaArgs, IpaMergedArgs, IpaMergedLangs, MainArgs, MainLangs, Options,
    },
    dict::{
        DGlossary, DGlossaryExtended, DIpa, DIpaMerged, DMain, Dictionary, Intermediate, Langs,
        find_or_download_jsonl, iter_datasets, writer::write_yomitan,
    },
    lang::{Edition, EditionSpec, Lang},
    models::kaikki::WordEntry,
    path::PathManager,
};

pub fn release() -> Result<()> {
    // let editions = [Edition::En, Edition::De, Edition::Fr];

    let mut editions = Edition::all();
    // English is the bottleneck, and while I'm not entirely sure this works, getting to work asap
    // with English dictionaries should make things faster. This puts English first.
    editions.sort_by_key(|ed| i32::from(*ed != Edition::En));

    println!("Making release with {} editions", editions.len());
    println!(
        "- {}",
        editions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );

    // First, download all jsonlines to prevent races when creating databases.
    //
    // NOTE: For some reason this takes time even when db are init, why?
    download_and_create_db(&editions);

    let start = Instant::now();

    editions.par_iter().for_each(|edition| {
        release_main(*edition);
        release_ipa(*edition);
        release_ipa_merged(*edition);
        release_glossary(*edition);
    });

    // let sources = Lang::all();
    // let sources = [Lang::Afb];
    // sources.par_iter().for_each(|source| {
    //     release_glossary_extended(*source);
    // });

    let elapsed = start.elapsed();
    println!("Finished dictionaries in {elapsed:.2?}");

    Ok(())
}

fn download_and_create_db(editions: &[Edition]) {
    let start = Instant::now();

    editions.par_iter().for_each(|edition| {
        let now = Instant::now();
        let args = MainArgs {
            langs: MainLangs {
                source: (*edition).into(),
                target: *edition,
            },
            dict_name: DictName::default(),
            options: Options {
                quiet: false,
                root_dir: "data".into(),
                ..Default::default()
            },
        };
        let pm: &PathManager = &args.try_into().unwrap();
        let path_jsonl = find_or_download_jsonl(*edition, None, pm).unwrap();
        println!("Finished download for {edition} ({:.2?})", now.elapsed());
        let _ = WiktextractDb::create(*edition, path_jsonl).unwrap();
        println!("Finished database for {edition} ({:.2?})", now.elapsed());
    });

    println!("Finished download & db creation in {:.2?}", start.elapsed());
}

// Pretty print utility
fn pp(dict_name: &str, first_lang: Lang, second_lang: Lang, time: Instant) {
    // Printing sizes requires a PM
    let label = format!("[{dict_name}-{first_lang}-{second_lang}]");
    eprintln!("{label:<20} done in {:.2?}", time.elapsed());
}

fn release_main(edition: Edition) {
    // Limit only this workload (as opposed to the full logic. IPA and glossaries are completely
    // fine and will never OOM).
    let pool = ThreadPoolBuilder::new()
        // 2 seems fine with a MemoryMax of 20GB (works on my machine TM)
        // 8 is fine for testing with only English/German/French editions
        .num_threads(2)
        .build()
        .expect("Failed to build local thread pool");

    pool.install(|| {
        Lang::all().par_iter().for_each(|source| {
            let start = Instant::now();

            let langs = match (edition, source) {
                (Edition::Simple, Lang::Simple) => MainLangs {
                    source: *source,
                    target: edition,
                },
                (Edition::Simple, _) | (_, Lang::Simple) => return,
                _ => MainLangs {
                    source: *source,
                    target: edition,
                },
            };

            let args = MainArgs {
                langs,
                dict_name: DictName::default(),
                options: Options {
                    quiet: true,
                    root_dir: "data".into(),
                    ..Default::default()
                },
            };

            match make_dict(DMain, args) {
                Ok(()) => pp("main", *source, edition.into(), start),
                Err(err) => tracing::error!("[main-{source}-{edition}] ERROR: {err:?}"),
            }
        });
    });
}

fn release_ipa(edition: Edition) {
    Lang::all().par_iter().for_each(|source| {
        let start = Instant::now();

        let langs = match (edition, source) {
            (Edition::Simple, Lang::Simple) => MainLangs {
                source: *source,
                target: edition,
            },
            (Edition::Simple, _) | (_, Lang::Simple) => return,
            _ => MainLangs {
                source: *source,
                target: edition,
            },
        };

        let args = IpaArgs {
            langs,
            dict_name: DictName::default(),
            options: Options {
                quiet: true,
                root_dir: "data".into(),
                ..Default::default()
            },
        };

        match make_dict(DIpa, args) {
            Ok(()) => pp("ipa", *source, edition.into(), start),
            Err(err) => tracing::error!("[ipa-{source}-{edition}] ERROR: {err:?}"),
        }
    });
}

fn release_ipa_merged(edition: Edition) {
    let start = Instant::now();

    let langs = match edition {
        Edition::Simple => return,
        _ => IpaMergedLangs {
            target: edition.into(),
        },
    };

    let args = IpaMergedArgs {
        langs,
        dict_name: DictName::default(),
        options: Options {
            quiet: true,
            root_dir: "data".into(),
            ..Default::default()
        },
    };

    match make_dict(DIpaMerged, args) {
        // Lang::Sq is a filler, doesn't exist
        Ok(()) => pp("gloss", Lang::Sq, edition.into(), start),
        Err(err) => tracing::error!("[ipa-merged--{edition}] ERROR: {err:?}"),
    }
}

fn release_glossary(edition: Edition) {
    Lang::all().par_iter().for_each(|target| {
        let start = Instant::now();

        let langs = match (edition, target) {
            (Edition::Simple, _) | (_, Lang::Simple) => return,
            _ if Lang::from(edition) == *target => return,
            _ => GlossaryLangs {
                source: edition,
                target: *target,
            },
        };

        let args = GlossaryArgs {
            langs,
            dict_name: DictName::default(),
            options: Options {
                quiet: true,
                root_dir: "data".into(),
                ..Default::default()
            },
        };

        match make_dict(DGlossary, args) {
            // Order may be wrong
            Ok(()) => pp("gloss", *target, edition.into(), start),
            Err(err) => tracing::error!("[gloss-{target}-{edition}] ERROR: {err:?}"),
        }
    });
}

#[allow(unused)]
fn release_glossary_extended(source: Lang) {
    Lang::all().par_iter().for_each(|target| {
        let start = Instant::now();

        let langs = match (source, target) {
            (Lang::Simple, _) | (_, Lang::Simple) => return,
            _ if source == *target => return,
            _ => GlossaryExtendedLangs {
                edition: EditionSpec::All,
                source,
                target: *target,
            },
        };

        let args = GlossaryExtendedArgs {
            langs,
            dict_name: DictName::default(),
            options: Options {
                quiet: true,
                root_dir: "data".into(),
                ..Default::default()
            },
        };

        match make_dict(DGlossaryExtended, args) {
            Ok(()) => pp("gloss", source, *target, start),
            Err(err) => tracing::error!("[gloss-all-{source}-{target}] ERROR: {err:?}"),
        }
    });
}

pub struct WiktextractDb {
    pub conn: Connection,
}

impl WiktextractDb {
    // TODO: hardcoded at the moment, should require a PathManager
    fn db_path_for(edition: Edition) -> String {
        format!("data/db/wiktextract_{edition}.db")
    }

    pub fn open(edition: Edition) -> Result<Self> {
        let db_path = Self::db_path_for(edition);
        let conn = Connection::open(&db_path)?;
        Ok(Self { conn })
    }

    pub fn create(edition: Edition, path_jsonl: PathBuf) -> Result<Self> {
        let db_path = Self::db_path_for(edition);

        if let Some(parent) = Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;

        conn.execute_batch(
            r"
            CREATE TABLE IF NOT EXISTS wiktextract (
                id INTEGER PRIMARY KEY,
                lang TEXT NOT NULL,
                entry BLOB NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_wiktextract_lang
            ON wiktextract(lang);
            ",
        )?;

        let mut db = Self { conn };

        // NOTE: Not sure if we need to check that we init the db beforehand
        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM wiktextract", [], |row| row.get(0))?;

        if count == 0 {
            tracing::info!("DB empty for {edition}, importing JSONL...");
            db.import_jsonl(path_jsonl)?;
        } else {
            tracing::trace!("DB already initialized for {edition} ({count} rows)");
        }

        Ok(db)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub fn import_jsonl<P: AsRef<Path>>(&mut self, jsonl_path: P) -> Result<()> {
        let start = Instant::now();
        let file = File::open(&jsonl_path)?;
        let reader = BufReader::new(file);

        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare("INSERT INTO wiktextract (lang, entry) VALUES (?, ?)")?;

            for line in reader.lines() {
                let line = line?;
                let word_entry: WordEntry = serde_json::from_str(&line)?;
                let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&word_entry)?;

                stmt.execute(params![word_entry.lang_code, bytes.as_ref()])?;
            }
        }
        tx.commit()?;
        tracing::debug!(
            "Making db took {:.3} ms",
            start.elapsed().as_secs_f64() * 1000.0
        );

        Ok(())
    }

    pub fn blob_to_word_entry(blob: &[u8]) -> Result<WordEntry> {
        let archived: &Archived<WordEntry> =
            rkyv::access::<Archived<WordEntry>, rkyv::rancor::Error>(blob).unwrap();
        let word_entry: WordEntry =
            rkyv::deserialize::<WordEntry, rkyv::rancor::Error>(archived).unwrap();
        Ok(word_entry)
    }
}

fn make_dict<D: Dictionary + EditionFrom>(dict: D, raw_args: D::A) -> Result<()> {
    let pm: &PathManager = &raw_args.try_into()?;
    let (_, source_pm, target_pm) = pm.langs();
    let opts = &pm.opts;
    pm.setup_dirs()?;

    tracing::trace!("{pm:#?}");

    let mut irs = D::I::default();

    for pair in iter_datasets(pm) {
        let (edition, _path_jsonl) = pair?;

        let db = WiktextractDb::open(edition)?;
        let langs = Langs {
            edition,
            source: source_pm,
            target: target_pm,
        };

        let other = match dict.edition_is() {
            EditionIs::Target => source_pm,
            EditionIs::Source => target_pm,
            EditionIs::All => target_pm,
        };
        tracing::trace!("Opened db for {edition} edition, selecting lang {other}...");

        let mut stmt = db
            .conn
            .prepare("SELECT entry FROM wiktextract WHERE lang = ?")?;
        let mut rows = stmt.query([other.as_ref()])?;

        while let Some(row) = rows.next()? {
            let blob: &[u8] = row.get_ref(0)?.as_blob()?;
            let mut entry = WiktextractDb::blob_to_word_entry(blob)?;

            dict.preprocess(langs, &mut entry, opts, &mut irs);
            dict.process(langs, &entry, &mut irs);
        }
    }

    if !opts.quiet {
        dict.found_ir_message(&irs);
    }

    if irs.is_empty() {
        return Ok(());
    }

    dict.postprocess(&mut irs);

    if opts.save_temps && dict.write_ir() {
        irs.write(pm)?;
    }

    if !opts.skip_yomitan {
        let labelled_entries = dict.to_yomitan(pm.langs, irs);
        write_yomitan(source_pm, target_pm, opts, &pm, labelled_entries)?;
    }

    Ok(())
}

enum EditionIs {
    Target,
    Source,
    All,
}

trait EditionFrom {
    fn edition_is(&self) -> EditionIs;
}

impl EditionFrom for DMain {
    fn edition_is(&self) -> EditionIs {
        EditionIs::Target
    }
}

impl EditionFrom for DIpa {
    fn edition_is(&self) -> EditionIs {
        EditionIs::Target
    }
}

impl EditionFrom for DGlossary {
    fn edition_is(&self) -> EditionIs {
        EditionIs::Source
    }
}

impl EditionFrom for DIpaMerged {
    fn edition_is(&self) -> EditionIs {
        // Does not really matter since for this dict source == target
        EditionIs::Source
    }
}

impl EditionFrom for DGlossaryExtended {
    fn edition_is(&self) -> EditionIs {
        EditionIs::All
    }
}
