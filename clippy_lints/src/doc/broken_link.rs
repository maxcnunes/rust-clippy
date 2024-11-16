use super::{DOC_BROKEN_LINK, Fragments};
use clippy_utils::diagnostics::span_lint;
use rustc_lint::LateContext;
use std::ops::Range;

// Check broken links in code docs.
pub fn check(cx: &LateContext<'_>, _trimmed_text: &str, range: Range<usize>, fragments: Fragments<'_>, link: &str) {
    if let Some(span) = fragments.span(cx, range) {
        // Broken links are replaced with "fake" value by `fake_broken_link_callback` at `doc/mod.rs`.
        if link == "fake" {
            span_lint(cx, DOC_BROKEN_LINK, span, "possible broken doc link");
        }
    }
}
