use maud::{Markup, html};

use crate::{
    models::yomitan::{
        BacklinkContent, BacklinkContentKind, DetailedDefinition, GenericNode, NTag, Node,
        NodeDataKey, StructuredContent, TagInfo, TermInfo, TermInfoForm, YomitanEntry,
    },
    tags::find_short_tag_in_bank,
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
            YomitanEntry::TermMeta(_) => unimplemented!(),
        }
    }
}

impl ToHtml for TermInfo {
    fn to_html(&self) -> Markup {
        let term = &self.0;
        let reading = &self.1;
        let tags = &self.2;
        let defs = &self.4;

        // Because we relied on yomitan, we have to re-do some work to get the info back
        // HACK: something quick for now
        let tinfos: Vec<TagInfo> = tags
            .split_whitespace()
            .map(|tag| {
                // SAFETY: we already found the tag in the back
                // (it's probably better to carry this info in the data structure,
                // then only serialize the short forms I guess!)
                find_short_tag_in_bank(tag).expect(&format!("tag {tag} was not in bank!"))
            })
            .collect();
        // TODO: we are supposed to sort them ourselves too!

        html! {
            div class="entry" {

                h2 { (term) }
                div class="reading" { (reading) }

                div class="definition-tag-list tag-list" {
                    @for tag in tinfos {
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

                ul class="gloss-list" {
                    span class="gloss-content structured-content" {
                        @for def in defs {
                            li { (def.to_html()) }
                        }
                    }
                }
            }
        }
    }
}

impl ToHtml for TermInfoForm {
    fn to_html(&self) -> Markup {
        html! {
            div class="entry form" {
                h2 { (&self.0) }
                div class="reading" { (&self.1) }

                ul {
                    @for def in &self.3 {
                        li { (def.to_html()) }
                    }
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
