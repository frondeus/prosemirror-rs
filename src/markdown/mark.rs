use crate::model::Mark;
use serde::{Deserialize, Serialize};

use super::{FootnoteAttrs, LinkAttrs, MarkdownMarkType, MD};

/// The marks that can be on some span
#[derive(Debug, Hash, Eq, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MarkdownMark {
    /// bold
    Strong,
    /// italics
    Em,
    /// monospace
    Code,
    /// hyper-linked
    Link {
        /// The attributes
        attrs: LinkAttrs,
    },
    /// [^1] style footnotes
    Footnote {
        /// The attributes
        attrs: FootnoteAttrs,
    },
    /// ~strikethrough~
    Strikethrough,
}

impl Mark<MD> for MarkdownMark {
    fn r#type(&self) -> MarkdownMarkType {
        match self {
            Self::Strong => MarkdownMarkType::Strong,
            Self::Em => MarkdownMarkType::Em,
            Self::Code => MarkdownMarkType::Code,
            Self::Link { .. } => MarkdownMarkType::Link,
            Self::Footnote { .. } => MarkdownMarkType::Footnote,
            Self::Strikethrough => MarkdownMarkType::Strikethrough,
        }
    }
}
impl MarkdownMark {
    /// Is this mark represented by Tag<'a> and TagEnd in pulldown cmark
    pub fn is_represented_by_tag(&self) -> bool {
        match self {
            MarkdownMark::Code => false,
            MarkdownMark::Footnote { .. } => false,

            MarkdownMark::Strong
            | MarkdownMark::Em
            | MarkdownMark::Link { .. }
            | MarkdownMark::Strikethrough => true,
        }
    }
}
