use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

use anyhow::Result;

use crate::{
    cli::Options,
    dict::writer::{html::prettify_html, renderer::Renderer},
    models::yomitan::YomitanDict,
    path::PathManager,
};

mod renderer;
use renderer::MdictTextRenderer;

pub fn write_mdict_text(opts: &Options, pm: &PathManager, ydict: YomitanDict) -> Result<PathBuf> {
    let dir_in_stage = pm.dir_in_stage("mdict");
    _ = fs::create_dir_all(&dir_in_stage);

    let dict_name = format!("{}.txt", pm.dict_name_expanded());
    let path_dict = dir_in_stage.join(dict_name);

    let file = File::create(&path_dict)?;
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

    // TODO: maybe also copy the css here?

    Ok(path_dict)
}
