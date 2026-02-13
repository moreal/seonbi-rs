use super::entity::HtmlEntity;

pub fn normalize_text(entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    let mut out = Vec::new();
    let mut i = 0usize;

    while i < entities.len() {
        let current = &entities[i];
        if !matches!(current, HtmlEntity::Text { .. } | HtmlEntity::Cdata { .. }) {
            out.push(current.clone());
            i += 1;
            continue;
        }

        let stack = current.tag_stack().clone();
        let mut merged = String::new();

        while i < entities.len() {
            let e = &entities[i];
            match e {
                HtmlEntity::Text { tag_stack, raw_text } if *tag_stack == stack => {
                    merged.push_str(raw_text);
                    i += 1;
                }
                HtmlEntity::Cdata { tag_stack, text } if *tag_stack == stack => {
                    merged.push_str(&escape_html_entities(text));
                    i += 1;
                }
                _ => break,
            }
        }

        out.push(HtmlEntity::Text {
            tag_stack: stack,
            raw_text: merged,
        });
    }

    out
}

pub fn normalize_cdata(entity: HtmlEntity) -> HtmlEntity {
    match entity {
        HtmlEntity::Cdata { tag_stack, text } => HtmlEntity::Text {
            tag_stack,
            raw_text: escape_html_entities(&text),
        },
        _ => entity,
    }
}

pub fn escape_html_entities(text: &str) -> String {
    let mut escaped = String::new();
    for c in text.chars() {
        match c {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            _ => escaped.push(c),
        }
    }
    escaped
}
