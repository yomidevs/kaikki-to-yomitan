use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use crate::{
    cli::Options, dict::writer::renderer::Renderer, models::yomitan::YomitanDict, path::PathManager,
};

use anyhow::Result;

mod renderer;
use renderer::HtmlRenderer;

pub fn write_html(opts: &Options, pm: &PathManager, ydict: YomitanDict) -> Result<()> {
    let dname = pm.dict_name_expanded();
    let filepath = format!("html/test-{}.html", dname);
    let filename = Path::new(&filepath);
    if let Some(parent) = filename.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(filename)?;
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
        if opts.pretty {
            let pretty = prettify_html(&html);
            writer.write_all(pretty.as_bytes())?;
        } else {
            writer.write_all(html.as_bytes())?;
        }
    }
    writer.write_all(br#"</body></html>"#)?;
    crate::utils::pretty_println_at_path("Wrote file", filename);

    Ok(())
}

// This is scuffed and introduces whitespace where it shouldn't, which then
// makes the css white-space: pre-line do some insane mangling.
// Stick with this for tests, without that css line, but remember to add it
// back in the final version.
pub fn prettify_html(html: &str) -> String {
    let mut result = String::new();
    let mut indent: usize = 0;
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => {
                if !in_tag {
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent)); // main offender
                }
                in_tag = true;
                result.push(ch);
            }
            '>' => {
                result.push(ch);
                in_tag = false;

                // Adjust indent based on tag type
                let last_tag = result.split('<').last().unwrap_or("");
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
