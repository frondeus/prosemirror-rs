use super::{attrs::Alignment, print_markdown, MarkdownMark, MarkdownNode, MD};
use crate::model::{AttrNode, Block, Fragment, Leaf, Node};
use displaydoc::Display;
use pulldown_cmark::{
    CodeBlockKind, CowStr, Event, HeadingLevel, InlineStr, LinkType, MetadataBlockKind, Tag,
};
// use pulldown_cmark_to_cmark::cmark;
use thiserror::Error;

/// Possible error when generating markdown
#[derive(Debug, Clone, PartialEq, Eq, Display, Error)]
pub struct ToMarkdownError {
    /// The inner error
    inner: std::fmt::Error,
}

impl From<std::fmt::Error> for ToMarkdownError {
    fn from(e: std::fmt::Error) -> ToMarkdownError {
        Self { inner: e }
    }
}

/// Turn a markdown document into a string
pub fn to_markdown(doc: &MarkdownNode) -> Result<String, ToMarkdownError> {
    let mut buf = String::with_capacity(doc.node_size() + 128);
    let events = MarkdownSerializer::new(doc);
    print_markdown::Printer::print(events, &mut buf);
    Ok(buf)
}

struct MarkdownSerializer<'a> {
    inner: Vec<(&'a MarkdownNode, usize)>,
    marks: Vec<&'a MarkdownMark>,
    stack: Vec<Event<'a>>,
}

impl<'a> MarkdownSerializer<'a> {
    fn new(doc: &'a MarkdownNode) -> Self {
        Self {
            inner: vec![(doc, 0)],
            marks: vec![],
            stack: vec![],
        }
    }
}

#[deprecated]
fn mark_tag(mark: &MarkdownMark) -> Tag {
    match mark {
        MarkdownMark::Strong => Tag::Strong,
        MarkdownMark::Em => Tag::Emphasis,
        MarkdownMark::Strikethrough => Tag::Strikethrough,
        MarkdownMark::Link { attrs } => Tag::Link {
            link_type: LinkType::Inline,
            dest_url: CowStr::Borrowed(attrs.href.as_str()),
            title: CowStr::Borrowed(attrs.title.as_str()),
            id: String::new().into(),
        },
        MarkdownMark::Code => unimplemented!("Should not be pushed on the mark stack: Code"),
        MarkdownMark::Footnote { attrs: _ } => {
            unimplemented!("Should not be pushed on the mark stack: Footnote")
        }
        MarkdownMark::HtmlTag => {
            unimplemented!("Should not be pushed on the mark stack: HtmlTag")
        }
    }
}

fn mark_to_start_event<'a>(mark: &'a MarkdownMark, text: CowStr<'a>) -> Event<'a> {
    match mark {
        MarkdownMark::Strong => Event::Start(Tag::Strong),
        MarkdownMark::Em => Event::Start(Tag::Emphasis),
        MarkdownMark::Strikethrough => Event::Start(Tag::Strikethrough),
        MarkdownMark::Link { attrs } => Event::Start(Tag::Link {
            link_type: LinkType::Inline,
            dest_url: CowStr::Borrowed(attrs.href.as_str()),
            title: CowStr::Borrowed(attrs.title.as_str()),
            id: String::new().into(),
        }),
        MarkdownMark::Code => Event::Code(text),
        MarkdownMark::Footnote { attrs: _ } => Event::FootnoteReference(text),
        MarkdownMark::HtmlTag => Event::Html(text),
    }
}

impl<'a> MarkdownSerializer<'a> {
    fn process_content(
        &mut self,
        index: usize,
        content: &'a Fragment<MD>,
        node: &'a MarkdownNode,
    ) -> bool {
        if let Some(child) = content.maybe_child(index) {
            self.inner.push((node, index + 1));
            self.inner.push((child, 0));
            false
        } else {
            true
        }
    }

    fn process_attr_node<A, F>(
        &mut self,
        index: usize,
        content: &'a Fragment<MD>,
        attrs: &'a A,
        node: &'a MarkdownNode,
        map: F,
    ) -> Option<Event<'a>>
    where
        F: FnOnce(&'a A) -> Tag<'a>,
    {
        if index == 0 {
            if let Some(mark) = self.marks.pop() {
                self.inner.push((node, 0));
                // Deprecation: While it should not really work, all tests for now pass so I'm not touching it
                #[allow(deprecated)]
                return Some(Event::End(mark_tag(mark).to_end()));
            }
        }
        let last = self.process_content(index, content, node);
        if index == 0 {
            if last {
                // close the tag next
                self.inner.push((node, index + 1));
            }
            Some(Event::Start(map(attrs)))
        } else if last {
            if let Some(mark) = self.marks.pop() {
                self.inner.push((node, index));
                // Deprecation: While it should not really work, all tests for now pass so I'm not touching it
                #[allow(deprecated)]
                return Some(Event::End(mark_tag(mark).to_end()));
            }
            let tag = map(attrs);
            if matches!(&tag, Tag::CodeBlock(..)) {
                self.stack.push(Event::End(tag.to_end()));
                Some(Event::Text(CowStr::Inlined(InlineStr::from('\n'))))
            } else {
                Some(Event::End(tag.to_end()))
            }
        } else {
            self.next()
        }
    }
}

impl<'a> Iterator for MarkdownSerializer<'a> {
    type Item = Event<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ev) = self.stack.pop() {
            return Some(ev);
        }

