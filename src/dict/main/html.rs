use maud::{Markup, html};

use crate::models::yomitan::{
    BacklinkContent, BacklinkContentKind, DetailedDefinition, GenericNode, NTag, Node, NodeDataKey,
    PhoneticTranscription, StructuredContent, TermInfo, TermInfoForm, TermMeta, YomitanEntry,
};

// I think there is a trait for this in maud???
trait ToHtml {
    fn to_html(&self) -> Markup;
}

pub fn render_entry(entry: &YomitanEntry) -> Markup {
    entry.to_html()
}

impl ToHtml for YomitanEntry {
    fn to_html(&self) -> Markup {
        match self {
            YomitanEntry::TermInfo(t) => t.to_html(),
            YomitanEntry::TermInfoForm(t) => t.to_html(),
            YomitanEntry::TermMeta(t) => t.to_html(),
        }
    }
}

impl ToHtml for TermInfo {
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
    // Note that we always have exactly one definition. That is how we comply with
    // the yomitan schema.
    fn to_html(&self) -> Markup {
        debug_assert!(self.definitions.len() == 1);
        let def = &self.definitions[0];

        html! {
            div class="entry" {
                div class="headword" {
                    ruby {
                        (self.term)
                        @if !self.reading.is_empty() {
                            rt { (self.reading) }
                        }
                    }
                }

                div class="definition-tag-list tag-list" {
                    @for tag in &self.definition_tags {
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

                (def.to_html())
            }
        }
    }
}

impl ToHtml for TermInfoForm {
    fn to_html(&self) -> Markup {
        html! {
            div class="entry form" {
                h2 { (&self.term) }
                div class="reading" { (&self.reading) }

                ul {
                    @for def in &self.definitions {
                        li { (def.to_html()) }
                    }
                }
            }
        }
    }
}

// Just a prototype, very rough
impl ToHtml for TermMeta {
    fn to_html(&self) -> Markup {
        let TermMeta::TermPhoneticTranscription(tm) = self;
        html! {
            div class="entry form" {
                h2 { (tm.term) }
                h3 { (tm.transcription.to_html()) }
            }
        }
    }
}

impl ToHtml for PhoneticTranscription {
    fn to_html(&self) -> Markup {
        html! {
            b { (self.reading) }
            ul {
                @for tr in &self.transcriptions {
                    li { (tr.ipa) (tr.tags.join("|")) }
                }
            }
        }
    }
}

impl ToHtml for DetailedDefinition {
    fn to_html(&self) -> Markup {
        match self {
            DetailedDefinition::Text(s) => html! { (s) },
            DetailedDefinition::StructuredContent(s) => s.to_html(),
            DetailedDefinition::Inflection((label, forms)) => {
                html! {
                    b { (label) } ": " (forms.join(", "))
                }
            }
        }
    }
}

impl ToHtml for StructuredContent {
    fn to_html(&self) -> Markup {
        self.content.to_html()
    }
}

impl ToHtml for Node {
    fn to_html(&self) -> Markup {
        match self {
            Node::Text(t) => html! { (t) },
            Node::Array(nodes) => html! {
                @for n in nodes {
                    (n.to_html())
                }
            },
            Node::Generic(g) => g.to_html(),
            Node::Backlink(b) => b.to_html(),
        }
    }
}

impl ToHtml for GenericNode {
    fn to_html(&self) -> Markup {
        let content = self.content.to_html();

        // Node data is map<String, String>
        // "data": {
        //   "content": "tag",
        //   "category": "partOfSpeech"
        // },
        // that we want to add to the tags metadata
        // like <span data-sc=content=tag

        let data = self.data.as_ref();

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

        match self.tag {
            NTag::Span => html! {
                span
                    class=[class]
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                {
                    (content)
                }
            },

            NTag::Div => html! {
                div
                    class=[class]
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                {
                    (content)
                }
            },

            NTag::Ol => html! {
                ol
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                {
                    (content)
                }
            },

            NTag::Ul => html! {
                ul
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                {
                    (content)
                }
            },

            NTag::Li => html! {
                li
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                {
                    (content)
                }
            },

            NTag::Details => html! {
                details
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                {
                    (content)
                }
            },

            NTag::Summary => html! {
                summary
                    data-sc-content=[content_attr]
                    data-sc-category=[category_attr]
                {
                    (content)
                }
            },
        }
    }
}

impl ToHtml for BacklinkContent {
    fn to_html(&self) -> Markup {
        let label = match self.content {
            BacklinkContentKind::Wiktionary => "Wiktionary",
            BacklinkContentKind::Kaikki => "Kaikki",
        };

        html! {
            a href=(self.href) data-sc-content="backlink" { (label) }
        }
    }
}
