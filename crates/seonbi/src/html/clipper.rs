use super::entity::HtmlEntity;

pub fn clip_text(prefix: &str, suffix: &str, entities: &[HtmlEntity]) -> Option<Vec<HtmlEntity>> {
    let prefixed = clip_prefix_text(prefix, entities)?;
    clip_suffix_text(suffix, &prefixed)
}

pub fn clip_prefix_text(prefix: &str, entities: &[HtmlEntity]) -> Option<Vec<HtmlEntity>> {
    if entities.is_empty() {
        return if prefix.is_empty() {
            Some(Vec::new())
        } else {
            None
        };
    }

    let mut out = Vec::new();
    let mut i = 0usize;
    while i < entities.len() {
        match &entities[i] {
            HtmlEntity::Comment { .. } => {
                out.push(entities[i].clone());
                i += 1;
            }
            HtmlEntity::Text { raw_text, .. } => {
                if prefix == raw_text {
                    out.extend_from_slice(&entities[i + 1..]);
                    return Some(out);
                }
                if raw_text.starts_with(prefix) {
                    let mut e = entities[i].clone();
                    if let HtmlEntity::Text { raw_text, .. } = &mut e {
                        *raw_text = raw_text[prefix.len()..].to_string();
                    }
                    out.push(e);
                    out.extend_from_slice(&entities[i + 1..]);
                    return Some(out);
                }
                return None;
            }
            _ => return None,
        }
    }

    None
}

pub fn clip_suffix_text(suffix: &str, entities: &[HtmlEntity]) -> Option<Vec<HtmlEntity>> {
    if entities.is_empty() {
        return if suffix.is_empty() {
            Some(Vec::new())
        } else {
            None
        };
    }

    let mut last_non_comment = None;
    for (idx, entity) in entities.iter().enumerate().rev() {
        if !matches!(entity, HtmlEntity::Comment { .. }) {
            last_non_comment = Some(idx);
            break;
        }
    }
    let last_idx = last_non_comment?;

    match &entities[last_idx] {
        HtmlEntity::Text { raw_text, .. } => {
            let mut out = entities[..last_idx].to_vec();
            if raw_text == suffix {
                out.extend_from_slice(&entities[last_idx + 1..]);
                return Some(out);
            }
            if raw_text.ends_with(suffix) {
                let clipped_len = raw_text.len() - suffix.len();
                let mut e = entities[last_idx].clone();
                if let HtmlEntity::Text { raw_text, .. } = &mut e {
                    *raw_text = raw_text[..clipped_len].to_string();
                }
                out.push(e);
                out.extend_from_slice(&entities[last_idx + 1..]);
                return Some(out);
            }
            None
        }
        _ => None,
    }
}