        if let Some((node, index)) = self.inner.pop() {
            match node {
                MarkdownNode::Doc(Block { content }) => {
                    self.process_content(index, content, node);
                    self.next()
                }
                MarkdownNode::Heading(AttrNode { attrs, content }) => {
                    self.process_attr_node(index, content, attrs, node, |attrs| Tag::Heading {
                        level: match attrs.level {
                            0 | 1 => HeadingLevel::H1,
                            2 => HeadingLevel::H2,
                            3 => HeadingLevel::H3,
                            4 => HeadingLevel::H4,
                            5 => HeadingLevel::H5,
                            6.. => HeadingLevel::H6,
                        },
                        attrs: Default::default(),
                        classes: Default::default(),
                        id: Default::default(),
                    })
                }
                MarkdownNode::CodeBlock(AttrNode { attrs, content }) => {
                    self.process_attr_node(index, content, attrs, node, |attrs| {
                        Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::Borrowed(&attrs.params)))
                    })
                }
                MarkdownNode::Text(text_node) => {
                    if let Some(last) = self.marks.last().copied() {
                        if !text_node.marks.contains(last) {
                            self.inner.push((node, index));
                            self.marks.pop();
                            // Deprecation: While it should not really work, all tests for now pass so I'm not touching it
                            #[allow(deprecated)]
                            return Some(Event::End(mark_tag(last).to_end()));
                        }
                    }
                    let text = CowStr::Borrowed(text_node.text.as_str());

                    let mut custom_event = None;
                    for mark in &text_node.marks {
                        let event = mark_to_start_event(mark, text.clone());

                        match event {
                            Event::Start(start) if !self.marks.contains(&mark) => {
                                self.inner.push((node, index));
                                self.marks.push(mark);
                                return Some(Event::Start(start));
                            }
                            Event::Start(_) => {}
                            event => {
                                custom_event = Some(event);
                            }
                        }
                    }
                    match custom_event {
                        Some(event) => Some(event),
                        None => Some(Event::Text(text)),
                    }
                }
                MarkdownNode::Blockquote(Block { content }) => {
                    self.process_attr_node(index, content, &(), node, |()| Tag::BlockQuote)
                }
                MarkdownNode::Paragraph(Block { content }) => {
                    self.process_attr_node(index, content, &(), node, |()| Tag::Paragraph)
                }
                MarkdownNode::BulletList(AttrNode { attrs, content }) => {
                    self.process_attr_node(index, content, attrs, node, |_| Tag::List(None))
                }
                MarkdownNode::OrderedList(AttrNode { attrs, content }) => {
                    self.process_attr_node(index, content, attrs, node, |_| {
                        Tag::List(Some(attrs.order as u64))
                    })
                }
                MarkdownNode::ListItem(Block { content }) => {
                    self.process_attr_node(index, content, &(), node, |()| Tag::Item)
                }
                MarkdownNode::HorizontalRule => Some(Event::Rule),
                MarkdownNode::HardBreak => {
                    // todo: inline marks
                    Some(Event::HardBreak)
                }
                MarkdownNode::TaskListMarker(Leaf { attrs }) => {
                    Some(Event::TaskListMarker(attrs.checked))
                }
                MarkdownNode::Metadata(Block { content }) => {
                    self.process_attr_node(index, content, &(), node, |()| {
                        Tag::MetadataBlock(MetadataBlockKind::YamlStyle)
                    })
                }
                MarkdownNode::Image(AttrNode { attrs, content }) => {
                    self.process_attr_node(index, content, &(), node, |()| Tag::Image {
                        link_type: LinkType::Inline,
                        dest_url: CowStr::Borrowed(attrs.src.as_str()),
                        title: CowStr::Borrowed(attrs.title.as_str()),
                        id: String::new().into(),
                    })
                }
                MarkdownNode::FootnoteDefinition(AttrNode { attrs, content }) => self
                    .process_attr_node(index, content, attrs, node, |attrs| {
                        Tag::FootnoteDefinition(CowStr::Borrowed(attrs.label.as_str()))
                    }),
                MarkdownNode::Table(AttrNode { attrs, content }) => {
                    self.process_attr_node(index, content, attrs, node, |attrs| {
                        Tag::Table(
                            attrs
                                .alignment
                                .iter()
                                .map(|a| match a {
                                    Alignment::None => pulldown_cmark::Alignment::None,
                                    Alignment::Left => pulldown_cmark::Alignment::Left,
                                    Alignment::Right => pulldown_cmark::Alignment::Right,
                                    Alignment::Center => pulldown_cmark::Alignment::Center,
                                })
                                .collect(),
                        )
                    })
                }
                MarkdownNode::TableHead(Block { content }) => {
                    self.process_attr_node(index, content, &(), node, |_| Tag::TableHead)
                }
                MarkdownNode::TableRow(Block { content }) => {
                    self.process_attr_node(index, content, &(), node, |_| Tag::TableRow)
                }
                MarkdownNode::TableCell(Block { content }) => {
                    self.process_attr_node(index, content, &(), node, |_| Tag::TableCell)
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use super::to_markdown;
    use crate::markdown::from_markdown;

    #[test]
    fn printer_tests() {
        test_runner::test_snapshots("md", "printed", |input| {
            let node = from_markdown(input).unwrap();
            let md = to_markdown(&node).unwrap();
            format!("~~~~~~~~~\n{md}\n~~~~~~~~~")
        })
        .unwrap();
    }
}
