use clippy_utils::diagnostics::span_lint;
use rustc_ast::{AttrKind, AttrStyle, Attribute};
use rustc_lint::LateContext;
use rustc_span::{BytePos, Span};

use super::DOC_BROKEN_LINK;

pub fn check(cx: &LateContext<'_>, attrs: &[Attribute]) {
    for span in BrokenLinkLoader::collect_spans_broken_link(attrs) {
        span_lint(cx, DOC_BROKEN_LINK, span, "possible broken doc link");
    }
}

struct BrokenLinkLoader {
    spans_broken_link: Vec<Span>,
    active: bool,
    processing_link_text: bool,
    processing_link_url: bool,
    start: u32,
}

impl BrokenLinkLoader {
    fn collect_spans_broken_link(attrs: &[Attribute]) -> Vec<Span> {
        let mut loader = BrokenLinkLoader {
            spans_broken_link: vec![],
            active: false,
            processing_link_text: false,
            processing_link_url: false,
            start: 0_u32,
        };
        loader.scan_attrs(attrs);
        loader.spans_broken_link
    }

    fn scan_attrs(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            if let AttrKind::DocComment(_com_kind, sym) = attr.kind
                && let AttrStyle::Outer = attr.style
            {
                self.scan_line(sym.as_str(), attr.span);
            }
        }
    }

    fn scan_line(&mut self, the_str: &str, attr_span: Span) {
        // Note that we specifically need the char _byte_ indices here, not the positional indexes
        // within the char array to deal with multi-byte characters properly. `char_indices` does
        // exactly that. It provides an iterator over tuples of the form `(byte position, char)`.
        let char_indices: Vec<_> = the_str.char_indices().collect();

        let mut no_url_curr_line = true;

        for (pos, c) in char_indices {
            if !self.active {
                if c == '[' {
                    self.processing_link_text = true;
                    self.active = true;
                    self.start = attr_span.lo().0 + u32::try_from(pos).unwrap();
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
                    no_url_curr_line = true;
                }
                continue;
            }

            if c == ')' {
                self.reset_lookup();
                no_url_curr_line = true;
            } else if no_url_curr_line && c != ' ' {
                no_url_curr_line = false;
            }
        }

        // If it got at the end of the line and it still processing a link part,
        // it means this is a broken link.
        if self.active && self.processing_link_url && !no_url_curr_line {
            let pos_end_line = u32::try_from(the_str.len()).unwrap() - 1;

            // +3 skips the opening delimiter
            let start = BytePos(self.start + 3);
            let end = start + BytePos(pos_end_line);

            let com_span = Span::new(start, end, attr_span.ctxt(), attr_span.parent());

            self.spans_broken_link.push(com_span);
        }
    }

    fn reset_lookup(&mut self) {
        self.processing_link_url = false;
        self.active = false;
        self.start = 0;
    }
}
