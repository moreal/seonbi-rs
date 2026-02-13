use std::collections::BTreeSet;

use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, LinkType, Parser, Tag, TagEnd};
use pulldown_cmark_to_cmark::cmark;
use thiserror::Error;

use crate::html::{
    HtmlEntity, HtmlTag, HtmlTagKind, HtmlTagStack, heading_level, html_tag_kind, html_tag_name,
    normalize_text, print_html, print_text, print_xhtml, scan_html,
};

const MD_KIND_ATTR: &str = "data-seonbi-cmark";
const MD_INFO_ATTR: &str = "data-seonbi-cmark-info";
const MD_LIST_START_ATTR: &str = "data-seonbi-cmark-list-start";

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

pub fn as_xhtml_transformer<F>(transformer: F, xhtml_text: &str) -> Result<String, ContentTypeError>
where
    F: Fn(Vec<HtmlEntity>) -> Vec<HtmlEntity>,
{
    let input = scan_html(xhtml_text).map_err(|_| ContentTypeError::ParseFailed)?;
    Ok(print_xhtml(&transformer(input)))
}

pub fn as_plain_text_transformer<F>(transformer: F, text: &str) -> Result<String, ContentTypeError>
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
    let input_entities = markdown_to_entities(input)?;
    let output_entities = transformer(normalize_text(input_entities));
    entities_to_markdown(&output_entities)
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

fn markdown_to_entities(input: &str) -> Result<Vec<HtmlEntity>, ContentTypeError> {
    let mut entities = Vec::new();
    let mut stack = HtmlTagStack::empty();
    let mut skipped_blockquote_paragraphs = 0usize;

    for event in Parser::new(input) {
        match event {
            Event::Start(Tag::Paragraph) if stack.last() == Some(HtmlTag::BlockQuote) => {
                skipped_blockquote_paragraphs += 1;
            }
            Event::End(TagEnd::Paragraph) if skipped_blockquote_paragraphs > 0 => {
                skipped_blockquote_paragraphs -= 1;
            }
            Event::Start(tag) => {
                if let Some((html_tag, attrs)) = markdown_start_tag_to_html(tag) {
                    push_start_tag(&mut entities, &mut stack, html_tag, attrs);
                }
            }
            Event::End(tag_end) => {
                if let Some(html_tag) = markdown_end_tag_to_html(tag_end) {
                    push_end_tag(&mut entities, &mut stack, html_tag);
                }
            }
            Event::Text(text) => entities.push(HtmlEntity::Cdata {
                tag_stack: stack.clone(),
                text: text.to_string(),
            }),
            Event::Code(code) => {
                let attrs = md_attrs("code-span");
                push_start_tag(&mut entities, &mut stack, HtmlTag::Code, attrs);
                entities.push(HtmlEntity::Cdata {
                    tag_stack: stack.clone(),
                    text: code.to_string(),
                });
                push_end_tag(&mut entities, &mut stack, HtmlTag::Code);
            }
            Event::Html(html) | Event::InlineHtml(html) => {
                if let Ok(parsed) = scan_html(&html) {
                    let rebased = rebase_entities(&stack, &parsed);
                    entities.extend(rebased);
                    apply_stack_delta(&mut stack, &parsed);
                } else {
                    entities.push(HtmlEntity::Cdata {
                        tag_stack: stack.clone(),
                        text: html.to_string(),
                    });
                }
            }
            Event::SoftBreak => {
                push_void_tag(&mut entities, &stack, HtmlTag::BR, md_attrs("softbreak"))
            }
            Event::HardBreak => {
                push_void_tag(&mut entities, &stack, HtmlTag::BR, md_attrs("hardbreak"))
            }
            Event::Rule => push_void_tag(&mut entities, &stack, HtmlTag::HR, md_attrs("rule")),
            Event::InlineMath(text) | Event::DisplayMath(text) => {
                entities.push(HtmlEntity::Cdata {
                    tag_stack: stack.clone(),
                    text: text.to_string(),
                });
            }
            Event::FootnoteReference(label) => entities.push(HtmlEntity::Cdata {
                tag_stack: stack.clone(),
                text: format!("[^{label}]"),
            }),
            Event::TaskListMarker(checked) => entities.push(HtmlEntity::Cdata {
                tag_stack: stack.clone(),
                text: if checked {
                    "[x] ".to_string()
                } else {
                    "[ ] ".to_string()
                },
            }),
        }
    }

    Ok(entities)
}

