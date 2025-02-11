use clippy_utils::diagnostics::span_lint;
use pulldown_cmark::BrokenLink as PullDownBrokenLink;
use rustc_lint::LateContext;
use rustc_resolve::rustdoc::{DocFragment, source_span_for_markdown_range};
use rustc_span::{BytePos, Pos, Span};

use super::DOC_BROKEN_LINK;

/// Scan and report broken link on documents.
/// It ignores false positives detected by pulldown_cmark, and only
/// warns users when the broken link is consider a URL.
pub fn check(cx: &LateContext<'_>, bl: &PullDownBrokenLink<'_>, doc: &String, fragments: &Vec<DocFragment>) {
    warn_if_broken_link(cx, bl, doc, fragments);
}

/// The reason why a link is considered broken.
// NOTE: We don't check these other cases because
// rustdoc itself will check and warn about it:
// - When a link url is broken across multiple lines in the URL path part
// - When a link tag is missing the close parenthesis character at the end.
// - When a link has whitespace within the url link.
enum BrokenLinkReason {
    MultipleLines,
}

#[derive(Debug)]
enum State {
    ProcessingLinkText,
    ProcessedLinkText,
    ProcessingLinkUrl(UrlState),
}

#[derive(Debug)]
enum UrlState {
    Empty,
    FilledEntireSingleLine,
    FilledBrokenMultipleLines,
}

fn warn_if_broken_link(cx: &LateContext<'_>, bl: &PullDownBrokenLink<'_>, doc: &String, fragments: &Vec<DocFragment>) {
    if let Some(span) = source_span_for_markdown_range(cx.tcx, doc, &bl.span, fragments) {
        // `PullDownBrokenLink` provided by pulldown_cmark always
        // start with `[` which makes pulldown_cmark consider this a link tag.
        let mut state = State::ProcessingLinkText;

        // Whether it was detected a line break within the link tag url part.
        let mut reading_link_url_new_line = false;

        // Skip the first char because we already know it is a `[` char.
        for (abs_pos, c) in doc.char_indices().skip(bl.span.start + 1) {
            match &state {
                State::ProcessingLinkText => {
                    if c == ']' {
                        state = State::ProcessedLinkText;
                    }
                },
                State::ProcessedLinkText => {
                    if c == '(' {
                        state = State::ProcessingLinkUrl(UrlState::Empty);
                    } else {
                        // not a real link, just skip it without reporting a broken link for it.
                        break;
                    }
                },
                State::ProcessingLinkUrl(url_state) => {
                    if c == '\n' {
                        reading_link_url_new_line = true;
                        continue;
                    }

                    if c == ')' {
                        // record full broken link tag
                        if let UrlState::FilledBrokenMultipleLines = url_state {
                            let offset = abs_pos - bl.span.start;
                            report_broken_link(cx, span, offset, BrokenLinkReason::MultipleLines);
                        }
                        break;
                    }

                    if !c.is_whitespace() {
                        if reading_link_url_new_line {
                            // It was reading a link url which was entirely in a single line, but a new char
                            // was found in this new line which turned the url into a broken state.
                            state = State::ProcessingLinkUrl(UrlState::FilledBrokenMultipleLines);
                            continue;
                        }

                        state = State::ProcessingLinkUrl(UrlState::FilledEntireSingleLine);
                    }
                },
            };
        }
    }
}

fn report_broken_link(cx: &LateContext<'_>, frag_span: Span, offset: usize, reason: BrokenLinkReason) {
    let start = frag_span.lo();
    let end = start + BytePos::from_usize(offset + 5);

    let span = Span::new(start, end, frag_span.ctxt(), frag_span.parent());

    let reason_msg = match reason {
        BrokenLinkReason::MultipleLines => "broken across multiple lines",
    };

    span_lint(
        cx,
        DOC_BROKEN_LINK,
        span,
        format!("possible broken doc link: {reason_msg}"),
    );
}
