use super::{entity::HtmlEntity, tag::HtmlTag, tag_stack::HtmlTagStack};

pub fn wrap(
    base_stack: &HtmlTagStack,
    tag: HtmlTag,
    attrs: &str,
    entities: Vec<HtmlEntity>,
) -> Vec<HtmlEntity> {
    let mut out = Vec::with_capacity(entities.len() + 2);
    out.push(HtmlEntity::StartTag {
        tag_stack: base_stack.clone(),
        tag,
        raw_attributes: attrs.to_string(),
    });

    let new_base = base_stack.push(tag);
    for entity in entities {
        let rebased = entity
            .tag_stack()
            .rebase(base_stack, &new_base);
        out.push(entity.with_tag_stack(rebased));
    }

    out.push(HtmlEntity::EndTag {
        tag_stack: base_stack.clone(),
        tag,
    });
    out
}

pub fn is_wrapped_by(entities: &[HtmlEntity], tag: HtmlTag) -> bool {
    is_wrapped_by_exact(entities, tag, None)
}

pub fn is_wrapped_by_exact(entities: &[HtmlEntity], tag: HtmlTag, attrs: Option<&str>) -> bool {
    if entities.is_empty() {
        return false;
    }

    match (&entities[0], &entities[entities.len() - 1]) {
        (
            HtmlEntity::StartTag {
                tag_stack: start_stack,
                tag: start_tag,
                raw_attributes,
            },
            HtmlEntity::EndTag {
                tag_stack: end_stack,
                tag: end_tag,
            },
        ) => {
            *start_tag == tag
                && *end_tag == tag
                && start_stack == end_stack
                && attrs.is_none_or(|a| raw_attributes == a)
        }
        _ => false,
    }
}
