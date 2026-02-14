use seonbi::{HtmlEntity, HtmlTag, clip_prefix_text, clip_suffix_text, clip_text};

#[test]
fn clip_prefix_text_works() {
    assert_eq!(clip_prefix_text("foo", &[]), None);
    assert_eq!(clip_prefix_text("", &[]), Some(vec![]));

    let entities = vec![
        HtmlEntity::Comment { tag_stack: [].into(), comment: "comment".to_string() },
        HtmlEntity::Text { tag_stack: [].into(), raw_text: "foobar".to_string() },
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::P,
            raw_attributes: String::new(),
        },
    ];

    assert_eq!(
        clip_prefix_text("foo", &entities),
        Some(vec![
            HtmlEntity::Comment { tag_stack: [].into(), comment: "comment".to_string() },
            HtmlEntity::Text { tag_stack: [].into(), raw_text: "bar".to_string() },
            HtmlEntity::StartTag {
                tag_stack: [].into(),
                tag: HtmlTag::P,
                raw_attributes: String::new(),
            }
        ])
    );
}

#[test]
fn clip_suffix_text_works() {
    let entities = vec![
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::P,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "foo".to_string() },
        HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
        HtmlEntity::Text { tag_stack: [].into(), raw_text: "foobar".to_string() },
    ];

    assert_eq!(
        clip_suffix_text("bar", &entities),
        Some(vec![
            HtmlEntity::StartTag {
                tag_stack: [].into(),
                tag: HtmlTag::P,
                raw_attributes: String::new(),
            },
            HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "foo".to_string() },
            HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
            HtmlEntity::Text { tag_stack: [].into(), raw_text: "foo".to_string() },
        ])
    );
}

#[test]
fn clip_text_works() {
    let entities = vec![
        HtmlEntity::Text { tag_stack: [].into(), raw_text: "foo".to_string() },
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::P,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "bar".to_string() },
        HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
        HtmlEntity::Text { tag_stack: [].into(), raw_text: "baz".to_string() },
    ];

    assert_eq!(
        clip_text("foo", "baz", &entities),
        Some(vec![
            HtmlEntity::StartTag {
                tag_stack: [].into(),
                tag: HtmlTag::P,
                raw_attributes: String::new(),
            },
            HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "bar".to_string() },
            HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
        ])
    );
}
