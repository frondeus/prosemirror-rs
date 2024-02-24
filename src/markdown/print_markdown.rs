//! This module contains the logic of writing pulldown cmark events back to markdown format.
//! We are not using `pulldown-cmark-to-cmark` because it does not support latest version of the parser library.

use std::ops::Range;

use pulldown_cmark::{CodeBlockKind, Event, MetadataBlockKind, Tag, TagEnd};

pub struct Printer<'a> {
    /// The buffer to write the markdown to.
    /// Do not use this directly, use `buffer()` and `buffer_mut()` instead.
    buffer: &'a mut String,
    /// Pulldown cmark since 0.10 has an asynchronous design, where the starting tag contains
    /// more information than the ending tag. This stack is used to store the starting tags.
    tag_stack: Vec<StackItem<'a>>,
    /// When processing new block, should we separate it by a newline?
    separate_by_newline: NewlineStrategy,
    /// If in codeblock, how many backticks add to the start and end of the block?
    codeblock_backticks: usize,

    /// In case there is a table, we want to render it to the separate string in order to align the columns.
    /// Additionally we want to store some contextual information about where row begins, where ends,
    /// what is the max number of columns in the table and its size.
    table_context: Option<TableCtx>,
}

#[derive(Debug, Default)]
struct TableCtx {
    /// This is owned string, used internally as a buffer to store the table.
    buffer: String,
    header: Vec<Range<usize>>,
    /// Row is a vector of cells, so rows are vector of vectors
    rows: Vec<Vec<Range<usize>>>,
}

impl TableCtx {
    /// Get number of columns.
    /// We do not assume that every row has the same number of columns.
    /// Therefore we need to iterate over all rows and find the maximum number of columns.
    pub fn max_column(&self) -> usize {
        let rows = std::iter::once(&self.header).chain(self.rows.iter());

        rows.map(|row| row.len()).max().unwrap_or(0)
    }

    /// Gets maximum length for every column
    pub fn max_column_length(&self) -> Vec<usize> {
        let max_column = self.max_column();

        let mut out = Vec::with_capacity(max_column);
        for idx in 0..max_column {
            let header = self.header.get(idx).map(|range| range.end - range.start);
            let rows = self
                .rows
                .iter()
                .map(|row| row.get(idx).map(|range| range.end - range.start))
                .fold(None::<usize>, |acc, len| match (acc, len) {
                    (Some(acc), Some(len)) => Some(acc.max(len)),
                    (Some(acc), None) => Some(acc),
                    (None, Some(len)) => Some(len),
                    (None, None) => None,
                });

            if let Some(max) = header.max(rows) {
                out.push(max.max(3));
            }
        }
        out
    }
}

struct StackItem<'a> {
    /// The tag that is currently being processed.
    tag: Tag<'a>,
    /// The range of the start input of the tag.
    /// Usefulness:
    /// - When the tag is codeblock start, we neeed to save its position in the buffer to add backticks later.
    range: Range<usize>,
    /// If in the list,  what is the index of last item?
    list_counter: Option<u64>,
}

#[derive(Debug, Copy, Clone)]
enum NewlineStrategy {
    None,
    /// One \n
    Once,
    /// Two \n
    Block,
}

impl<'a> Printer<'a> {
    fn buffer(&self) -> &String {
        self.table_context
            .as_ref()
            .map(|t| &t.buffer)
            .unwrap_or(self.buffer)
    }
    fn buffer_mut(&mut self) -> &mut String {
        self.table_context
            .as_mut()
            .map(|t| &mut t.buffer)
            .unwrap_or(self.buffer)
    }

    fn print_str(&mut self, s: &str) {
        self.buffer_mut().push_str(s);
    }

    fn print_string(&mut self, s: String) {
        self.buffer_mut().push_str(&s);
    }

    fn buffer_len(&self) -> usize {
        self.buffer().len()
    }

    fn insert_str(&mut self, pos: usize, s: &str) {
        self.buffer_mut().insert_str(pos, s);
    }

    fn print_newline(&mut self) {
        match self.separate_by_newline {
            NewlineStrategy::None => {}
            NewlineStrategy::Once => self.print_str("\n"),
            NewlineStrategy::Block => self.print_str("\n\n"),
        }
        self.separate_by_newline = NewlineStrategy::None;
    }

    fn print_block(&mut self, s: &str) {
        self.print_newline();
        self.print_str(s);
    }

    fn tag_is_block(&mut self) {
        self.separate_by_newline = NewlineStrategy::Block;
    }

