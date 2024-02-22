use super::{
    attrs::FootnoteAttrs, BulletListAttrs, CodeBlockAttrs, HeadingAttrs, ImageAttrs, LinkAttrs,
    MarkdownMark, MarkdownNode, OrderedListAttrs, MD,
};
use crate::model::{AttrNode, Block, Fragment, Leaf, MarkSet, Node, Text, TextNode};
use displaydoc::Display;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use std::{convert::TryInto, num::TryFromIntError};
use thiserror::Error;

/// Errors that can occur when reading a markdown file
#[derive(Debug, PartialEq, Display, Error)]
pub enum FromMarkdownError {
    /// Heading level too deep
    LevelMismatch(#[from] TryFromIntError),
    /// Not supported: `{0}`
    NotSupported(&'static str),
    /// The stack was empty
    StackEmpty,
    /// Event mismatch
    MisplacedEndTag(&'static str, Attrs),
    /// No children allowed in {0:?}
    NoChildrenAllowed(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attrs {
    Doc,
    Paragraph,
    Heading(HeadingAttrs),
    Blockquote,
    CodeBlock(CodeBlockAttrs),
    OrderedList(OrderedListAttrs),
    BulletList(BulletListAttrs),
    ListItem,
    Image(ImageAttrs),
    FootnoteDefinition(FootnoteAttrs),
}

/// Creates a MarkdownNode::Doc from a text
pub fn from_markdown(text: &str) -> Result<MarkdownNode, FromMarkdownError> {
    let mut options = Options::empty();
    // options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    // options.insert(Options::ENABLE_STRIKETHROUGH);
    // options.insert(Options::ENABLE_TASKLISTS);
    // options.insert(Options::ENABLE_SMART_PUNCTUATION);

    // options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    // options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    // options.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS)

    let parser = Parser::new_ext(text, options);
    let mut d = MarkdownDeserializer::default();
    d.deserialize(parser)
}

#[derive(Default)]
pub struct MarkdownDeserializer {
    stack: Vec<(Vec<MarkdownNode>, Attrs)>,
    mark_set: MarkSet<MD>,
}

impl MarkdownDeserializer {
    /*#[must_use]
    fn push_text(&mut self) -> Result<(), FromMarkdownError> {
        let last = self.stack.last_mut().ok_or(FromMarkdownError::StackEmpty)?;
        if !self.text.is_empty() {
            last.0.push(MarkdownNode::Text(TextNode {
                marks: self.mark_set.clone(),
                text: Text::from(std::mem::take(&mut self.text)),
            }));
        }
        Ok(())
    }*/

    fn push_stack(&mut self, attrs: Attrs) {
        self.stack.push((Vec::new(), attrs));
    }

    fn pop_stack(&mut self) -> Result<(Vec<MarkdownNode>, Attrs), FromMarkdownError> {
        let popped = self.stack.pop().ok_or(FromMarkdownError::StackEmpty)?;
        Ok(popped)
    }

    fn add_content(&mut self, node: MarkdownNode) -> Result<(), FromMarkdownError> {
        let last = self.stack.last_mut().ok_or(FromMarkdownError::StackEmpty)?;
        last.0.push(node);
        Ok(())
    }

    fn deserialize(&mut self, parser: Parser) -> Result<MarkdownNode, FromMarkdownError> {
        self.push_stack(Attrs::Doc);
        for event in parser {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Paragraph => {
                        self.stack.push((Vec::new(), Attrs::Paragraph));
                    }
                    Tag::Heading {
                        level,
                        attrs: _,
                        id: _,
                        classes: _,
                    } => {
                        let level = match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        };
                        self.stack
                            .push((Vec::new(), Attrs::Heading(HeadingAttrs { level })));
                    }
                    Tag::BlockQuote => {
                        self.stack.push((Vec::new(), Attrs::Blockquote));
                    }
                    Tag::CodeBlock(kind) => {
                        let params = if let CodeBlockKind::Fenced(params) = kind {
                            params.to_string()
                        } else {
                            String::new()
                        };
                        self.stack
                            .push((Vec::new(), Attrs::CodeBlock(CodeBlockAttrs { params })));
                    }
                    Tag::List(ord) => {
                        if let Some(order) = ord {
                            self.stack.push((
                                Vec::new(),
                                Attrs::OrderedList(OrderedListAttrs {
                                    order: order.try_into()?, // TODO: other error
                                    tight: false,
                                }),
                            ))
                        } else {
                            self.stack.push((
                                Vec::new(),
                                Attrs::BulletList(BulletListAttrs { tight: false }),
                            ));
                        }
                    }
                    Tag::Item => {
                        self.stack.push((Vec::new(), Attrs::ListItem));
                    }
                    Tag::FootnoteDefinition(label) => {
                        self.stack.push((
                            Vec::new(),
                            Attrs::FootnoteDefinition(FootnoteAttrs {
                                label: label.to_string(),
                            }),
                        ));
                    }
                    Tag::Table(_) => {
                        // Requires opt-in feature
                        return Err(FromMarkdownError::NotSupported("Table"));
                    }
                    Tag::TableHead => {
                        // Requires opt-in feature
                        return Err(FromMarkdownError::NotSupported("TableHead"));
                    }
                    Tag::TableRow => {
                        // Requires opt-in feature
                        return Err(FromMarkdownError::NotSupported("TableRow"));
                    }
                    Tag::TableCell => {
                        // Requires opt-in feature
                        return Err(FromMarkdownError::NotSupported("TableCell"));
                    }
                    Tag::Emphasis => {
                        self.mark_set.add(&MarkdownMark::Em);
                    }
                    Tag::Strong => {
                        self.mark_set.add(&MarkdownMark::Strong);
                    }
                    Tag::Strikethrough => {
                        // Requires opt-in feature
                        return Err(FromMarkdownError::NotSupported("Strikethrough"));
                    }
                    Tag::HtmlBlock => {
                        // return Err(FromMarkdownError::NotSupported("HtmlBlock"));
                    }
                    Tag::MetadataBlock(_) => {
                        // Requires opt-in feature
                        return Err(FromMarkdownError::NotSupported("MetadataBlock"));
                    }
                    Tag::Link {
                        link_type: _,
                        dest_url,
                        title,
                        id: _,
                    } => {
                        self.mark_set.add(&MarkdownMark::Link {
                            attrs: LinkAttrs {
                                href: dest_url.to_string(),
                                title: title.to_string(),
                            },
                        });
                    }
                    Tag::Image {
                        link_type: _,
                        dest_url,
                        title,
                        id: _,
                    } => {
                        self.push_stack(Attrs::Image(ImageAttrs {
                            src: dest_url.to_string(),
                            alt: String::new(),
                            title: title.to_string(),
                        }));
                    }
                },
                Event::End(tag) => match tag {
                    TagEnd::Paragraph => {
                        let (content, attrs) = self.pop_stack()?;
                        if matches!(attrs, Attrs::Paragraph) {
                            let p = MarkdownNode::Paragraph(Block {
                                content: Fragment::from(content),
                            });
                            self.add_content(p)?;
                        } else {
                            return Err(FromMarkdownError::MisplacedEndTag("Paragraph", attrs));
                        }
                    }
                    TagEnd::Heading(_) => {
                        let (content, attrs) = self.pop_stack()?;
                        if let Attrs::Heading(attrs) = attrs {
                            let h = MarkdownNode::Heading(AttrNode {
                                attrs,
                                content: Fragment::from(content),
                            });
                            self.add_content(h)?;
                        } else {
                            return Err(FromMarkdownError::MisplacedEndTag("Heading", attrs));
                        }
                    }
                    TagEnd::BlockQuote => {
                        let (content, attrs) = self.pop_stack()?;
                        if let Attrs::Blockquote = attrs {
                            let b = MarkdownNode::Blockquote(Block {
                                content: Fragment::from(content),
                            });
                            self.add_content(b)?;
                        } else {
                            return Err(FromMarkdownError::MisplacedEndTag("BlockQuote", attrs));
                        }
                    }
                    TagEnd::CodeBlock => {
                        let (mut content, attrs) = self.pop_stack()?;
                        if let Attrs::CodeBlock(attrs) = attrs {
                            if let Some(MarkdownNode::Text(t)) = content.last_mut() {
                                t.text.remove_last_newline();
                            }
                            let cb = MarkdownNode::CodeBlock(AttrNode {
                                attrs,
                                content: Fragment::from(content),
                            });
                            self.add_content(cb)?;
                        } else {
                            return Err(FromMarkdownError::MisplacedEndTag("CodeBlock", attrs));
                        }
                    }
                    TagEnd::List(_) => {
                        let (content, attrs) = self.pop_stack()?;
                        match attrs {
                            Attrs::BulletList(attrs) => {
                                let l = MarkdownNode::BulletList(AttrNode {
                                    attrs,
                                    content: Fragment::from(content),
                                });
                                self.add_content(l)?;
                            }
                            Attrs::OrderedList(attrs) => {
                                let l = MarkdownNode::OrderedList(AttrNode {
                                    attrs,
                                    content: Fragment::from(content),
                                });
                                self.add_content(l)?;
                            }
                            _ => {
                                return Err(FromMarkdownError::MisplacedEndTag("List", attrs));
                            }
                        }
                    }
                    TagEnd::Item => {
                        let (content, attrs) = self.pop_stack()?;
                        if let Attrs::ListItem = attrs {
                            let cb = MarkdownNode::ListItem(Block {
                                content: Fragment::from(content),
                            });
                            self.add_content(cb)?;
                        }
                    }
                    TagEnd::FootnoteDefinition => {
                        let (mut content, attrs) = self.pop_stack()?;
                        if let Attrs::FootnoteDefinition(attrs) = attrs {
                            if let Some(MarkdownNode::Text(t)) = content.last_mut() {
                                t.text.remove_last_newline();
                            }
                            let cb = MarkdownNode::FootnoteDefinition(AttrNode {
                                attrs,
                                content: Fragment::from(content),
                            });
                            self.add_content(cb)?;
                        } else {
                            return Err(FromMarkdownError::MisplacedEndTag(
                                "FootnoteDefinition",
                                attrs,
                            ));
                        }
                    }
                    TagEnd::Table => {
                        return Err(FromMarkdownError::NotSupported("Table"));
                    }
                    TagEnd::TableHead => {
                        return Err(FromMarkdownError::NotSupported("TableHead"));
                    }
                    TagEnd::TableRow => {
                        return Err(FromMarkdownError::NotSupported("TableRow"));
                    }
                    TagEnd::TableCell => {
                        return Err(FromMarkdownError::NotSupported("TableCell"));
                    }
                    TagEnd::HtmlBlock => {}
                    TagEnd::MetadataBlock(_) => {
                        return Err(FromMarkdownError::NotSupported("MetadataBlock"));
                    }
                    TagEnd::Emphasis => {
                        self.mark_set.remove(&MarkdownMark::Em);
                    }
                    TagEnd::Strong => {
                        self.mark_set.remove(&MarkdownMark::Strong);
                    }
                    TagEnd::Strikethrough => {
                        return Err(FromMarkdownError::NotSupported("Strikethrough"));
                    }
                    TagEnd::Link => {
                        self.mark_set
                            .remove_matching(|m| matches!(m, &MarkdownMark::Link { .. }));
                    }
                    TagEnd::Image { .. } => {
                        let (content, attrs) = self.pop_stack()?;
                        if let Attrs::Image(mut attrs) = attrs {
                            let alt: String = content
                                .into_iter()
                                .map(|node| node.text_content())
                                .collect();

                            attrs.alt = alt;
                            let cb = MarkdownNode::Image(Leaf { attrs });
                            self.add_content(cb)?;
                        } else {
                            return Err(FromMarkdownError::MisplacedEndTag("Image", attrs));
                        }
                    }
                },
                Event::Text(text) => {
                    self.add_content(MarkdownNode::Text(TextNode {
                        text: Text::from(text.to_string()),
                        marks: self.mark_set.clone(),
                    }))?;
                }
                Event::Code(text) => {
                    let mut marks = self.mark_set.clone();
                    marks.add(&MarkdownMark::Code);
                    self.add_content(MarkdownNode::Text(TextNode {
                        text: Text::from(text.to_string()),
                        marks,
                    }))?;
                }
                Event::InlineHtml(html) => {
                    let mut marks = self.mark_set.clone();
                    marks.add(&MarkdownMark::HtmlTag);
                    self.add_content(MarkdownNode::Text(TextNode {
                        text: Text::from(html.to_string()),
                        marks,
                    }))?;
                }
                Event::Html(html) => {
                    let mut marks = self.mark_set.clone();
                    marks.add(&MarkdownMark::HtmlTag);
                    self.add_content(MarkdownNode::Text(TextNode {
                        text: Text::from(html.to_string()),
                        marks,
                    }))?;
                }
                Event::FootnoteReference(label) => {
                    let mut marks = self.mark_set.clone();
                    marks.add(&MarkdownMark::Footnote {
                        attrs: FootnoteAttrs {
                            label: label.to_string(),
                        },
                    });
                    self.add_content(MarkdownNode::Text(TextNode {
                        text: Text::from(label.to_string()),
                        marks,
                    }))?;
                }
                Event::SoftBreak => {
                    self.add_content(MarkdownNode::Text(TextNode {
                        text: Text::from("\n".to_string()),
                        marks: self.mark_set.clone(),
                    }))?;
                }
                Event::HardBreak => {
                    self.add_content(MarkdownNode::HardBreak)?;
                }
                Event::Rule => {
                    self.add_content(MarkdownNode::HorizontalRule)?;
                }
                Event::TaskListMarker(_) => {
                    // This requires opt-in feature
                    return Err(FromMarkdownError::NotSupported("TaskListMarker"));
                }
            }
        }
        let (content, attrs) = self.pop_stack()?;
        if let Attrs::Doc = attrs {
            Ok(MarkdownNode::Doc(Block {
                content: Fragment::from(content),
            }))
        } else {
            Err(FromMarkdownError::MisplacedEndTag("Doc", attrs))
        }
    }
}

#[cfg(test)]
mod tests {
    // use pulldown_cmark::{CowStr, Event, HeadingLevel, Parser, Tag, TagEnd};

    use super::from_markdown;

    #[test]
    fn parser_tests() {
        test_runner::test_snapshots("md", "parsed", |input| {
            let ast = match from_markdown(input) {
                Ok(ast) => ast,
                Err(e) => return format!("Error: {}", e),
            };

            serde_json::to_string_pretty(&ast).unwrap()
        })
        .unwrap();
    }

    // #[test]
    // fn test_alerts() {
    //     let test_string = "\
    //     ### Alert Area\n\
    //     \n\
    //     :::success\n\
    //     Yes :tada:\n\
    //     :::\n\
    //     ";

    //     let p = Parser::new(test_string);
    //     let v: Vec<Event> = p.collect();
    //     assert_eq!(
    //         v,
    //         vec![
    //             Event::Start(Tag::Heading {
    //                 level: HeadingLevel::H3,
    //                 attrs: Default::default(),
    //                 classes: Default::default(),
    //                 id: Default::default(),
    //             }),
    //             Event::Text(CowStr::Borrowed("Alert Area")),
    //             Event::End(TagEnd::Heading(HeadingLevel::H3)),
    //             Event::Start(Tag::Paragraph),
    //             Event::Text(CowStr::Borrowed(":::success")),
    //             Event::SoftBreak,
    //             Event::Text(CowStr::Borrowed("Yes :tada:")),
    //             Event::SoftBreak,
    //             Event::Text(CowStr::Borrowed(":::")),
    //             Event::End(TagEnd::Paragraph),
    //         ]
    //     );
    // }
}
