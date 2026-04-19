use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Result;

use crate::{
    cli::Options,
    dict::writer::{html::prettify_html, renderer::Renderer},
    models::yomitan::YomitanDict,
    path::PathManager,
    utils::pretty_println_at_path,
};

mod renderer;
use renderer::MdictTextRenderer;

pub fn write_mdict_text(opts: &Options, pm: &PathManager, ydict: YomitanDict) -> Result<()> {
    let dname = pm.dict_name_expanded();
    let filepath = format!("html/test-{}.txt", dname);
    let filename = Path::new(&filepath);
    if let Some(parent) = filename.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    for entry in ydict.into_iter_flat() {
        writer.write_all(entry.term().as_bytes())?;
        writer.write_all(b"\n")?;

        let html = MdictTextRenderer::render_entry(&entry).into_string();
        // Requires a css include in each entry!
        writer.write_all(b"<link rel='stylesheet' href='styles.css' type='text/css'>")?;
        if opts.pretty {
            writer.write_all(prettify_html(&html).as_bytes())?;
        } else {
            writer.write_all(html.as_bytes())?;
        }
        writer.write_all(b"\n</>\n")?;
    }

    pretty_println_at_path("Wrote mdict-text", filename);

    Ok(())
}
