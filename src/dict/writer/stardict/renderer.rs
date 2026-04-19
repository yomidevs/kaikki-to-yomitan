//! Rendering for the Stardict format that mostly targets KOReader.
//!
//! Note that, as of now, ruby is not supported, nor are links.

use maud::{Markup, html};

use crate::{
    dict::writer::renderer::Renderer,
    models::yomitan::{BacklinkContent, TermInfo},
};

// Rendering that mostly targets KOReader
pub(crate) struct StardictRenderer;

impl Renderer for StardictRenderer {
    // Render term info without ruby
    fn render_term_info(entry: &TermInfo) -> Markup {
        html! {
            div class="entry" {
                div class="headword" {
                    (entry.term)
                    @if !entry.reading.is_empty() {
                        span class="reading" { " [" (entry.reading) "]" }
                    }
                }
                @if entry.definitions.len() == 1 {
                    (Self::render_detailed_definition(&entry.definitions[0]))
                } @else {
                    ol class="definition-list" {
                        @for def in &entry.definitions {
                            li {
                                (Self::render_detailed_definition(def))
                            }
                        }
                    }
                }
            }
        }
    }

    fn render_backlink(_: &BacklinkContent) -> Markup {
        html! {}
    }
}
