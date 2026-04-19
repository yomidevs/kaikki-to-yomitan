use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::{
    cli::Options, dict::writer::renderer::Renderer, models::yomitan::YomitanDict, path::PathManager,
};

use anyhow::Result;

mod renderer;
use renderer::HtmlRenderer;

const STYLES_CSS_HTML: &[u8] = include_bytes!("../../../../assets/styles_html.css");

#[allow(unused)]
pub fn write_html(opts: &Options, pm: &PathManager, ydict: YomitanDict) -> Result<PathBuf> {
    let dir_in_stage = pm.dir_in_stage("html");
    _ = fs::create_dir_all(&dir_in_stage);

    let dict_name = format!("{}.html", pm.dict_name_expanded());
    let path_dict = dir_in_stage.join(&dict_name);
    let path_css = dir_in_stage.join("styles.css");

    let file = File::create(&path_dict)?;
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

    for entry in ydict.into_iter_flat() {
        let html = HtmlRenderer::render_entry(&entry).into_string();
        // Ignore pretty since it can mess the result
        writer.write_all(html.as_bytes())?;
    }

    writer.write_all(br"</body></html>")?;

    // Copy assets/styles_html.css into dir_in_stage/styles.css
    fs::write(&path_css, STYLES_CSS_HTML)?;

    Ok(dir_in_stage)
}

pub fn write_test_html(opts: &Options, pm: &PathManager, ydict: YomitanDict) -> Result<PathBuf> {
    let dir_in_stage = pm.dir_in_stage("html");
    _ = fs::create_dir_all(&dir_in_stage);

    let dict_name = format!("{}.html", pm.dict_name_expanded());
    let path_dict = dir_in_stage.join(dict_name);

    let file = File::create(&path_dict)?;
    let mut writer = BufWriter::new(file);

    for entry in ydict.into_iter_flat() {
        let html = HtmlRenderer::render_entry(&entry).into_string();
        if opts.pretty {
            writer.write_all(prettify_html(&html).as_bytes())?;
        } else {
            writer.write_all(html.as_bytes())?;
        }
    }

    Ok(path_dict)
}

pub fn prettify_html(html: &str) -> String {
    let mut result = String::new();
    let mut indent: usize = 0;
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => {
                if !in_tag {
                    result.push('\n');
                    // WARN: whitespace messes up browser rendering when coupled with yomitan
                    // css (white-space: pre-line).
                    result.push_str(&"  ".repeat(indent)); // main offender
                }
                in_tag = true;
                result.push(ch);
            }
            '>' => {
                result.push(ch);
                in_tag = false;

                // Adjust indent based on tag type
                let last_tag = result.split('<').next_back().unwrap_or("");
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
