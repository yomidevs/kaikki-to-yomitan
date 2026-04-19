//! Rendering for the Stardict format that mostly targets KOReader.
//!
//! Note that, as of now, ruby is not supported, nor are links.

use maud::{Markup, html};

use crate::{
    dict::writer::renderer::Renderer,
    models::yomitan::{BacklinkContent, GenericNode, NodeDataKey, TagInfo},
};

// Rendering that mostly targets KOReader
pub struct StardictRenderer;

impl Renderer for StardictRenderer {
    fn skip_render_generic_node(node: &GenericNode) -> bool {
        let data = node.data.as_ref();
        let content_attr = data
            .and_then(|d| d.0.get(&NodeDataKey::Content))
            .map(|s| s.as_str());
        matches!(
            content_attr,
            Some("preamble" | "summary-entry" | "example-sentence")
        )
    }

    // Render headword without ruby
    fn render_headword(term: &str, reading: &str) -> Markup {
        html!(
            (term)
            @if !reading.is_empty() {
                span class="reading" { " [" (reading) "]" }
            }
        )
    }

    // b doesnt work :/
    fn render_definition_tag(tag: &TagInfo) -> Markup {
        html! {
            h3 { (tag.long_tag) }
        }
    }

    fn render_backlink(_: &BacklinkContent) -> Markup {
        html! {}
    }
}
