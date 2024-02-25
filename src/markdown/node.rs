use crate::model::{AttrNode, Block, Fragment, Leaf, MarkSet, Node, Text, TextNode};
use derivative::Derivative;
use serde::{Deserialize, Serialize};

use super::{
    BulletListAttrs, CodeBlockAttrs, FootnoteAttrs, HeadingAttrs, ImageAttrs, MarkdownNodeType,
    OrderedListAttrs, TableAttrs, TaskListMarkerAttrs, MD,
};

/// The node type for the markdown schema
#[derive(Debug, Derivative, Deserialize, Serialize, PartialEq, Eq)]
#[derivative(Clone(bound = ""))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MarkdownNode {
    /// The document root
    Doc(Block<MD>),
    /// A heading, e.g. `<h1>`
    Heading(AttrNode<MD, HeadingAttrs>),
    /// A code block
    CodeBlock(AttrNode<MD, CodeBlockAttrs>),
    /// A text node
    Text(TextNode<MD>),
    /// A blockquote
    Blockquote(Block<MD>),
    /// A paragraph
    Paragraph(Block<MD>),
    /// A bullet list
    BulletList(AttrNode<MD, BulletListAttrs>),
    /// An ordered list
    OrderedList(AttrNode<MD, OrderedListAttrs>),
    /// A list item
    ListItem(Block<MD>),
    /// A horizontal line `<hr>`
    HorizontalRule,
    /// A hard break `<br>`
    HardBreak,
    /// [ ] or [x]
    TaskListMarker(Leaf<TaskListMarkerAttrs>),
    /// An image `<img>`
    /// The alt text is a content of type `Text`.
    Image(AttrNode<MD, ImageAttrs>),
    /// A footnote definition `[^1]: ...`
    FootnoteDefinition(AttrNode<MD, FootnoteAttrs>),
    /// YAML style matadata blocks
    Metadata(Block<MD>),
    /// | markdown | table |
    /// | --- | --- |
    /// | table | table |
    Table(AttrNode<MD, TableAttrs>),
    /// Header of the table, with the names of the columns
    TableHead(Block<MD>),
    /// A row in a table, that is not a header
    TableRow(Block<MD>),
    /// A cell in a table, both header and normal cells
    TableCell(Block<MD>),
}

impl From<TextNode<MD>> for MarkdownNode {
    fn from(text_node: TextNode<MD>) -> Self {
        Self::Text(text_node)
    }
}

impl Node<MD> for MarkdownNode {
    fn text_node(&self) -> Option<&TextNode<MD>> {
        if let Self::Text(node) = self {
            Some(node)
        } else {
            None
        }
    }

    fn new_text_node(node: TextNode<MD>) -> Self {
        Self::Text(node)
    }

    fn is_block(&self) -> bool {
        match self {
            Self::Doc { .. } => true,
            Self::Paragraph { .. } => true,
            Self::Blockquote { .. } => true,
            Self::HorizontalRule => true,
            Self::Heading { .. } => true,
            Self::CodeBlock { .. } => true,
            Self::OrderedList { .. } => true,
            Self::BulletList { .. } => true,
            Self::ListItem { .. } => true,
            Self::Text { .. } => false,
            Self::Image { .. } => false,
            Self::HardBreak => false,
            Self::TaskListMarker(_) => false,
            Self::FootnoteDefinition(_) => true,
            Self::Metadata { .. } => true,
            Self::Table { .. } => true,
            Self::TableCell(_) | Self::TableHead(_) | Self::TableRow(_) => true,
        }
    }

