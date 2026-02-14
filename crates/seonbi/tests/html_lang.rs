use seonbi::{
    HtmlEntity, HtmlTag, LangHtmlEntity, annotate_with_lang, extract_lang, is_korean,
    is_never_korean,
};

#[test]
fn extract_lang_works() {
    assert_eq!(extract_lang(""), None);
    assert_eq!(extract_lang("lang=en"), Some("en".to_string()));
    assert_eq!(extract_lang("lang='ko-KR'"), Some("ko-kr".to_string()));
    assert_eq!(extract_lang("id=\"foo\" lang=zh-CN class=bar"), Some("zh-cn".to_string()));
}

#[test]
fn annotate_with_lang_works() {
    let source = vec![
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::P,
            raw_attributes: "id=\"foo\" lang=\"en\"".to_string(),
        },
        HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "English".to_string() },
        HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
    ];

    assert_eq!(
        annotate_with_lang(source),
        vec![
            LangHtmlEntity {
                lang: Some("en".to_string()),
                entity: HtmlEntity::StartTag {
                    tag_stack: [].into(),
                    tag: HtmlTag::P,
                    raw_attributes: "id=\"foo\" lang=\"en\"".to_string(),
                },
            },
            LangHtmlEntity {
                lang: Some("en".to_string()),
                entity: HtmlEntity::Text {
                    tag_stack: [HtmlTag::P].into(),
                    raw_text: "English".to_string(),
                },
            },
            LangHtmlEntity {
                lang: Some("en".to_string()),
                entity: HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
            },
        ]
    );
}

#[test]
fn korean_check_works() {
    assert!(is_korean("ko-KR"));
    assert!(!is_korean("en"));
    assert!(!is_never_korean(&None));
    assert!(is_never_korean(&Some("en".to_string())));
}