fn entities_to_markdown(entities: &[HtmlEntity]) -> Result<String, ContentTypeError> {
    let mut events = Vec::<Event<'static>>::new();
    let mut open = Vec::<OpenTag>::new();

    for entity in entities {
        match entity {
            HtmlEntity::Text { raw_text, .. } => {
                let decoded = html_escape::decode_html_entities(raw_text).to_string();
                events.push(Event::Text(CowStr::from(decoded)));
            }
            HtmlEntity::Cdata { text, .. } => events.push(Event::Text(CowStr::from(text.clone()))),
            HtmlEntity::Comment { comment, .. } => {
                events.push(Event::Html(CowStr::from(format!("<!--{comment}-->"))));
            }
            HtmlEntity::StartTag {
                tag,
                raw_attributes,
                ..
            } => {
                if let Some(kind) = get_attr(raw_attributes, MD_KIND_ATTR) {
                    match kind.as_str() {
                        "softbreak" => {
                            events.push(Event::SoftBreak);
                            open.push(OpenTag::Ignore(*tag));
                            continue;
                        }
                        "hardbreak" => {
                            events.push(Event::HardBreak);
                            open.push(OpenTag::Ignore(*tag));
                            continue;
                        }
                        "rule" => {
                            events.push(Event::Rule);
                            open.push(OpenTag::Ignore(*tag));
                            continue;
                        }
                        _ => {}
                    }

                    if let Some((start, end)) =
                        html_to_markdown_start_tag(*tag, raw_attributes, &kind)
                    {
                        events.push(Event::Start(start));
                        open.push(OpenTag::Markdown(end));
                        continue;
                    }
                }

                let start_html = format!("<{}{raw_attributes}>", html_tag_name(*tag));
                events.push(Event::Html(CowStr::from(start_html)));
                if html_tag_kind(*tag) == HtmlTagKind::Void {
                    open.push(OpenTag::Ignore(*tag));
                } else {
                    open.push(OpenTag::Raw(*tag));
                }
            }
            HtmlEntity::EndTag { tag, .. } => {
                let Some(state) = open.pop() else {
                    continue;
                };

                match state {
                    OpenTag::Markdown(end) => {
                        events.push(Event::End(end));
                    }
                    OpenTag::Raw(open_tag) => {
                        let close_tag = html_tag_name(open_tag);
                        events.push(Event::Html(CowStr::from(format!("</{close_tag}>"))));
                    }
                    OpenTag::Ignore(open_tag) => {
                        if open_tag != *tag {
                            let _ = tag;
                        }
                    }
                }
            }
        }
    }

    let mut markdown = String::new();
    cmark(events.iter().cloned(), &mut markdown).map_err(|_| ContentTypeError::ParseFailed)?;
    markdown = markdown.replace("&#32;", " ");
    markdown = markdown.replace("\n > ", "\n> ");
    markdown = markdown.replace("\n> \n> ", "\n> ");
    if !markdown.ends_with('\n') {
        markdown.push('\n');
    }
    Ok(markdown)
}

