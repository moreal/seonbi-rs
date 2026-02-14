use seonbi::{HtmlEntity, HtmlTag, is_preserved_entity, is_preserved_tag, is_preserved_tag_stack};

#[test]
fn preserved_tag_and_stack() {
    assert!(!is_preserved_tag(HtmlTag::P));
    assert!(!is_preserved_tag(HtmlTag::Em));
    assert!(!is_preserved_tag(HtmlTag::Title));
    assert!(is_preserved_tag(HtmlTag::Canvas));
    assert!(is_preserved_tag(HtmlTag::Code));
    assert!(is_preserved_tag(HtmlTag::Kbd));
    assert!(is_preserved_tag(HtmlTag::Pre));
    assert!(is_preserved_tag(HtmlTag::Script));
    assert!(is_preserved_tag(HtmlTag::Style));
    assert!(is_preserved_tag(HtmlTag::Template));
    assert!(is_preserved_tag(HtmlTag::TextArea));

    assert!(!is_preserved_tag_stack(&[].into()));
    assert!(!is_preserved_tag_stack(&[HtmlTag::P, HtmlTag::Em].into()));
    assert!(is_preserved_tag_stack(&[HtmlTag::Div, HtmlTag::Script].into()));
}

#[test]
fn preserved_entity_works() {
    assert!(!is_preserved_entity(&HtmlEntity::Text {
        tag_stack: [].into(),
        raw_text: String::new(),
    }));
    assert!(is_preserved_entity(&HtmlEntity::Comment {
        tag_stack: [].into(),
        comment: "...".to_string(),
    }));
    assert!(is_preserved_entity(&HtmlEntity::StartTag {
        tag_stack: [HtmlTag::P].into(),
        tag: HtmlTag::Code,
        raw_attributes: String::new(),
    }));
}
