use seonbi::{HtmlEntity, HtmlTag, normalize_text, scan_html};

fn text(stack: &[HtmlTag], raw: &str) -> HtmlEntity {
    HtmlEntity::Text { tag_stack: stack.to_vec().into(), raw_text: raw.to_string() }
}

fn start(stack: &[HtmlTag], tag: HtmlTag, attrs: &str) -> HtmlEntity {
    HtmlEntity::StartTag {
        tag_stack: stack.to_vec().into(),
        tag,
        raw_attributes: attrs.to_string(),
    }
}

fn end(stack: &[HtmlTag], tag: HtmlTag) -> HtmlEntity {
    HtmlEntity::EndTag { tag_stack: stack.to_vec().into(), tag }
}

#[test]
fn scanner_basics() {
    assert_eq!(scan_html("").unwrap(), vec![]);
    assert_eq!(scan_html("foobar").unwrap(), vec![text(&[], "foobar")]);
    assert_eq!(scan_html("foo<1bar").unwrap(), vec![text(&[], "foo<1bar")]);

    assert_eq!(
        scan_html("<!-- foo -->").unwrap(),
        vec![HtmlEntity::Comment { tag_stack: [].into(), comment: " foo ".to_string() }]
    );

    assert_eq!(
        scan_html("<![CDATA[foo]]>").unwrap(),
        vec![HtmlEntity::Cdata { tag_stack: [].into(), text: "foo".to_string() }]
    );
}

#[test]
fn scanner_comment_and_cdata_mixed() {
    assert_eq!(
        scan_html("a<!-- x --><![CDATA[y]]>b").unwrap(),
        vec![
            text(&[], "a"),
            HtmlEntity::Comment { tag_stack: [].into(), comment: " x ".to_string() },
            HtmlEntity::Cdata { tag_stack: [].into(), text: "y".to_string() },
            text(&[], "b"),
        ]
    );
}

#[test]
fn scanner_preserves_raw_attributes() {
    assert_eq!(
        scan_html("<p class=\"x\" data-y='z' disabled foo=bar title=\"a&amp;b\">ok</p>").unwrap(),
        vec![
            start(&[], HtmlTag::P, " class=\"x\" data-y='z' disabled foo=bar title=\"a&amp;b\"",),
            text(&[HtmlTag::P], "ok"),
            end(&[], HtmlTag::P),
        ]
    );
}

#[test]
fn scanner_void_and_self_closing_tags() {
    assert_eq!(
        scan_html("<div><img alt=\"x\" /><br/></div>").unwrap(),
        vec![
            start(&[], HtmlTag::Div, ""),
            start(&[HtmlTag::Div], HtmlTag::Img, " alt=\"x\" "),
            end(&[HtmlTag::Div], HtmlTag::Img),
            start(&[HtmlTag::Div], HtmlTag::BR, ""),
            end(&[HtmlTag::Div], HtmlTag::BR),
            end(&[], HtmlTag::Div),
        ]
    );

    assert_eq!(
        scan_html("<br></br><hr></hr>").unwrap(),
        vec![
            start(&[], HtmlTag::BR, ""),
            end(&[], HtmlTag::BR),
            start(&[], HtmlTag::HR, ""),
            end(&[], HtmlTag::HR),
        ]
    );
}

#[test]
fn scanner_tag_stack_tracking() {
    assert_eq!(
        scan_html("<p><em>test</em></p>").unwrap(),
        vec![
            start(&[], HtmlTag::P, ""),
            start(&[HtmlTag::P], HtmlTag::Em, ""),
            text(&[HtmlTag::P, HtmlTag::Em], "test"),
            end(&[HtmlTag::P], HtmlTag::Em),
            end(&[], HtmlTag::P),
        ]
    );

    assert_eq!(
        scan_html("<p><em/>").unwrap(),
        vec![
            start(&[], HtmlTag::P, ""),
            start(&[HtmlTag::P], HtmlTag::Em, ""),
            end(&[HtmlTag::P], HtmlTag::Em),
        ]
    );

    assert_eq!(
        scan_html("<p><hr>").unwrap(),
        vec![
            start(&[], HtmlTag::P, ""),
            start(&[HtmlTag::P], HtmlTag::HR, ""),
            end(&[HtmlTag::P], HtmlTag::HR),
        ]
    );
}

#[test]
fn scanner_flat_tag_structure() {
    assert_eq!(
        scan_html("<p>one</p><p>two</p>").unwrap(),
        vec![
            start(&[], HtmlTag::P, ""),
            text(&[HtmlTag::P], "one"),
            end(&[], HtmlTag::P),
            start(&[], HtmlTag::P, ""),
            text(&[HtmlTag::P], "two"),
            end(&[], HtmlTag::P),
        ]
    );
}

fn assert_malformed_as_text(input: &str) {
    let result = scan_html(input).unwrap();
    assert_eq!(
        normalize_text(result),
        vec![HtmlEntity::Text { tag_stack: [].into(), raw_text: input.to_string() }]
    );
}

#[test]
fn malformed_treated_as_text() {
    for malformed in
        ["<![CDATA[foo", "<!-- foo", "<invalid>", "<p class=\"unterminated>ok", "</unknown>"]
    {
        assert_malformed_as_text(malformed);
    }

    let result = scan_html("<p><invalid></p>").unwrap();
    assert_eq!(
        normalize_text(result),
        vec![start(&[], HtmlTag::P, ""), text(&[HtmlTag::P], "<invalid>"), end(&[], HtmlTag::P),]
    );
}