fn markdown_start_tag_to_html(tag: Tag<'_>) -> Option<(HtmlTag, String)> {
    match tag {
        Tag::Paragraph => Some((HtmlTag::P, md_attrs("paragraph"))),
        Tag::Heading { level, .. } => {
            Some((heading_tag_from_markdown(level)?, md_attrs("heading")))
        }
        Tag::BlockQuote(_) => Some((HtmlTag::BlockQuote, md_attrs("blockquote"))),
        Tag::CodeBlock(kind) => {
            let attrs = match kind {
                CodeBlockKind::Fenced(info) => {
                    let mut attrs = md_attrs("code-block");
                    push_attr(&mut attrs, MD_INFO_ATTR, &info);
                    attrs
                }
                CodeBlockKind::Indented => md_attrs("code-block"),
            };
            Some((HtmlTag::Pre, attrs))
        }
        Tag::List(start) => {
            if let Some(start) = start {
                let mut attrs = md_attrs("list-ordered");
                push_attr(&mut attrs, MD_LIST_START_ATTR, &start.to_string());
                Some((HtmlTag::OL, attrs))
            } else {
                Some((HtmlTag::UL, md_attrs("list-unordered")))
            }
        }
        Tag::Item => Some((HtmlTag::LI, md_attrs("item"))),
        Tag::Emphasis => Some((HtmlTag::Em, md_attrs("emphasis"))),
        Tag::Strong => Some((HtmlTag::Strong, md_attrs("strong"))),
        Tag::Link {
            dest_url, title, ..
        } => {
            let mut attrs = md_attrs("link");
            push_attr(&mut attrs, "href", &dest_url);
            push_attr(&mut attrs, "title", &title);
            Some((HtmlTag::A, attrs))
        }
        Tag::Image {
            dest_url, title, ..
        } => {
            let mut attrs = md_attrs("image");
            push_attr(&mut attrs, "src", &dest_url);
            push_attr(&mut attrs, "title", &title);
            Some((HtmlTag::Img, attrs))
        }
        _ => None,
    }
}

fn markdown_end_tag_to_html(tag: TagEnd) -> Option<HtmlTag> {
    match tag {
        TagEnd::Paragraph => Some(HtmlTag::P),
        TagEnd::Heading(level) => heading_tag_from_markdown(level),
        TagEnd::BlockQuote(_) => Some(HtmlTag::BlockQuote),
        TagEnd::CodeBlock => Some(HtmlTag::Pre),
        TagEnd::List(ordered) => Some(if ordered { HtmlTag::OL } else { HtmlTag::UL }),
        TagEnd::Item => Some(HtmlTag::LI),
        TagEnd::Emphasis => Some(HtmlTag::Em),
        TagEnd::Strong => Some(HtmlTag::Strong),
        TagEnd::Link => Some(HtmlTag::A),
        TagEnd::Image => Some(HtmlTag::Img),
        _ => None,
    }
}

fn html_to_markdown_start_tag(
    tag: HtmlTag,
    raw_attributes: &str,
    kind: &str,
) -> Option<(Tag<'static>, TagEnd)> {
    match kind {
        "paragraph" => Some((Tag::Paragraph, TagEnd::Paragraph)),
        "heading" => {
            let level = heading_level(tag)?;
            let level = heading_level_to_markdown(level)?;
            Some((
                Tag::Heading {
                    level,
                    id: None,
                    classes: Vec::new(),
                    attrs: Vec::new(),
                },
                TagEnd::Heading(level),
            ))
        }
        "blockquote" => Some((Tag::BlockQuote(None), TagEnd::BlockQuote(None))),
        "code-block" => {
            let info = get_attr(raw_attributes, MD_INFO_ATTR).unwrap_or_default();
            Some((
                Tag::CodeBlock(CodeBlockKind::Fenced(CowStr::from(info))),
                TagEnd::CodeBlock,
            ))
        }
        "list-unordered" => Some((Tag::List(None), TagEnd::List(false))),
        "list-ordered" => {
            let start = get_attr(raw_attributes, MD_LIST_START_ATTR)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(1);
            Some((Tag::List(Some(start)), TagEnd::List(true)))
        }
        "item" => Some((Tag::Item, TagEnd::Item)),
        "emphasis" => Some((Tag::Emphasis, TagEnd::Emphasis)),
        "strong" => Some((Tag::Strong, TagEnd::Strong)),
        "link" => {
            let dest_url = get_attr(raw_attributes, "href").unwrap_or_default();
            let title = get_attr(raw_attributes, "title").unwrap_or_default();
            Some((
                Tag::Link {
                    link_type: LinkType::Inline,
                    dest_url: CowStr::from(dest_url),
                    title: CowStr::from(title),
                    id: CowStr::from(String::new()),
                },
                TagEnd::Link,
            ))
        }
        _ => {
            let _ = tag;
            None
        }
    }
}

