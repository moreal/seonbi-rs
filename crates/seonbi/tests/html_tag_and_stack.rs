use std::str::FromStr;

use seonbi::{
    HtmlTag, HtmlTagKind, HtmlTagStack, heading_level, heading_tag, html_tag_kind, html_tag_name,
};

#[test]
fn tag_kind_name_heading() {
    assert_eq!(html_tag_kind(HtmlTag::BR), HtmlTagKind::Void);
    assert_eq!(html_tag_kind(HtmlTag::Script), HtmlTagKind::RawText);
    assert_eq!(html_tag_kind(HtmlTag::TextArea), HtmlTagKind::EscapableRawText);
    assert_eq!(html_tag_name(HtmlTag::TextArea), "textarea");
    assert_eq!(HtmlTag::from_str("blockquote"), Ok(HtmlTag::BlockQuote));
    assert_eq!(heading_level(HtmlTag::H1), Some(1));
    assert_eq!(heading_level(HtmlTag::P), None);
    assert_eq!(heading_tag(6), Some(HtmlTag::H6));
    assert_eq!(heading_tag(7), None);
}

#[test]
fn tag_stack_works() {
    let stack = HtmlTagStack::from([HtmlTag::Div, HtmlTag::Article, HtmlTag::P, HtmlTag::Em]);
    assert_eq!(stack.depth(), 4);
    assert_eq!(stack.last(), Some(HtmlTag::Em));

    let popped = stack.pop(HtmlTag::P);
    assert_eq!(popped.to_list(), vec![HtmlTag::Div, HtmlTag::Article, HtmlTag::Em]);

    let target =
        HtmlTagStack::from([HtmlTag::Article, HtmlTag::BlockQuote, HtmlTag::P, HtmlTag::Em]);
    let rebased = target.rebase(
        &HtmlTagStack::from([HtmlTag::Article, HtmlTag::BlockQuote]),
        &HtmlTagStack::from([HtmlTag::Div]),
    );
    assert_eq!(rebased.to_list(), vec![HtmlTag::Div, HtmlTag::P, HtmlTag::Em]);

    assert!(stack.descends_from(&HtmlTagStack::from([HtmlTag::Div, HtmlTag::Article])));
}
