use seonbi::{HtmlEntity, HtmlTag, HtmlTagStack, print_html, print_xhtml};

#[test]
fn print_html_and_xhtml() {
    let sample = vec![
        HtmlEntity::Comment { tag_stack: HtmlTagStack::empty(), comment: " foo ".to_string() },
        HtmlEntity::StartTag {
            tag_stack: HtmlTagStack::empty(),
            tag: HtmlTag::P,
            raw_attributes: " id=\"a\"".to_string(),
        },
        HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "Hello,".to_string() },
        HtmlEntity::StartTag {
            tag_stack: [HtmlTag::P].into(),
            tag: HtmlTag::BR,
            raw_attributes: String::new(),
        },
        HtmlEntity::EndTag { tag_stack: [HtmlTag::P].into(), tag: HtmlTag::BR },
        HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "\n".to_string() },
        HtmlEntity::StartTag {
            tag_stack: [HtmlTag::P].into(),
            tag: HtmlTag::Em,
            raw_attributes: "class=\"b\"".to_string(),
        },
        HtmlEntity::Cdata {
            tag_stack: [HtmlTag::P, HtmlTag::Em].into(),
            text: "world".to_string(),
        },
        HtmlEntity::EndTag { tag_stack: [HtmlTag::P].into(), tag: HtmlTag::Em },
        HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "!".to_string() },
        HtmlEntity::EndTag { tag_stack: HtmlTagStack::empty(), tag: HtmlTag::P },
        HtmlEntity::StartTag {
            tag_stack: HtmlTagStack::empty(),
            tag: HtmlTag::P,
            raw_attributes: String::new(),
        },
        HtmlEntity::EndTag { tag_stack: HtmlTagStack::empty(), tag: HtmlTag::P },
    ];

    assert_eq!(
        print_html(&sample),
        "<!-- foo --><p id=\"a\">Hello,<br>\n<em class=\"b\"><![CDATA[world]]></em>!</p><p></p>"
    );
    assert_eq!(
        print_xhtml(&sample),
        "<!-- foo --><p id=\"a\">Hello,<br/>\n<em class=\"b\"><![CDATA[world]]></em>!</p><p></p>"
    );
}
