#![warn(clippy::doc_broken_link)]

fn main() {
    doc_valid_link();
    doc_valid_link_broken_title();
    doc_valid_link_broken_url_tag();
    doc_invalid_link_broken_url_scheme_part();
    doc_invalid_link_broken_url_host_part();
}

/// Test valid link, whole link single line.
/// [doc valid link](https://test.fake/doc_valid_link)
pub fn doc_valid_link() {}

/// Test valid link, title tag broken across multiple lines.
/// [doc invalid link broken
/// title](https://test.fake/doc_valid_link_broken_title)
pub fn doc_valid_link_broken_title() {}

/// Test valid link, url tag broken across multiple lines, but
/// the whole url part in a single line.
/// [doc valid link broken url tag](
/// https://test.fake/doc_valid_link_broken_url_tag)
pub fn doc_valid_link_broken_url_tag() {}

/// Test invalid link, url part broken across multiple lines.
/// [doc invalid link broken url scheme part part](https://
/// test.fake/doc_invalid_link_broken_url_scheme_part)
//~^^ ERROR: possible broken doc link
pub fn doc_invalid_link_broken_url_scheme_part() {}

/// Test invalid link, url part broken across multiple lines.
/// [doc invalid link broken url host part](https://test
/// .fake/doc_invalid_link_broken_url_host_part)
//~^^ ERROR: possible broken doc link
pub fn doc_invalid_link_broken_url_host_part() {}

/// This might be considered a link false positive
/// and should be ignored by this lint rule:
/// Example of referencing some code with brackets [T].
pub fn doc_ignore_link_false_positive_1() {}

/// This might be considered a link false positive
/// and should be ignored by this lint rule:
/// [`T`]. Continue text after brackets,
/// then (something in
/// parenthesis).
pub fn doc_ignore_link_false_positive_2() {}
