//! HTML renderers for the YomitanEntry type.

use crate::models::yomitan::*;
use maud::{Markup, html};

pub trait Renderer {
    fn render_entry(entry: &YomitanEntry) -> Markup {
        match entry {
            YomitanEntry::TermInfo(t) => Self::render_term_info(t),
            YomitanEntry::TermInfoForm(t) => Self::render_term_info_form(t),
            YomitanEntry::TermMeta(t) => Self::render_term_meta(t),
        }
    }

    // 1. Reading
    // A simple reading can be rendered with
    // div class="entry" {
    //    h2 { (self.term) }
    //    div class="reading" { (self.reading) }
    //    ...
    // but yomitan renders them as ruby.
    // See https://github.com/yomidevs/yomitan/blob/master/ext/js/display/display-generator.js#L1050
    //
    // WARN: rendering as ruby may not be supported in some readers.
    // See https://github.com/koreader/koreader/issues/15259#issuecomment-4231135351
    //
    //
    // 2. Multiple definitions
    // This part works in yomitan because they group multiple definitions...
    // but other formats may not.
    //
    // It is unclear to me if we want to merge them or not, prior to this rendering.
    //
    // Note that, for the main dictionary, we always have exactly one definition in lemmas.
    // This is NOT true for forms in the main dictionary, nor for the glossary dictionary.
    fn render_term_info(entry: &TermInfo) -> Markup {
        html! {
            div class="entry" {
                div class="headword" {
                    ruby {
                        (entry.term)
                        @if !entry.reading.is_empty() {
                            rt { (entry.reading) }
                        }
                    }
                }
                div class="definition-tag-list tag-list" {
                    @for tag in &entry.definition_tags {
                        span
                            class="tag"
                            title=(tag.long_tag)
                            data-details=(tag.long_tag)
                            data-category=(tag.category)
                        {
                            span class="tag-label" {
                                span class="tag-label-content" { (tag.short_tag) }
                            }
                        }
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

    fn render_term_info_form(entry: &TermInfoForm) -> Markup {
        html! {
            div class="entry form" {
                h2 { (&entry.term) }
                div class="reading" { (&entry.reading) }
                ul {
                    @for def in &entry.definitions {
                        li { (Self::render_detailed_definition(def)) }
                    }
                }
            }
        }
    }

    fn render_term_meta(entry: &TermMeta) -> Markup {
        let TermMeta::TermPhoneticTranscription(tm) = entry;
        html! {
            div class="entry form" {
                h2 { (tm.term) }
                h3 { (Self::render_phonetic_transcription(&tm.transcription)) }
            }
        }
    }

    fn render_phonetic_transcription(pt: &PhoneticTranscription) -> Markup {
        html! {
            b { (pt.reading) }
            ul {
                @for tr in &pt.transcriptions {
                    li { (tr.ipa) (tr.tags.join("|")) }
                }
            }
        }
    }

    fn render_detailed_definition(def: &DetailedDefinition) -> Markup {
        match def {
            DetailedDefinition::Text(s) => html! { (s) },
            DetailedDefinition::StructuredContent(s) => Self::render_structured_content(s),
            DetailedDefinition::Inflection((label, forms)) => html! {
                b { (label) } ": " (forms.join(", "))
            },
        }
    }

    fn render_structured_content(sc: &StructuredContent) -> Markup {
        Self::render_node(&sc.content)
    }

    fn render_node(node: &Node) -> Markup {
        match node {
            Node::Text(t) => html! { (t) },
            Node::Array(nodes) => html! {
                @for n in nodes { (Self::render_node(n)) }
            },
            Node::Generic(g) => Self::render_generic_node(g),
            Node::Backlink(b) => Self::render_backlink(b),
        }
    }

    fn render_generic_node(node: &GenericNode) -> Markup {
        let content = Self::render_node(&node.content);

        let data = node.data.as_ref();
        let content_attr = data
            .and_then(|d| d.0.get(&NodeDataKey::Content))
            .map(|s| s.as_str());
        let category_attr = data
            .and_then(|d| d.0.get(&NodeDataKey::Category))
            .map(|s| s.as_str());
        let class = match content_attr {
            Some("tag") => Some("gloss-sc-span"),
            _ => None,
        };

        // https://github.com/lambda-fairy/maud/issues/240
        // The attr=[value] syntax skips the attribute if the value is None
        match node.tag {
            NTag::Span => html! {
                span
                    class=[class]
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                { (content) }
            },
            NTag::Div => html! {
                div
                    class=[class]
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                { (content) }
            },
            NTag::Ol => html! {
                ol
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                { (content) }
            },
            NTag::Ul => html! {
                ul
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                { (content) }
            },
            NTag::Li => html! {
                li
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                { (content) }
            },
            NTag::Details => html! {
                details
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                { (content) }
            },
            NTag::Summary => html! {
                summary
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                { (content) }
            },
        }
    }

    fn render_backlink(b: &BacklinkContent) -> Markup {
        let label = match b.content {
            BacklinkContentKind::Wiktionary => "Wiktionary",
            BacklinkContentKind::Kaikki => "Kaikki",
        };
        html! {
            a href=(b.href) data-sc-content="backlink" { (label) }
        }
    }
}
