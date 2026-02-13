use html_escape::decode_html_entities;

use super::{entity::HtmlEntity, tag::html_tag_kind, tag::HtmlTagKind};

pub fn print_html(entities: &[HtmlEntity]) -> String {
    print_html_like(false, entities)
}

pub fn print_xhtml(entities: &[HtmlEntity]) -> String {
    print_html_like(true, entities)
}

fn print_html_like(xhtml: bool, entities: &[HtmlEntity]) -> String {
    let mut out = String::new();
    let mut i = 0usize;

    while i < entities.len() {
        if i + 1 < entities.len() && is_void_pair(&entities[i], &entities[i + 1]) {
            if let HtmlEntity::StartTag {
                tag,
                raw_attributes,
                ..
            } = &entities[i]
            {
                out.push('<');
                out.push_str(tag.name());
                push_attrs(&mut out, raw_attributes);
                if xhtml {
                    out.push_str("/>");
                } else {
                    out.push('>');
                }
            }
            i += 2;
            continue;
        }

        render_entity(&mut out, &entities[i]);
        i += 1;
    }

    out
}

fn is_void_pair(a: &HtmlEntity, b: &HtmlEntity) -> bool {
    match (a, b) {
        (
            HtmlEntity::StartTag {
                tag_stack: sa,
                tag: ta,
                ..
            },
            HtmlEntity::EndTag {
                tag_stack: sb,
                tag: tb,
            },
        ) => html_tag_kind(*ta) == HtmlTagKind::Void && sa == sb && ta == tb,
        _ => false,
    }
}

fn render_entity(out: &mut String, entity: &HtmlEntity) {
    match entity {
        HtmlEntity::StartTag {
            tag,
            raw_attributes,
            ..
        } => {
            out.push('<');
            out.push_str(tag.name());
            push_attrs(out, raw_attributes);
            out.push('>');
        }
        HtmlEntity::EndTag { tag, .. } => {
            out.push_str("</");
            out.push_str(tag.name());
            out.push('>');
        }
        HtmlEntity::Text { raw_text, .. } => out.push_str(raw_text),
        HtmlEntity::Cdata { text, .. } => {
            out.push_str("<![CDATA[");
            out.push_str(text);
            out.push_str("]]>");
        }
        HtmlEntity::Comment { comment, .. } => {
            out.push_str("<!--");
            out.push_str(comment);
            out.push_str("-->");
        }
    }
}

fn push_attrs(out: &mut String, attrs: &str) {
    if attrs.is_empty() {
        return;
    }
    if attrs.chars().next().is_some_and(|c| c.is_whitespace()) {
        out.push_str(attrs);
    } else {
        out.push(' ');
        out.push_str(attrs);
    }
}

pub fn print_text(entities: &[HtmlEntity]) -> String {
    let mut out = String::new();
    for entity in entities {
        match entity {
            HtmlEntity::Text { raw_text, .. } => {
                out.push_str(&decode_html_entities(raw_text));
            }
            HtmlEntity::Cdata { text, .. } => out.push_str(text),
            _ => {}
        }
    }
    out
}
