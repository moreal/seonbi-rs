use seonbi::{normalize_text, scan_html, HtmlEntity, HtmlTag};

fn text(stack: &[HtmlTag], raw: &str) -> HtmlEntity {
    HtmlEntity::Text {
        tag_stack: stack.to_vec().into(),
        raw_text: raw.to_string(),
    }
}

#[test]
fn scanner_basics() {
    assert_eq!(scan_html("").unwrap(), vec![]);
    assert_eq!(scan_html("foobar").unwrap(), vec![text(&[], "foobar")]);

    assert_eq!(
        scan_html("<!-- foo -->").unwrap(),
        vec![HtmlEntity::Comment {
            tag_stack: [].into(),
            comment: " foo ".to_string(),
        }]
    );

    assert_eq!(
        scan_html("<![CDATA[foo]]>").unwrap(),
        vec![HtmlEntity::Cdata {
            tag_stack: [].into(),
            text: "foo".to_string(),
        }]
    );
}

#[test]
fn scanner_tag_and_stack() {
    assert_eq!(
        scan_html("<p><em>test</em></p>").unwrap(),
        vec![
            HtmlEntity::StartTag {
                tag_stack: [].into(),
                tag: HtmlTag::P,
                raw_attributes: String::new(),
            },
            HtmlEntity::StartTag {
                tag_stack: [HtmlTag::P].into(),
                tag: HtmlTag::Em,
                raw_attributes: String::new(),
            },
            text(&[HtmlTag::P, HtmlTag::Em], "test"),
            HtmlEntity::EndTag {
                tag_stack: [HtmlTag::P].into(),
                tag: HtmlTag::Em,
            },
            HtmlEntity::EndTag {
                tag_stack: [].into(),
                tag: HtmlTag::P,
            },
        ]
    );

    assert_eq!(
        scan_html("<p><em/>").unwrap(),
        vec![
            HtmlEntity::StartTag {
                tag_stack: [].into(),
                tag: HtmlTag::P,
                raw_attributes: String::new(),
            },
            HtmlEntity::StartTag {
                tag_stack: [HtmlTag::P].into(),
                tag: HtmlTag::Em,
                raw_attributes: String::new(),
            },
            HtmlEntity::EndTag {
                tag_stack: [HtmlTag::P].into(),
                tag: HtmlTag::Em,
            },
        ]
    );

    assert_eq!(
        scan_html("<p><hr>").unwrap(),
        vec![
            HtmlEntity::StartTag {
                tag_stack: [].into(),
                tag: HtmlTag::P,
                raw_attributes: String::new(),
            },
            HtmlEntity::StartTag {
                tag_stack: [HtmlTag::P].into(),
                tag: HtmlTag::HR,
                raw_attributes: String::new(),
            },
            HtmlEntity::EndTag {
                tag_stack: [HtmlTag::P].into(),
                tag: HtmlTag::HR,
            },
        ]
    );
}

#[test]
fn malformed_treated_as_text() {
    let result = scan_html("<![CDATA[foo").unwrap();
    assert_eq!(
        normalize_text(result),
        vec![HtmlEntity::Text {
            tag_stack: [].into(),
            raw_text: "<![CDATA[foo".to_string(),
        }]
    );

    let result = scan_html("<invalid>").unwrap();
    assert_eq!(
        normalize_text(result),
        vec![HtmlEntity::Text {
            tag_stack: [].into(),
            raw_text: "<invalid>".to_string(),
        }]
    );
}