    fn r#type(&self) -> MarkdownNodeType {
        match self {
            Self::Doc { .. } => MarkdownNodeType::Doc,
            Self::Paragraph { .. } => MarkdownNodeType::Paragraph,
            Self::Blockquote { .. } => MarkdownNodeType::Blockquote,
            Self::HorizontalRule => MarkdownNodeType::HorizontalRule,
            Self::Heading { .. } => MarkdownNodeType::Heading,
            Self::CodeBlock { .. } => MarkdownNodeType::CodeBlock,
            Self::OrderedList { .. } => MarkdownNodeType::OrderedList,
            Self::BulletList { .. } => MarkdownNodeType::BulletList,
            Self::ListItem { .. } => MarkdownNodeType::ListItem,
            Self::Text { .. } => MarkdownNodeType::Text,
            Self::Image { .. } => MarkdownNodeType::Image,
            Self::HardBreak => MarkdownNodeType::HardBreak,
            Self::FootnoteDefinition(_) => MarkdownNodeType::FootnoteDefinition,
            Self::TaskListMarker(_) => MarkdownNodeType::TaskListMarker,
            Self::Metadata { .. } => MarkdownNodeType::Metadata,
            Self::Table { .. } => MarkdownNodeType::Table,
            Self::TableHead(_) => MarkdownNodeType::TableHead,
            Self::TableRow(_) => MarkdownNodeType::TableRow,
            Self::TableCell(_) => MarkdownNodeType::TableCell,
        }
    }

    fn text<A: Into<String>>(text: A) -> Self {
        Self::Text(TextNode {
            text: Text::from(text.into()),
            marks: MarkSet::<MD>::default(),
        })
    }

    fn content(&self) -> Option<&Fragment<MD>> {
        match self {
            Self::Doc(doc) => Some(&doc.content),
            Self::Heading(AttrNode { content, .. }) => Some(content),
            Self::CodeBlock(AttrNode { content, .. }) => Some(content),
            Self::Text { .. } => None,
            Self::Blockquote(Block { content }) => Some(content),
            Self::Paragraph(Block { content }) => Some(content),
            Self::BulletList(AttrNode { content, .. }) => Some(content),
            Self::OrderedList(AttrNode { content, .. }) => Some(content),
            Self::ListItem(Block { content }) => Some(content),
            Self::HorizontalRule => None,
            Self::HardBreak => None,
            Self::Image { .. } => None,
            Self::FootnoteDefinition(AttrNode { content, .. }) => Some(content),
            Self::TaskListMarker(_) => None,
            Self::Metadata(Block { content }) => Some(content),
            Self::Table(AttrNode { content, .. }) => Some(content),
            Self::TableHead(Block { content }) => Some(content),
            Self::TableRow(Block { content }) => Some(content),
            Self::TableCell(Block { content }) => Some(content),
        }
    }

    fn marks(&self) -> Option<&MarkSet<MD>> {
        None
    }

    fn mark(&self, set: MarkSet<MD>) -> Self {
        // TODO: marks on other nodes
        if let Some(text_node) = self.text_node() {
            Self::Text(TextNode {
                marks: set,
                text: text_node.text.clone(),
            })
        } else {
            self.clone()
        }
    }

    fn copy<F>(&self, map: F) -> Self
    where
        F: FnOnce(&Fragment<MD>) -> Fragment<MD>,
    {
        match self {
            Self::Doc(block) => Self::Doc(block.copy(map)),
            Self::Heading(node) => Self::Heading(node.copy(map)),
            Self::CodeBlock(node) => Self::CodeBlock(node.copy(map)),
            Self::Text(node) => Self::Text(node.clone()),
            Self::Blockquote(block) => Self::Blockquote(block.copy(map)),
            Self::Paragraph(block) => Self::Paragraph(block.copy(map)),
            Self::BulletList(node) => Self::BulletList(node.copy(map)),
            Self::OrderedList(node) => Self::OrderedList(node.copy(map)),
            Self::ListItem(block) => Self::ListItem(block.copy(map)),
            Self::HorizontalRule => Self::HorizontalRule,
            Self::HardBreak => Self::HardBreak,
            Self::Image(img) => Self::Image(img.clone()),
            Self::FootnoteDefinition(node) => Self::FootnoteDefinition(node.copy(map)),
            Self::TaskListMarker(marker) => Self::TaskListMarker(marker.clone()),
            Self::Metadata(block) => Self::Metadata(block.copy(map)),
            Self::Table(node) => Self::Table(node.copy(map)),
            Self::TableHead(block) => Self::TableHead(block.copy(map)),
            Self::TableRow(block) => Self::TableRow(block.copy(map)),
            Self::TableCell(block) => Self::TableCell(block.copy(map)),
        }
    }
}

impl From<&str> for MarkdownNode {
    fn from(text: &str) -> Self {
        Self::text(text)
    }
}