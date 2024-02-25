//! # The markdown schema
//!
//! This module is derived from the `prosemirror-markdown` schema and the
//! the general JSON serialization of nodes.
mod attrs;
mod content;
pub mod helper;
mod mark;
mod node;
mod schema;

#[cfg(feature = "cmark")]
mod from_markdown;
#[cfg(feature = "cmark")]
mod print_markdown;
#[cfg(feature = "cmark")]
mod to_markdown;

pub use attrs::{
    CodeBlockAttrs, FootnoteAttrs, HTMLAttrs, HeadingAttrs, ImageAttrs, LinkAttrs,
    OrderedListAttrs, TableAttrs, TaskListMarkerAttrs,
};
pub use content::MarkdownContentMatch;
pub use mark::MarkdownMark;
pub use node::MarkdownNode;
pub use schema::{MarkdownMarkType, MarkdownNodeType, MD};

#[cfg(feature = "cmark")]
pub use from_markdown::{from_markdown, FromMarkdownError};
#[cfg(feature = "cmark")]
pub use to_markdown::{to_markdown, ToMarkdownError};