    fn in_tag(&self, f: impl Fn(&Tag<'a>) -> bool) -> bool {
        self.tag_stack.iter().any(|it| f(&it.tag))
    }

    pub fn print(events: impl Iterator<Item = Event<'a>>, buffer: &'a mut String) {
        let mut printer = Printer {
            buffer,
            tag_stack: Vec::new(),
            separate_by_newline: NewlineStrategy::None,
            codeblock_backticks: 0,
            table_context: None,
        };
        for event in events {
            printer.print_event(event);
        }
    }

    fn print_event(&mut self, event: Event<'a>) {
        match event {
            Event::Start(start) => self.print_tag_start(start),
            Event::End(end) => self.print_tag_end(end),
            Event::Text(text) => {
                self.print_str(&text);
                if text.ends_with('\n') {
                    // Is in blockquote
                    if self.in_tag(|tag| matches!(tag, Tag::BlockQuote)) {
                        self.print_str("> ");
                    }
                }
                if self.in_tag(|tag| matches!(tag, Tag::CodeBlock(CodeBlockKind::Fenced(_)))) {
                    // Count how many backticks in a row are there in the text
                    let (_, max_acc) = text.chars().fold((0, 0), |(acc, max_acc), c| {
                        if c == '`' {
                            (acc + 1, (acc + 1).max(max_acc))
                        } else {
                            (0, max_acc)
                        }
                    });
                    self.codeblock_backticks = self.codeblock_backticks.max(max_acc);
                }
            }
            Event::Code(code) => {
                // Count how many backticks in a row are there in the text
                let (_, backticks) = code.chars().fold((0, 0), |(acc, max_acc), c| {
                    if c == '`' {
                        (acc + 1, (acc + 1).max(max_acc))
                    } else {
                        (0, max_acc)
                    }
                });
                let backticks = "`".repeat(backticks + 1);
                self.print_string(format!("{backticks}{code}{backticks}"));
            }
            Event::Html(h) => {
                self.print_block(&h);
            }
            Event::InlineHtml(h) => self.print_str(&h),
            Event::FootnoteReference(reference) => self.print_string(format!("[^{reference}]")),
            Event::SoftBreak => self.print_str("\n"),
            Event::HardBreak => self.print_str("  \n"),
            Event::Rule => {
                self.print_block("---");
                self.tag_is_block()
            }
            Event::TaskListMarker(true) => self.print_str("[x] "),
            Event::TaskListMarker(false) => self.print_str("[ ] "),
        }
    }

    fn print_tag_start(&mut self, start: Tag<'a>) {
        let mut range_before = self.buffer_len();
        let mut list_from = None;
        match &start {
            Tag::Paragraph => self.print_newline(),
            Tag::Heading {
                level,
                id: _,
                classes: _,
                attrs: _,
            } => self.print_block(&format!("{} ", "#".repeat(*level as usize))),
            Tag::BlockQuote => {
                self.print_block("> ");
            }
            Tag::CodeBlock(CodeBlockKind::Indented) => todo!(),
            Tag::CodeBlock(CodeBlockKind::Fenced(lang)) => {
                self.print_newline();
                range_before = self.buffer_len();
                self.codeblock_backticks = 0;
                self.print_str(&format!("{lang}\n"));
            }
            Tag::HtmlBlock => todo!(),
            Tag::List(from) => {
                self.print_newline();
                list_from = *from;
            }
            Tag::Item => {
                self.print_newline();
                let list_identation = self
                    .tag_stack
                    .iter()
                    .filter(|it| matches!(it.tag, Tag::List(_)))
                    .count()
                    - 1;
                let list = self.tag_stack.last().map(|list| list.list_counter).unwrap();
                self.print_str(" ".repeat(list_identation * 2).as_str());
                match list {
                    Some(from) => {
                        self.print_str(&format!("{}. ", from));
                        self.tag_stack.last_mut().unwrap().list_counter = Some(from + 1);
                    }
                    None => {
                        self.print_str("- ");
                    }
                }
                self.separate_by_newline = NewlineStrategy::Once;
            }
            Tag::FootnoteDefinition(label) => {
                self.print_block(&format!("[^{label}]: "));
            }
            Tag::Table(_) => {
                self.print_newline();
                self.table_context = Some(Default::default());
            }
            Tag::TableHead => {}
            Tag::TableRow => {
                let ctx = self.table_context.as_mut().unwrap();
                ctx.rows.push(Vec::new());
            }
            Tag::TableCell => {
                // self.print_str("| ");
            }
            Tag::Emphasis => {
                self.print_str("_");
            }
            Tag::Strong => self.print_str("**"),
            Tag::Strikethrough => self.print_str("~"),
            Tag::Link {
                link_type: _,
                dest_url: _,
                title: _,
                id: _,
            } => self.print_str("["),
            Tag::Image {
                link_type: _,
                dest_url: _,
                title: _,
                id: _,
            } => self.print_str("!["),
            Tag::MetadataBlock(MetadataBlockKind::YamlStyle) => {
                self.print_newline();
                self.print_str("---\n");
            }
            Tag::MetadataBlock(MetadataBlockKind::PlusesStyle) => todo!(),
        }
        let range_after = self.buffer_len();
        let range = range_before..range_after;
        self.tag_stack.push(StackItem {
            tag: start,
            range,
            list_counter: list_from,
        });
    }

    fn print_tag_end(&mut self, end: TagEnd) {
        let start = self.tag_stack.pop().unwrap();

        match (start.tag, end) {
            (Tag::Paragraph, TagEnd::Paragraph) => self.tag_is_block(),
            (
                Tag::Heading {
                    level: _,
                    id: _,
                    classes: _,
                    attrs: _,
                },
                TagEnd::Heading(_),
            ) => self.separate_by_newline = NewlineStrategy::Once,
            (Tag::BlockQuote, TagEnd::BlockQuote) => {
                self.tag_is_block();
            }
            (Tag::CodeBlock(CodeBlockKind::Fenced(_)), TagEnd::CodeBlock) => {
                let backticks = self.codeblock_backticks.max(2) + 1;
                let backticks = "`".repeat(backticks);
                let pos_to_insert_backticks = start.range.start;
                self.insert_str(pos_to_insert_backticks, &backticks);
                self.print_str(&backticks);
                self.tag_is_block();
            }
            (Tag::CodeBlock(CodeBlockKind::Indented), TagEnd::CodeBlock) => {
                todo!()
            }
            (Tag::HtmlBlock, TagEnd::HtmlBlock) => todo!(),
            (Tag::List(_), TagEnd::List(_)) => {
                self.tag_is_block();
            }
            (Tag::Item, TagEnd::Item) => {
                self.separate_by_newline = NewlineStrategy::Once;
            }
            (Tag::FootnoteDefinition(_), TagEnd::FootnoteDefinition) => {
                self.separate_by_newline = NewlineStrategy::Once;
            }
            (Tag::Table(_), TagEnd::Table) => {
                let table_ctx = std::mem::take(&mut self.table_context).unwrap();
                let max_column_len = table_ctx.max_column_length();
                let headers = table_ctx.header.into_iter();
                self.print_table_row(headers, &max_column_len, &table_ctx.buffer);
                for max in &max_column_len {
                    self.print_str("| ");
                    self.print_string("-".repeat(*max));
                    self.print_str(" ");
                }
                self.print_str("|\n");
                for row in table_ctx.rows {
                    self.print_table_row(row.into_iter(), &max_column_len, &table_ctx.buffer);
                }
                self.tag_is_block();
            }
            (Tag::TableHead, TagEnd::TableHead) => {}
            (Tag::TableRow, TagEnd::TableRow) => {}
            (Tag::TableCell, TagEnd::TableCell) => {
                let mut range = start.range.clone();
                range.end = self.buffer_len();
                let in_head = self.in_tag(|t| matches!(t, Tag::TableHead));
                let ctx = self.table_context.as_mut().unwrap();
                if in_head {
                    ctx.header.push(range.clone());
                } else {
                    ctx.rows.last_mut().unwrap().push(range.clone());
                }
            }
            (Tag::Emphasis, TagEnd::Emphasis) => {
                self.print_str("_");
            }
            (Tag::Strong, TagEnd::Strong) => self.print_str("**"),
            (Tag::Strikethrough, TagEnd::Strikethrough) => self.print_str("~"),
            (
                Tag::Link {
                    link_type: _,
                    dest_url,
                    title,
                    id: _,
                },
                TagEnd::Link,
            ) => {
                let title = if title.trim().is_empty() {
                    String::new()
                } else {
                    format!(" \"{}\"", title)
                };
                self.print_string(format!("]({dest_url}{title})"));
            }
            (
                Tag::Image {
                    link_type: _,
                    dest_url,
                    title,
                    id: _,
                },
                TagEnd::Image,
            ) => {
                let title = if title.trim().is_empty() {
                    String::new()
                } else {
                    format!(" \"{}\"", title)
                };
                self.print_string(format!("]({dest_url}{title})"));
            }
            (Tag::MetadataBlock(style), TagEnd::MetadataBlock(_)) => {
                match style {
                    MetadataBlockKind::YamlStyle => self.print_str("\n---"),
                    MetadataBlockKind::PlusesStyle => self.print_str("\n+++"),
                }
                self.tag_is_block();
            }
            (start, end) => panic!("Mismatched tags: {:?} and {:?}", start, end),
        }
    }

    fn print_table_row(
        &mut self,
        mut row: impl Iterator<Item = Range<usize>>,
        max_column_len: &[usize],
        buffer: &str,
    ) {
        for max in max_column_len {
            match row.next() {
                Some(header) => {
                    let source = &buffer[header.clone()];
                    let len = header.end - header.start;
                    let diff = if *max > len { max - len } else { 0 };
                    self.print_str("| ");
                    self.print_str(source);
                    self.print_string(" ".repeat(diff + 1));
                }
                None => {
                    let diff = *max;
                    self.print_str("| ");
                    self.print_string(" ".repeat(diff + 1));
                }
            }
        }
        self.print_str("|\n");
    }
}
