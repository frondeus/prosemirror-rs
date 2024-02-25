use crate::markdown::{MarkdownNodeType, MD};
use crate::model::{util, ContentMatch, Fragment, Node, NodeType};
use crate::util::then_some;
use std::ops::RangeBounds;

/// The content match type for markdown
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum MarkdownContentMatch {
    /// `*`
    Star,
    /// `inline*`
    InlineStar,
    /// `block+`
    BlockPlus,
    /// `block*`
    BlockStar,
    /// `(text | image)*`
    OrTextImageStar,
    /// `text*`
    TextStar,
    /// `list_item+`
    ListItemPlus,
    /// `list_item*`
    ListItemStar,
    /// `paragraph block*`
    ParagraphBlockStar,
    /// empty
    Empty,
}

impl ContentMatch<MD> for MarkdownContentMatch {
    fn match_type(self, r#type: MarkdownNodeType) -> Option<Self> {
        match self {
            Self::Star => Some(Self::Star),
            Self::InlineStar => then_some(r#type.is_inline(), Self::InlineStar),
            Self::BlockPlus | Self::BlockStar => then_some(r#type.is_block(), Self::BlockStar),
            Self::OrTextImageStar => then_some(
                matches!(r#type, MarkdownNodeType::Text | MarkdownNodeType::Image),
                Self::OrTextImageStar,
            ),
            Self::TextStar => then_some(matches!(r#type, MarkdownNodeType::Text), Self::TextStar),
            Self::ListItemPlus | Self::ListItemStar => then_some(
                matches!(r#type, MarkdownNodeType::ListItem),
                Self::ListItemStar,
            ),
            Self::ParagraphBlockStar => then_some(
                matches!(r#type, MarkdownNodeType::Paragraph),
                Self::BlockStar,
            ),
            Self::Empty => None,
        }
    }

    fn match_fragment_range<R: RangeBounds<usize>>(
        self,
        fragment: &Fragment<MD>,
        range: R,
    ) -> Option<Self> {
        let start = util::from(&range);
        let end = util::to(&range, fragment.child_count());

        let mut test = self;
        for child in &fragment.children()[start..end] {
            let child_type = child.r#type();
            match test.match_type(child_type) {
                Some(next) => {
                    test = next;
                }
                None => {
                    // eprintln!("Test failed at {child_type:?} for {fragment:?}");
                    return None;
                }
            }
        }
        Some(test)
    }

    fn valid_end(self) -> bool {
        match self {
            MarkdownContentMatch::Star => true,
            MarkdownContentMatch::InlineStar => true,
            MarkdownContentMatch::BlockPlus => false,
            MarkdownContentMatch::BlockStar => true,
            MarkdownContentMatch::OrTextImageStar => true,
            MarkdownContentMatch::TextStar => true,
            MarkdownContentMatch::ListItemPlus => false,
            MarkdownContentMatch::ListItemStar => true,
            MarkdownContentMatch::ParagraphBlockStar => true,
            MarkdownContentMatch::Empty => true,
        }
    }
}

impl MarkdownContentMatch {
    pub(crate) fn compatible(self, other: Self) -> bool {
        match self {
            Self::Star => true,
            Self::InlineStar => matches!(
                other,
                Self::InlineStar | Self::OrTextImageStar | Self::TextStar
            ),
            Self::BlockPlus | Self::BlockStar => matches!(
                other,
                Self::BlockPlus | Self::ParagraphBlockStar | Self::BlockStar
            ),
            Self::OrTextImageStar => matches!(
                other,
                Self::InlineStar | Self::OrTextImageStar | Self::TextStar
            ),
            Self::TextStar => matches!(
                other,
                Self::InlineStar | Self::OrTextImageStar | Self::TextStar
            ),
            Self::ListItemPlus | Self::ListItemStar => {
                matches!(other, Self::ListItemPlus | Self::ListItemStar)
            }
            Self::ParagraphBlockStar => matches!(other, Self::BlockPlus | Self::ParagraphBlockStar),
            Self::Empty => false,
        }
    }
}
