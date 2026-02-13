use std::str::FromStr;

use thiserror::Error;

use super::{
    entity::HtmlEntity,
    tag::{html_tag_kind, HtmlTag, HtmlTagKind},
    tag_stack::HtmlTagStack,
};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ScanError {
    #[error("invalid html")]
    InvalidHtml,
}

pub fn scan_html(input: &str) -> Result<Vec<HtmlEntity>, ScanError> {
    let mut entities = Vec::new();
    let mut pos = 0usize;
    let mut tag_stack = HtmlTagStack::empty();

    while pos < input.len() {
        let (text, next_pos) = scan_text(input, pos, &tag_stack);
        if !text.is_empty() {
            entities.push(HtmlEntity::Text {
                tag_stack: tag_stack.clone(),
                raw_text: text,
            });
        }
        pos = next_pos;
        if pos >= input.len() {
            break;
        }

        let segment = &input[pos..];
        if let Some((chunk, next_stack, consumed)) = parse_entity(segment, &tag_stack) {
            entities.extend(chunk);
            tag_stack = next_stack;
            pos += consumed;
            continue;
        }

        let c = segment.chars().next().ok_or(ScanError::InvalidHtml)?;
        entities.push(HtmlEntity::Text {
            tag_stack: tag_stack.clone(),
            raw_text: c.to_string(),
        });
        pos += c.len_utf8();
    }

    Ok(entities)
}

fn scan_text(input: &str, start: usize, stack: &HtmlTagStack) -> (String, usize) {
    let mut pos = start;
    let mut out = String::new();

    while pos < input.len() {
        let remain = &input[pos..];
        if !remain.starts_with('<') {
            if let Some(next_lt) = remain.find('<') {
                out.push_str(&remain[..next_lt]);
                pos += next_lt;
            } else {
                out.push_str(remain);
                pos = input.len();
            }
            continue;
        }

        let mut chars = remain.chars();
        let _lt = chars.next();
        let second = chars.next();

        match second {
            Some(c)
                if c != '!'
                    && c != '/'
                    && !c.is_ascii_uppercase()
                    && !c.is_ascii_lowercase() =>
            {
                out.push('<');
                out.push(c);
                pos += 1 + c.len_utf8();
            }
            _ => {
                let _ = stack;
                break;
            }
        }
    }

    (out, pos)
}

fn parse_entity(
    input: &str,
    tag_stack: &HtmlTagStack,
) -> Option<(Vec<HtmlEntity>, HtmlTagStack, usize)> {
    parse_comment(input, tag_stack)
        .or_else(|| parse_cdata(input, tag_stack))
        .or_else(|| parse_start_tag(input, tag_stack))
        .or_else(|| parse_end_tag(input, tag_stack))
}

fn parse_comment(
    input: &str,
    tag_stack: &HtmlTagStack,
) -> Option<(Vec<HtmlEntity>, HtmlTagStack, usize)> {
    let rest = input.strip_prefix("<!--")?;
    let end = rest.find("-->")?;
    let comment = rest[..end].to_string();
    let consumed = 4 + end + 3;
    Some((
        vec![HtmlEntity::Comment {
            tag_stack: tag_stack.clone(),
            comment,
        }],
        tag_stack.clone(),
        consumed,
    ))
}

fn parse_cdata(
    input: &str,
    tag_stack: &HtmlTagStack,
) -> Option<(Vec<HtmlEntity>, HtmlTagStack, usize)> {
    let rest = input.strip_prefix("<![CDATA[")?;
    let end = rest.find("]]>")?;
    let text = rest[..end].to_string();
    let consumed = 9 + end + 3;
    Some((
        vec![HtmlEntity::Cdata {
            tag_stack: tag_stack.clone(),
            text,
        }],
        tag_stack.clone(),
        consumed,
    ))
}

fn parse_start_tag(
    input: &str,
    tag_stack: &HtmlTagStack,
) -> Option<(Vec<HtmlEntity>, HtmlTagStack, usize)> {
    if !input.starts_with('<') {
        return None;
    }

    let bytes = input.as_bytes();
    let mut i = 1usize;
    if i >= bytes.len() || !bytes[i].is_ascii_alphabetic() {
        return None;
    }

    let name_start = i;
    i += 1;
    while i < bytes.len() && bytes[i].is_ascii_alphanumeric() {
        i += 1;
    }

    let tag_name = &input[name_start..i];
    let tag = HtmlTag::from_str(tag_name).ok()?;

    let mut attrs = String::new();
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '/' || c == '>' {
            break;
        }
        if c == '"' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] as char != '"' {
                j += 1;
            }
            if j >= bytes.len() {
                return None;
            }
            attrs.push_str(&input[i..=j]);
            i = j + 1;
            continue;
        }
        if c == '\'' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] as char != '\'' {
                j += 1;
            }
            if j >= bytes.len() {
                return None;
            }
            attrs.push_str(&input[i..=j]);
            i = j + 1;
            continue;
        }

        let start = i;
        while i < bytes.len() {
            let c2 = bytes[i] as char;
            if c2 == '"' || c2 == '\'' || c2 == '/' || c2 == '>' {
                break;
            }
            i += 1;
        }
        if i == start {
            return None;
        }
        attrs.push_str(&input[start..i]);
    }

    let mut self_closing = false;
    if i < bytes.len() && bytes[i] as char == '/' {
        self_closing = true;
        i += 1;
    }

    if i >= bytes.len() || bytes[i] as char != '>' {
        return None;
    }
    i += 1;

    let mut entities = vec![HtmlEntity::StartTag {
        tag_stack: tag_stack.clone(),
        tag,
        raw_attributes: attrs,
    }];

    let next_stack = if self_closing || html_tag_kind(tag) == HtmlTagKind::Void {
        entities.push(HtmlEntity::EndTag {
            tag_stack: tag_stack.clone(),
            tag,
        });
        tag_stack.clone()
    } else {
        tag_stack.push(tag)
    };

    Some((entities, next_stack, i))
}

fn parse_end_tag(
    input: &str,
    tag_stack: &HtmlTagStack,
) -> Option<(Vec<HtmlEntity>, HtmlTagStack, usize)> {
    let rest = input.strip_prefix("</")?;
    let bytes = rest.as_bytes();
    let mut i = 0usize;
    if i >= bytes.len() || !bytes[i].is_ascii_alphabetic() {
        return None;
    }
    i += 1;
    while i < bytes.len() && bytes[i].is_ascii_alphanumeric() {
        i += 1;
    }
    let tag_name = &rest[..i];
    let tag = HtmlTag::from_str(tag_name).ok()?;
    if i >= bytes.len() || bytes[i] as char != '>' {
        return None;
    }
    i += 1;

    if html_tag_kind(tag) == HtmlTagKind::Void {
        return Some((Vec::new(), tag_stack.clone(), i + 2));
    }

    let next_stack = tag_stack.pop(tag);
    Some((
        vec![HtmlEntity::EndTag {
            tag_stack: next_stack.clone(),
            tag,
        }],
        next_stack,
        i + 2,
    ))
}
