use crate::de;
use serde::{Deserialize, Serialize};

/// Attributes for a heading (i.e. `<h1>`, `<h2>`, ...)
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct HeadingAttrs {
    /// The level of the heading (i.e. `1` for `<h1>`)
    pub level: u8,
}

/// Attributes for a code block
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CodeBlockAttrs {
    /// ???
    pub params: String,
}

/// Attributes for a bullet list
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct BulletListAttrs {
    /// ???
    pub tight: bool,
}

/// Attributes for an ordered list
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct OrderedListAttrs {
    /// Initial value
    pub order: usize,
    /// ???
    pub tight: bool,
}

/// Attributes for an image
/// Alt text is stored as a content of type `Text`
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct ImageAttrs {
    /// Source URL
    pub src: String,
    /// Title (Tooltip)
    #[serde(default, deserialize_with = "de::deserialize_or_default")]
    pub title: String,
}

/// The attributes for a hyperlink
#[derive(Debug, Hash, Eq, Clone, PartialEq, Deserialize, Serialize)]
pub struct LinkAttrs {
    /// The URL the link points to
    pub href: String,
    /// The title of the link
    #[serde(default, deserialize_with = "de::deserialize_or_default")]
    pub title: String,
}

/// The attributes for a footnote
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct FootnoteAttrs {
    /// The label of the footnote
    pub label: String,
}

/// The attributes for a task list marker [x] or [ ]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TaskListMarkerAttrs {
    /// Whether the task is checked [x] or not [ ]
    pub checked: bool,
}

/// The attributes for a table
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct TableAttrs {
    /// The alignment of the columns
    pub alignment: Vec<Alignment>,
}

/// 1:1 copy of the `Alignment` from `pullown-cmark`. We only added `Eq` implementation.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum Alignment {
    None,
    Left,
    Center,
    Right,
}
