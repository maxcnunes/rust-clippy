use clippy_utils::diagnostics::span_lint;
use rustc_ast::{AttrKind, AttrStyle, Attribute};
use rustc_lint::LateContext;
use rustc_span::{BytePos, Span};

use super::DOC_BROKEN_LINK;

pub fn check(cx: &LateContext<'_>, attrs: &[Attribute]) {
    for broken_link in BrokenLinkLoader::collect_spans_broken_link(attrs) {
        let reason_msg = match broken_link.reason {
            BrokenLinkReason::MultipleLines => "broken across multiple lines",
            BrokenLinkReason::MissingCloseParenthesis => "missing close parenthesis",
            BrokenLinkReason::WhiteSpace => "whitespace within url",
        };

        span_lint(
            cx,
            DOC_BROKEN_LINK,
            broken_link.span,
            format!("possible broken doc link: {reason_msg}"),
        );
    }
}

/// The reason why a link is considered broken.
enum BrokenLinkReason {
    MultipleLines,
    MissingCloseParenthesis,
    WhiteSpace,
}

/// Broken link data.
struct BrokenLink {
    reason: BrokenLinkReason,
    span: Span,
}

/// Scan AST attributes looking up in doc comments for broken links
/// which rustdoc won't be able to properly create link tags later.
struct BrokenLinkLoader {
    /// List of spans for detected broken links.
    broken_links: Vec<BrokenLink>,

    /// Mark it has detected a link and it is processing whether
    /// or not it is broken.
    active: bool,

    /// Keep track of the span for the processing broken link.
    active_span: Option<Span>,

    /// Keep track where exactly the link definition has started in the code.
    active_pos_start: u32,

    /// Mark it is processing the link text tag.
    processing_link_text: bool,

    /// Mark it is processing the link url tag.
    processing_link_url: bool,

    /// Mark it is reading the url tag content. It will be false if the loader
    /// got to the url tag processing, but all the chars read so far were just
    /// whitespaces.
    reading_link_url: bool,

    /// Mark the url url isn't empty, but it still being processed in a new line.
    reading_link_url_new_line: bool,

    /// Mark the current link url is broken across multiple lines.
    url_multiple_lines: bool,

    /// Mark the link's span start position.
    start: u32,
}

impl BrokenLinkLoader {
    /// Return spans of broken links.
    fn collect_spans_broken_link(attrs: &[Attribute]) -> Vec<BrokenLink> {
        let mut loader = BrokenLinkLoader {
            broken_links: vec![],
            active: false,
            active_pos_start: 0,
            active_span: None,
            processing_link_text: false,
            processing_link_url: false,
            reading_link_url: false,
            reading_link_url_new_line: false,
            url_multiple_lines: false,
            start: 0_u32,
        };
        loader.scan_attrs(attrs);
        loader.broken_links
    }

    fn scan_attrs(&mut self, attrs: &[Attribute]) {
        for idx in 0..attrs.len() {
            let attr = &attrs[idx];
            if let AttrKind::DocComment(_com_kind, sym) = attr.kind
                && let AttrStyle::Outer = attr.style
            {
                self.scan_line(sym.as_str(), attr.span);
            } else {
                if idx > 0 && self.active && self.processing_link_url {
                    let prev_attr = &attrs[idx - 1];
                    let prev_end_line = prev_attr.span.hi().0;
                    self.record_broken_link(prev_end_line, BrokenLinkReason::MissingCloseParenthesis);
                }
                self.reset_lookup();
            }
        }

        if self.active && self.processing_link_url {
            let last_end_line = attrs.last().unwrap().span.hi().0;
            self.record_broken_link(last_end_line, BrokenLinkReason::MissingCloseParenthesis);
            self.reset_lookup();
        }
    }

    fn scan_line(&mut self, line: &str, attr_span: Span) {
        // Note that we specifically need the char _byte_ indices here, not the positional indexes
        // within the char array to deal with multi-byte characters properly. `char_indices` does
        // exactly that. It provides an iterator over tuples of the form `(byte position, char)`.
        let char_indices: Vec<_> = line.char_indices().collect();

        self.reading_link_url_new_line = self.reading_link_url;

        for (pos, c) in char_indices {
            if pos == 0 && c.is_whitespace() {
                // ignore prefix whitespace on comments
                continue;
            }

            if !self.active {
                if c == '[' {
                    self.processing_link_text = true;
                    self.active = true;
                    // +3 skips the opening delimiter
                    self.active_pos_start = attr_span.lo().0 + u32::try_from(pos).unwrap() + 3;
                    self.active_span = Some(attr_span);
                }
                continue;
            }

            if self.processing_link_text {
                if c == ']' {
                    self.processing_link_text = false;
                }
                continue;
            }

            if !self.processing_link_url {
                if c == '(' {
                    self.processing_link_url = true;
                } else {
                    // not a real link, start lookup over again
                    self.reset_lookup();
                }
                continue;
            }

            if c == ')' {
                // record full broken link tag
                if self.url_multiple_lines {
                    // +3 skips the opening delimiter and +1 to include the closing parethesis
                    let pos_end = attr_span.lo().0 + u32::try_from(pos).unwrap() + 4;
                    self.record_broken_link(pos_end, BrokenLinkReason::MultipleLines);
                    self.reset_lookup();
                }
                self.reset_lookup();
                continue;
            }

            if self.reading_link_url && c.is_whitespace() {
                let pos_end = u32::try_from(pos).unwrap();
                self.record_broken_link(pos_end, BrokenLinkReason::WhiteSpace);
                self.reset_lookup();
                continue;
            }

            if !c.is_whitespace() {
                self.reading_link_url = true;
            }

            if self.reading_link_url_new_line {
                self.url_multiple_lines = true;
            }
        }
    }

    fn reset_lookup(&mut self) {
        self.active = false;
        self.start = 0;
        self.processing_link_text = false;
        self.processing_link_url = false;
        self.reading_link_url = false;
        self.reading_link_url_new_line = false;
        self.url_multiple_lines = false;
    }

    fn record_broken_link(&mut self, pos_end: u32, reason: BrokenLinkReason) {
        if let Some(attr_span) = self.active_span {
            let start = BytePos(self.active_pos_start);
            let end = BytePos(pos_end);

            let span = Span::new(start, end, attr_span.ctxt(), attr_span.parent());

            self.broken_links.push(BrokenLink { reason, span });
        }
    }
}
