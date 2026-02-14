use super::{
    entity::HtmlEntity, tag::HtmlTag, tag::HtmlTagKind, tag::html_tag_kind, tag_stack::HtmlTagStack,
};

pub fn is_preserved_tag(tag: HtmlTag) -> bool {
    match tag {
        HtmlTag::Code | HtmlTag::Kbd | HtmlTag::Pre | HtmlTag::TextArea => true,
        _ => !matches!(html_tag_kind(tag), HtmlTagKind::Normal | HtmlTagKind::EscapableRawText),
    }
}

pub fn is_preserved_tag_stack(stack: &HtmlTagStack) -> bool {
    stack.any(is_preserved_tag)
}

pub fn is_preserved_entity(entity: &HtmlEntity) -> bool {
    match entity {
        HtmlEntity::Comment { .. } => true,
        HtmlEntity::StartTag { tag_stack, tag, .. } | HtmlEntity::EndTag { tag_stack, tag } => {
            is_preserved_tag(*tag) || is_preserved_tag_stack(tag_stack)
        }
        _ => is_preserved_tag_stack(entity.tag_stack()),
    }
}
