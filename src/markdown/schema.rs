use crate::markdown::{MarkdownContentMatch, MarkdownMark, MarkdownNode};
use crate::model::{ContentMatch, Fragment, MarkSet, MarkType, Node, NodeType, Schema};

/// The markdown schema type
pub struct MD;

impl Schema for MD {
    type Node = MarkdownNode;
    type Mark = MarkdownMark;
    type MarkType = MarkdownMarkType;
    type NodeType = MarkdownNodeType;
    type ContentMatch = MarkdownContentMatch;
}

/// The type of a markdown mark.
#[derive(Debug, Hash, Eq, Copy, Clone, PartialEq, PartialOrd, Ord)]
pub enum MarkdownMarkType {
    /// bold
    Strong,
    /// italics
    Em,
    /// monospace
    Code,
    /// hyper-linked
    Link,
    /// [^1] style footnotes
    Footnote,
    /// <foo>, </foo> or <foo /> tags
    HtmlTag,
    /// ~strikethrough~
    Strikethrough,
}

impl MarkType for MarkdownMarkType {}

/// The node-spec type for the markdown schema
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MarkdownNodeType {
    /// The document root
    Doc,
    /// A heading, e.g. `<h1>`
    Heading,
    /// A code block
    CodeBlock,
    /// A text node
    Text,
    /// A blockquote
    Blockquote,
    /// A paragraph
    Paragraph,
    /// A bullet list
    BulletList,
    /// An ordered list
    OrderedList,
    /// A list item
    ListItem,
    /// A horizontal line `<hr>`
    HorizontalRule,
    /// A hard break `<br>`
    HardBreak,
    /// An image `<img>`
    Image,
    /// Footnote definition
    FootnoteDefinition,
    /// Task list marker [x] or [ ]
    TaskListMarker,
    /// YAML style metadata
    Metadata,
    /// A github style table
    Table,
    /// A table head
    TableHead,
    /// A table row
    TableRow,
    /// A table cell
    TableCell,
    /// HTML node
    /// - bool is inline
    HTML(bool),
}

impl MarkdownNodeType {
    fn _allow_marks(self) -> bool {
        match self {
            Self::Doc
            | Self::Blockquote
            | Self::BulletList
            | Self::OrderedList
            | Self::ListItem => false, // block && !textblock

            Self::CodeBlock => false, // marks = ""

            Self::Heading | Self::Paragraph => true, // textblock
            Self::FootnoteDefinition => true,
            Self::Metadata => false,
            Self::TableCell | Self::TableHead | Self::TableRow | Self::Table => true,

            Self::Text
            | Self::TaskListMarker
            | Self::HorizontalRule
            | Self::HardBreak
            | Self::Image => true, // inline

            Self::HTML(_) => true,
        }
    }
}

impl NodeType<MD> for MarkdownNodeType {
    fn allow_marks(self, _marks: &MarkSet<MD>) -> bool {
        self._allow_marks()
    }

    fn allows_mark_type(self, _mark_type: MarkdownMarkType) -> bool {
        self._allow_marks()
    }

    fn is_block(self) -> bool {
        match self {
            MarkdownNodeType::Doc => true,
            MarkdownNodeType::Heading => false,
            MarkdownNodeType::CodeBlock => true,
            MarkdownNodeType::Text => false,
            MarkdownNodeType::Blockquote => true,
            MarkdownNodeType::Paragraph => true,
            MarkdownNodeType::BulletList => true,
            MarkdownNodeType::OrderedList => true,
            MarkdownNodeType::ListItem => false,
            MarkdownNodeType::HorizontalRule => true,
            MarkdownNodeType::HardBreak => false,
            MarkdownNodeType::Image => false,
            MarkdownNodeType::FootnoteDefinition => true,
            MarkdownNodeType::TaskListMarker => false,
            MarkdownNodeType::Metadata => true,
            MarkdownNodeType::Table => true,
            MarkdownNodeType::TableHead => false,
            MarkdownNodeType::TableRow => false,
            MarkdownNodeType::TableCell => false,
            MarkdownNodeType::HTML(is_inline) => !is_inline,
        }
    }

    fn content_match(self) -> MarkdownContentMatch {
        match self {
            Self::Doc => MarkdownContentMatch::Star,
            Self::Heading => MarkdownContentMatch::OrTextImageStar,
            Self::CodeBlock => MarkdownContentMatch::TextStar,
            Self::Text => MarkdownContentMatch::Empty,
            Self::Blockquote => MarkdownContentMatch::BlockPlus,
            Self::Paragraph => MarkdownContentMatch::InlineStar,
            Self::BulletList => MarkdownContentMatch::ListItemPlus,
            Self::OrderedList => MarkdownContentMatch::ListItemPlus,
            Self::ListItem => MarkdownContentMatch::ParagraphBlockStar,
            Self::HorizontalRule => MarkdownContentMatch::Empty,
            Self::HardBreak => MarkdownContentMatch::Empty,
            Self::Image => MarkdownContentMatch::Empty,
            Self::FootnoteDefinition => MarkdownContentMatch::InlineStar,
            Self::TaskListMarker => MarkdownContentMatch::Empty,
            Self::Metadata => MarkdownContentMatch::TextStar,
            Self::Table => MarkdownContentMatch::BlockPlus,
            Self::TableHead => MarkdownContentMatch::BlockPlus,
            Self::TableRow => MarkdownContentMatch::BlockPlus,
            Self::TableCell => MarkdownContentMatch::InlineStar,
            Self::HTML(true) => MarkdownContentMatch::InlineStar,
            Self::HTML(false) => MarkdownContentMatch::BlockStar,
        }
    }

    fn compatible_content(self, other: Self) -> bool {
        self == other || self.content_match().compatible(other.content_match())
    }

    /// Returns true if the given fragment is valid content for this node type with the given
    /// attributes.
    fn valid_content(self, fragment: &Fragment<MD>) -> bool {
        // eprintln!("{self:?} Is valid content? {fragment:?}");
        let result = self.content_match().match_fragment(fragment);

        if let Some(m) = result {
            if m.valid_end() {
                for child in fragment.children() {
                    if child.marks().filter(|m| !self.allow_marks(m)).is_some() {
                        // eprintln!("{self:?} FALSE - mark");
                        return false;
                    }
                }

                // eprintln!("{self:?} TRUE");
                return true;
            }
            // eprintln!("{self:?} not valid end?");
        }

        // eprintln!("{self:?} FALSE - content match");
        false
    }
}
