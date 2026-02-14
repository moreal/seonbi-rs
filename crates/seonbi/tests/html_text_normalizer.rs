use seonbi::{HtmlEntity, HtmlTag, escape_html_entities, normalize_cdata, normalize_text};

#[test]
fn normalize_text_works() {
    let normalized = normalize_text(vec![
        HtmlEntity::Text { tag_stack: [].into(), raw_text: "foo ".to_string() },
        HtmlEntity::Text { tag_stack: [].into(), raw_text: "&amp; bar".to_string() },
        HtmlEntity::Cdata { tag_stack: [].into(), text: " & baz ".to_string() },
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::P,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "qux ".to_string() },
        HtmlEntity::Cdata { tag_stack: [HtmlTag::P].into(), text: "& \"quux\"".to_string() },
        HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
        HtmlEntity::Cdata { tag_stack: [].into(), text: " <end>".to_string() },
    ]);

    assert_eq!(
        normalized,
        vec![
            HtmlEntity::Text {
                tag_stack: [].into(),
                raw_text: "foo &amp; bar &amp; baz ".to_string(),
            },
            HtmlEntity::StartTag {
                tag_stack: [].into(),
                tag: HtmlTag::P,
                raw_attributes: String::new(),
            },
            HtmlEntity::Text {
                tag_stack: [HtmlTag::P].into(),
                raw_text: "qux &amp; &quot;quux&quot;".to_string(),
            },
            HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
            HtmlEntity::Text { tag_stack: [].into(), raw_text: " &lt;end&gt;".to_string() },
        ]
    );
}

#[test]
fn normalize_cdata_works() {
    let entity = HtmlEntity::Cdata {
        tag_stack: [HtmlTag::Div, HtmlTag::P].into(),
        text: "<p>foo & bar</p>".to_string(),
    };
    assert_eq!(
        normalize_cdata(entity),
        HtmlEntity::Text {
            tag_stack: [HtmlTag::Div, HtmlTag::P].into(),
            raw_text: "&lt;p&gt;foo &amp; bar&lt;/p&gt;".to_string(),
        }
    );
}

#[test]
fn escape_entities_works() {
    assert_eq!(escape_html_entities("<p id=\"foo\">"), "&lt;p id=&quot;foo&quot;&gt;");
    assert_eq!(escape_html_entities("AT&T"), "AT&amp;T");
}
