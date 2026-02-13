use std::collections::BTreeSet;

use thiserror::Error;

use crate::html::{print_html, print_text, print_xhtml, scan_html, HtmlEntity};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentType(String);

impl ContentType {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum ContentTypeError {
    #[error("unknown content type: {0}")]
    UnknownContentType(String),
    #[error("failed to parse input")]
    ParseFailed,
}

pub fn content_type_from_text(text: &str) -> Option<ContentType> {
    let lowered = text.trim().to_ascii_lowercase();
    if content_types()
        .iter()
        .any(|ct| ct.as_str().eq_ignore_ascii_case(&lowered))
    {
        Some(ContentType(lowered))
    } else {
        None
    }
}

pub fn content_type_text(ct: &ContentType) -> &str {
    ct.as_str()
}

pub fn content_types() -> BTreeSet<ContentType> {
    BTreeSet::from([
        ContentType("text/html".to_string()),
        ContentType("application/xhtml+xml".to_string()),
        ContentType("text/plain".to_string()),
        ContentType("text/markdown".to_string()),
    ])
}

pub fn as_html_transformer<F>(transformer: F, html_text: &str) -> Result<String, ContentTypeError>
where
    F: Fn(Vec<HtmlEntity>) -> Vec<HtmlEntity>,
{
    let input = scan_html(html_text).map_err(|_| ContentTypeError::ParseFailed)?;
    Ok(print_html(&transformer(input)))
}

pub fn as_xhtml_transformer<F>(
    transformer: F,
    xhtml_text: &str,
) -> Result<String, ContentTypeError>
where
    F: Fn(Vec<HtmlEntity>) -> Vec<HtmlEntity>,
{
    let input = scan_html(xhtml_text).map_err(|_| ContentTypeError::ParseFailed)?;
    Ok(print_xhtml(&transformer(input)))
}

pub fn as_plain_text_transformer<F>(
    transformer: F,
    text: &str,
) -> Result<String, ContentTypeError>
where
    F: Fn(Vec<HtmlEntity>) -> Vec<HtmlEntity>,
{
    let escaped = html_escape::encode_text(text).to_string();
    let entities = vec![HtmlEntity::Text {
        tag_stack: crate::html::HtmlTagStack::empty(),
        raw_text: escaped,
    }];
    Ok(print_text(&transformer(entities)))
}

pub fn as_common_mark_transformer<F>(
    transformer: F,
    input: &str,
) -> Result<String, ContentTypeError>
where
    F: Fn(Vec<HtmlEntity>) -> Vec<HtmlEntity>,
{
    let mut html = String::new();
    let parser = pulldown_cmark::Parser::new(input);
    pulldown_cmark::html::push_html(&mut html, parser);
    let entities = scan_html(&html).map_err(|_| ContentTypeError::ParseFailed)?;
    let output = transformer(entities);
    Ok(print_html(&output))
}

pub fn transform_with_content_type<F>(
    content_type: &ContentType,
    transformer: F,
    input: &str,
) -> Result<String, ContentTypeError>
where
    F: Fn(Vec<HtmlEntity>) -> Vec<HtmlEntity>,
{
    match content_type.as_str() {
        "text/html" => as_html_transformer(transformer, input),
        "application/xhtml+xml" => as_xhtml_transformer(transformer, input),
        "text/plain" => as_plain_text_transformer(transformer, input),
        "text/markdown" => as_common_mark_transformer(transformer, input),
        other => Err(ContentTypeError::UnknownContentType(other.to_string())),
    }
}

impl From<&str> for ContentType {
    fn from(value: &str) -> Self {
        ContentType(value.trim().to_ascii_lowercase())
    }
}