fn heading_tag_from_markdown(level: HeadingLevel) -> Option<HtmlTag> {
    match level {
        HeadingLevel::H1 => Some(HtmlTag::H1),
        HeadingLevel::H2 => Some(HtmlTag::H2),
        HeadingLevel::H3 => Some(HtmlTag::H3),
        HeadingLevel::H4 => Some(HtmlTag::H4),
        HeadingLevel::H5 => Some(HtmlTag::H5),
        HeadingLevel::H6 => Some(HtmlTag::H6),
    }
}

fn heading_level_to_markdown(level: u8) -> Option<HeadingLevel> {
    match level {
        1 => Some(HeadingLevel::H1),
        2 => Some(HeadingLevel::H2),
        3 => Some(HeadingLevel::H3),
        4 => Some(HeadingLevel::H4),
        5 => Some(HeadingLevel::H5),
        6 => Some(HeadingLevel::H6),
        _ => None,
    }
}

fn md_attrs(kind: &str) -> String {
    format!(" {MD_KIND_ATTR}=\"{}\"", escape_attr(kind))
}

fn push_attr(attrs: &mut String, name: &str, value: &str) {
    attrs.push(' ');
    attrs.push_str(name);
    attrs.push_str("=\"");
    attrs.push_str(&escape_attr(value));
    attrs.push('"');
}

fn escape_attr(value: &str) -> String {
    html_escape::encode_double_quoted_attribute(value).to_string()
}

fn get_attr(raw_attrs: &str, name: &str) -> Option<String> {
    let pattern = format!(r#"{name}=""#);
    let start = raw_attrs.find(&pattern)? + pattern.len();
    let rest = &raw_attrs[start..];
    let end = rest.find('"')?;
    Some(html_escape::decode_html_entities(&rest[..end]).to_string())
}

fn push_start_tag(
    out: &mut Vec<HtmlEntity>,
    stack: &mut HtmlTagStack,
    tag: HtmlTag,
    raw_attributes: String,
) {
    let current = stack.clone();
    out.push(HtmlEntity::StartTag {
        tag_stack: current.clone(),
        tag,
        raw_attributes,
    });
    if html_tag_kind(tag) == HtmlTagKind::Void {
        out.push(HtmlEntity::EndTag {
            tag_stack: current,
            tag,
        });
    } else {
        *stack = stack.push(tag);
    }
}

fn push_end_tag(out: &mut Vec<HtmlEntity>, stack: &mut HtmlTagStack, tag: HtmlTag) {
    *stack = stack.pop(tag);
    out.push(HtmlEntity::EndTag {
        tag_stack: stack.clone(),
        tag,
    });
}

fn push_void_tag(
    out: &mut Vec<HtmlEntity>,
    stack: &HtmlTagStack,
    tag: HtmlTag,
    raw_attributes: String,
) {
    out.push(HtmlEntity::StartTag {
        tag_stack: stack.clone(),
        tag,
        raw_attributes,
    });
    out.push(HtmlEntity::EndTag {
        tag_stack: stack.clone(),
        tag,
    });
}

fn rebase_entities(base: &HtmlTagStack, parsed: &[HtmlEntity]) -> Vec<HtmlEntity> {
    parsed
        .iter()
        .cloned()
        .map(|entity| {
            let rebased = entity.tag_stack().rebase(&HtmlTagStack::empty(), base);
            entity.with_tag_stack(rebased)
        })
        .collect()
}

fn apply_stack_delta(stack: &mut HtmlTagStack, entities: &[HtmlEntity]) {
    for entity in entities {
        match entity {
            HtmlEntity::StartTag { tag, .. } => {
                if html_tag_kind(*tag) != HtmlTagKind::Void {
                    *stack = stack.push(*tag);
                }
            }
            HtmlEntity::EndTag { tag, .. } => {
                *stack = stack.pop(*tag);
            }
            _ => {}
        }
    }
}

enum OpenTag {
    Markdown(TagEnd),
    Raw(HtmlTag),
    Ignore(HtmlTag),
}
