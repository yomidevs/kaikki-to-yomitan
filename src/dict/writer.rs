//! Shared write behaviour.
//!
//! Structs that are dictionary dependent, like the intermediate representation or diagnostics, are
//! not included here and should be next to their dictionary for visibility.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::Result;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::cli::Options;
use crate::dict::core::LabelledYomitanEntries;
use crate::dict::index::get_index;
use crate::lang::Lang;
use crate::models::yomitan::YomitanEntry;
use crate::path::PathManager;
use crate::tags::get_tag_bank_as_tag_info;
use crate::utils::{CHECK_C, pretty_print_at_path, pretty_println_at_path};

const BANK_SIZE: usize = 25_000;

const STYLES_CSS: &[u8] = include_bytes!("../../assets/styles.css");
const STYLES_CSS_EXPERIMENTAL: &[u8] = include_bytes!("../../assets/styles_experimental.css");

enum Sink<'a> {
    Disk,
    Zip(&'a mut ZipWriter<File>, SimpleFileOptions),
}

/// Write yomitan labelled entries in banks to a sink (either disk or zip).
///
/// When zipping, also write metadata (index, css etc.).
pub fn write_yomitan(
    source: Lang,
    target: Lang,
    opts: &Options,
    pm: &PathManager,
    labelled_entries: Vec<LabelledYomitanEntries>,
) -> Result<()> {
    let mut bank_index = 0;

    // use crate::dict::heap::HeapSize;
    // let heap_size = labelled_entries.heap_size();
    // let heap_size_msg = crate::utils::human_size(heap_size as f64);
    // tracing::error!(
    //     "[{source}-{target}] YomitanEntry Vec heap size: {}",
    //     heap_size_msg
    // );

    if opts.save_temps {
        let out_dir = pm.dir_temp_dict();
        fs::create_dir_all(&out_dir)?;
        for lentry in labelled_entries {
            write_banks(
                opts.pretty,
                opts.quiet,
                &lentry.entries,
                &mut bank_index,
                lentry.label,
                &out_dir,
                Sink::Disk,
            )?;
        }

        if !opts.quiet {
            pretty_println_at_path(&format!("{CHECK_C} Wrote temp data"), &out_dir);
        }
        return Ok(());
    }

    let writer_path = pm.path_dict();
    let writer_file = File::create(&writer_path)?;
    let mut zip = ZipWriter::new(writer_file);
    let zip_opts =
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Zip index.json
    let index_string = get_index(pm.dict_ty, &pm.dict_name_expanded(), source, target);
    zip.start_file("index.json", zip_opts)?;
    zip.write_all(index_string.as_bytes())?;

    // Zip a copy of styles.css
    zip.start_file("styles.css", zip_opts)?;
    if opts.experimental {
        zip.write_all(STYLES_CSS_EXPERIMENTAL)?;
    } else {
        zip.write_all(STYLES_CSS)?;
    }

    // Zip a (potentially localized) version without aliases of tag_bank_term.json
    let tag_bank = get_tag_bank_as_tag_info(target);
    let tag_bank_bytes = serde_json::to_vec_pretty(&tag_bank)?;
    zip.start_file("tag_bank_1.json", zip_opts)?; // it needs to end in _1
    zip.write_all(&tag_bank_bytes)?;

    for lentry in labelled_entries {
        write_banks(
            opts.pretty,
            opts.quiet,
            &lentry.entries,
            &mut bank_index,
            lentry.label,
            &writer_path,
            Sink::Zip(&mut zip, zip_opts),
        )?;
    }

    zip.finish()?;

    pretty_println_at_path(&format!("{CHECK_C} Wrote yomitan dict"), &writer_path);

    Ok(())
}

/// Writes `yomitan_entries` in batches to `out_sink` (either disk or a zip).
#[tracing::instrument(skip_all, level = "DEBUG")]
fn write_banks(
    pretty: bool,
    quiet: bool,
    yomitan_entries: &[YomitanEntry],
    bank_index: &mut usize,
    label: &str,
    out_dir: &Path,
    mut sink: Sink,
) -> Result<()> {
    // NOTE: this assumes that once a type is passed, all the remaining entries are of same type
    let bank_name_prefix = match yomitan_entries.first() {
        Some(first) => first.file_prefix(),
        None => return Ok(()),
    };

    let total_bank_num = yomitan_entries.len().div_ceil(BANK_SIZE);

    for (bank_num, bank) in yomitan_entries.chunks(BANK_SIZE).enumerate() {
        *bank_index += 1;

        let json_bytes = if pretty {
            serde_json::to_vec_pretty(&bank)?
        } else {
            serde_json::to_vec(&bank)?
        };

        let bank_name = format!("{bank_name_prefix}_{bank_index}.json");
        let file_path = out_dir.join(&bank_name);

        match sink {
            Sink::Disk => {
                let mut file = File::create(&file_path)?;
                file.write_all(&json_bytes)?;
            }
            Sink::Zip(ref mut zip, zip_options) => {
                zip.start_file(&bank_name, zip_options)?;
                zip.write_all(&json_bytes)?;
            }
        }

        if !quiet {
            if bank_num > 0 {
                print!("\r\x1b[K");
            }
            pretty_print_at_path(
                &format!(
                    "Wrote yomitan {label} bank {}/{total_bank_num} ({} entries)",
                    bank_num + 1,
                    bank.len()
                ),
                &file_path,
            );
            std::io::stdout().flush()?;
        }
    }

    if !quiet {
        println!();
    }

    Ok(())
}
