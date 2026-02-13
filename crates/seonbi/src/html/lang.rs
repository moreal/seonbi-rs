use super::{entity::HtmlEntity, tag::HtmlTag};

pub type LanguageTag = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangHtmlEntity {
    pub lang: Option<LanguageTag>,
    pub entity: HtmlEntity,
}

pub fn extract_lang(raw_attrs: &str) -> Option<LanguageTag> {
    let mut i = 0usize;
    let bytes = raw_attrs.as_bytes();
    let mut found: Option<String> = None;

    while i < bytes.len() {
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        let name_start = i;
        while i < bytes.len()
            && !(bytes[i] as char).is_whitespace()
            && bytes[i] as char != '='
        {
            i += 1;
        }
        let name = &raw_attrs[name_start..i];

        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }

        let mut value = String::new();
        if i < bytes.len() && bytes[i] as char == '=' {
            i += 1;
            while i < bytes.len() && (bytes[i] as char).is_whitespace() {
                i += 1;
            }
            if i >= bytes.len() {
                value.clear();
            } else if bytes[i] as char == '"' || bytes[i] as char == '\'' {
                let quote = bytes[i] as char;
                i += 1;
                let start = i;
                while i < bytes.len() && bytes[i] as char != quote {
                    i += 1;
                }
                value = raw_attrs[start..i.min(bytes.len())].to_string();
                if i < bytes.len() && bytes[i] as char == quote {
                    i += 1;
                }
            } else {
                let start = i;
                while i < bytes.len() && !(bytes[i] as char).is_whitespace() {
                    i += 1;
                }
                value = raw_attrs[start..i].to_string();
            }
        }

        if name.eq_ignore_ascii_case("lang") {
            found = Some(value);
            break;
        }
    }

    let normalized = found
        .map(|s| {
            s.replace("&hyphen;", "-")
                .replace("&dash;", "-")
                .replace("&#8208;", "-")
                .replace("&#x2010;", "-")
                .replace("&#X2010;", "-")
                .trim()
                .to_ascii_lowercase()
        })
        .filter(|s| !s.is_empty());

    normalized
}

pub fn annotate_with_lang(entities: Vec<HtmlEntity>) -> Vec<LangHtmlEntity> {
    let mut stack: Vec<(HtmlTag, Option<LanguageTag>)> = Vec::new();
    let mut out = Vec::with_capacity(entities.len());

    for entity in entities {
        let entity_probe = entity.clone();
        match entity_probe {
            HtmlEntity::StartTag {
                tag,
                raw_attributes,
                ..
            } => {
                let parent_lang = stack.first().and_then(|(_, l)| l.clone());
                let this_lang = extract_lang(&raw_attributes).or(parent_lang);
                out.push(LangHtmlEntity {
                    lang: this_lang.clone(),
                    entity,
                });
                stack.insert(0, (tag, this_lang));
            }
            HtmlEntity::EndTag { tag, .. } => {
                let this_lang = stack.first().and_then(|(_, l)| l.clone());
                if let Some((t, _)) = stack.first() {
                    if *t == tag {
                        stack.remove(0);
                    }
                }
                out.push(LangHtmlEntity {
                    lang: this_lang,
                    entity,
                });
            }
            _ => {
                let parent_lang = stack.first().and_then(|(_, l)| l.clone());
                out.push(LangHtmlEntity {
                    lang: parent_lang,
                    entity,
                });
            }
        }
    }

    out
}

pub fn is_korean(lang: &str) -> bool {
    let l = lang.to_ascii_lowercase();
    l == "ko" || l == "kor" || l.starts_with("ko-") || l.starts_with("kor-")
}

pub fn is_never_korean(lang: &Option<LanguageTag>) -> bool {
    match lang {
        None => false,
        Some(l) => !is_korean(l),
    }
}
