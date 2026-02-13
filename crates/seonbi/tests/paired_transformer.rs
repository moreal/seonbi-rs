use seonbi::{
    HtmlEntity, HtmlTag, HtmlTagStack, PairedTransformer, clip_text, is_preserved_tag_stack,
    normalize_text, transform_pairs,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Marker {
    Paren,
}

fn split_once(text: &str, token: &str) -> Option<(String, String, String)> {
    let idx = text.find(token)?;
    let pre = text[..idx].to_string();
    let post = text[idx + token.len()..].to_string();
    Some((pre, token.to_string(), post))
}

fn make_transformer() -> PairedTransformer<Marker> {
    PairedTransformer {
        ignores_tag_stack: Box::new(is_preserved_tag_stack),
        match_start: Box::new(|_prev, text| {
            split_once(text, "(").map(|(pre, token, post)| (Marker::Paren, pre, token, post))
        }),
        match_end: Box::new(|text| {
            split_once(text, ")").map(|(pre, token, post)| (Marker::Paren, pre, token, post))
        }),
        are_matches_paired: Box::new(|a, b| a == b),
        transform_pair: Box::new(|_, _, buffer| {
            let middle = clip_text("(", ")", &buffer).unwrap_or(buffer);
            let stack =
                middle.first().map(|e| e.tag_stack().clone()).unwrap_or_else(HtmlTagStack::empty);
            vec![HtmlEntity::Text { tag_stack: stack, raw_text: "PAIR".to_string() }]
        }),
    }
}

#[test]
fn transforms_simple_pair() {
    let input = vec![HtmlEntity::Text { tag_stack: [].into(), raw_text: "x (y) z".to_string() }];

    let out = normalize_text(transform_pairs(&make_transformer(), input));
    assert_eq!(
        out,
        vec![HtmlEntity::Text { tag_stack: [].into(), raw_text: "x PAIR z".to_string() }]
    );
}

#[test]
fn transforms_pair_across_html_boundaries() {
    let input = vec![
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::P,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "(ab".to_string() },
        HtmlEntity::StartTag {
            tag_stack: [HtmlTag::P].into(),
            tag: HtmlTag::Em,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text {
            tag_stack: [HtmlTag::P, HtmlTag::Em].into(),
            raw_text: "cd".to_string(),
        },
        HtmlEntity::EndTag { tag_stack: [HtmlTag::P].into(), tag: HtmlTag::Em },
        HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: ")".to_string() },
        HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
    ];

    let out = normalize_text(transform_pairs(&make_transformer(), input));
    assert!(out.iter().any(|e| {
        matches!(
            e,
            HtmlEntity::Text { raw_text, .. }
            if raw_text == "PAIR"
        )
    }));
}

#[test]
fn skips_preserved_tag_stack() {
    let input = vec![
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::Pre,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text { tag_stack: [HtmlTag::Pre].into(), raw_text: "(x)".to_string() },
        HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::Pre },
    ];

    let out = normalize_text(transform_pairs(&make_transformer(), input.clone()));
    assert_eq!(out, input);
}
