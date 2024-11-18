#![warn(clippy::doc_broken_link)]

fn main() {}

pub struct FakeType {}

/// Test valid link, whole link single line.
/// [doc valid link](https://test.fake/doc_valid_link)
pub fn doc_valid_link() {}

/// Test valid link, whole link single line but it has special chars such as brackets and
/// parenthesis. [doc invalid link url invalid char](https://test.fake/doc_valid_link_url_invalid_char?foo[bar]=1&bar(foo)=2)
pub fn doc_valid_link_url_invalid_char() {}

/// Test valid link, text tag broken across multiple lines.
/// [doc invalid link broken
/// text](https://test.fake/doc_valid_link_broken_text)
pub fn doc_valid_link_broken_text() {}

/// Test valid link, url tag broken across multiple lines, but
/// the whole url part in a single line.
/// [doc valid link broken url tag two lines first](https://test.fake/doc_valid_link_broken_url_tag_two_lines_first
/// )
pub fn doc_valid_link_broken_url_tag_two_lines_first() {}

/// Test valid link, url tag broken across multiple lines, but
/// the whole url part in a single line.
/// [doc valid link broken url tag two lines second](
/// https://test.fake/doc_valid_link_broken_url_tag_two_lines_second)
pub fn doc_valid_link_broken_url_tag_two_lines_second() {}

/// Test valid link, url tag broken across multiple lines, but
/// the whole url part in a single line, but the closing pharentesis
/// in a third line.
/// [doc valid link broken url tag three lines](
/// https://test.fake/doc_valid_link_broken_url_tag_three_lines
/// )
pub fn doc_valid_link_broken_url_tag_three_lines() {}

/// Test invalid link, url part broken across multiple lines.
/// [doc invalid link broken url scheme part part](https://
/// test.fake/doc_invalid_link_broken_url_scheme_part)
//~^^ ERROR: possible broken doc link: broken across multiple lines
pub fn doc_invalid_link_broken_url_scheme_part() {}

/// Test invalid link, url part broken across multiple lines.
/// [doc invalid link broken url host part](https://test
/// .fake/doc_invalid_link_broken_url_host_part)
//~^^ ERROR: possible broken doc link: broken across multiple lines
pub fn doc_invalid_link_broken_url_host_part() {}

// NOTE: We don't test doc links where the url is broken accross
// multiple lines in the path part because that is something
// rustdoc itself will check and warn about it.
pub fn doc_invalid_link_broken_url_path_part() {}

/// Test invalid link, url missing close parenthesis.
/// [doc invalid link broken url missing close parenthesis](
/// https://test.fake/doc_invalid_link_missing_close_parenthesis
//~^^ ERROR: possible broken doc link: missing close parenthesis
pub fn doc_invalid_link_missing_close_parenthesis() {}

/// Test invalid link, url whitespace within url.
/// [doc invalid link broken url whitespace within url](
/// https://test.fake/doc_invalid_link_url whitespace_within_url)
//~^^ ERROR: possible broken doc link: whitespace within url
pub fn doc_invalid_link_url_whitespace_within_url() {}

/// This might be considered a link false positive
/// and should be ignored by this lint rule:
/// Example of referencing some code with brackets [FakeType].
pub fn doc_ignore_link_false_positive_1() {}

/// This might be considered a link false positive
/// and should be ignored by this lint rule:
/// [`FakeType`]. Continue text after brackets,
/// then (something in
/// parenthesis).
pub fn doc_ignore_link_false_positive_2() {}
