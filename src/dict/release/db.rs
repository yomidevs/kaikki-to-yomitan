use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::Result;
use rkyv::Archived;
use rusqlite::{Connection, params};

use crate::{lang::Edition, models::kaikki::WordEntry};

pub struct WiktextractDb {
    pub conn: Connection,
}

impl WiktextractDb {
    /// Path to the folder that contains the databases for all editions.
    fn db_folder<P>(root_dir: P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        root_dir.as_ref().join("db")
    }

    /// Path for the database of this edition.
    fn db_path_for<P>(root_dir: P, edition: Edition) -> PathBuf
    where
        P: AsRef<Path>,
    {
        Self::db_folder(root_dir).join(format!("wiktextract_{edition}.db"))
    }

    pub fn open<P>(root_dir: P, edition: Edition) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let db_path = Self::db_path_for(root_dir, edition);
        let conn = Connection::open(&db_path)?;
        Ok(Self { conn })
    }

    pub fn create<P>(root_dir: P, edition: Edition, path_jsonl: PathBuf) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let _ = std::fs::create_dir(Self::db_folder(&root_dir));

        let db_path = Self::db_path_for(&root_dir, edition);
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
