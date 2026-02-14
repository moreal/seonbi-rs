use seonbi::{HtmlEntity, HtmlTag, is_wrapped_by, is_wrapped_by_exact, wrap};

#[test]
fn wrap_works() {
    let wrapped = wrap(
        &[HtmlTag::Div, HtmlTag::Article].into(),
        HtmlTag::BlockQuote,
        " class=\"q\"",
        vec![
            HtmlEntity::StartTag {
                tag_stack: [HtmlTag::Div, HtmlTag::Article].into(),
                tag: HtmlTag::P,
                raw_attributes: String::new(),
            },
            HtmlEntity::Text {
                tag_stack: [HtmlTag::Div, HtmlTag::Article, HtmlTag::P].into(),
                raw_text: "foo".to_string(),
            },
            HtmlEntity::EndTag {
                tag_stack: [HtmlTag::Div, HtmlTag::Article].into(),
                tag: HtmlTag::P,
            },
        ],
    );

    assert!(is_wrapped_by(&wrapped, HtmlTag::BlockQuote));
    assert!(is_wrapped_by_exact(&wrapped, HtmlTag::BlockQuote, Some(" class=\"q\"")));
}
