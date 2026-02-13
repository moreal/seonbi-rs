use seonbi::{
    ContentType, HtmlEntity, as_common_mark_transformer, as_html_transformer,
    as_plain_text_transformer, as_xhtml_transformer, transform_with_content_type,
};

fn text_reverser(entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    entities.into_iter().map(reverse_text).collect()
}

fn reverse_text(entity: HtmlEntity) -> HtmlEntity {
    match entity {
        HtmlEntity::Text {
            tag_stack,
            raw_text,
        } => {
            let decoded = html_escape::decode_html_entities(&raw_text).to_string();
            let reversed: String = decoded.chars().rev().collect();
            HtmlEntity::Text {
                tag_stack,
                raw_text: html_escape::encode_text(&reversed).to_string(),
            }
        }
        HtmlEntity::Cdata { tag_stack, text } => HtmlEntity::Cdata {
            tag_stack,
            text: text.chars().rev().collect(),
        },
        other => other,
    }
}

#[test]
fn as_html_transformer_works() {
    let r = as_html_transformer(text_reverser, "<p>foo <em>bar</em><br> baz</p>").unwrap();
    assert_eq!(r, "<p> oof<em>rab</em><br>zab </p>");
}

#[test]
fn as_xhtml_transformer_works() {
    let r = as_xhtml_transformer(text_reverser, "<p>foo <em>bar</em><br> baz</p>").unwrap();
    assert_eq!(r, "<p> oof<em>rab</em><br/>zab </p>");
}

#[test]
fn as_plain_text_transformer_works() {
    let r = as_plain_text_transformer(text_reverser, "<p>foo <em>bar</em><br> baz</p>").unwrap();
    assert_eq!(r, ">p/<zab >rb<>me/<rab>me< oof>p<");
}

#[test]
fn as_common_mark_transformer_works() {
    let r = as_common_mark_transformer(
        text_reverser,
        "# Foo\n\nBar *Baz*\nQux\n\n> Quote <em>tag</em>\n",
    )
    .unwrap();
    assert_eq!(r, "# ooF\n\n raB*zaB*\nxuQ\n\n>  etouQ<em>gat</em>\n");
}

#[test]
fn transform_with_content_type_works() {
    let input = "*foo* <em>bar</em><br>";

    let html =
        transform_with_content_type(&ContentType::from("text/html"), text_reverser, input).unwrap();
    assert_eq!(html, " *oof*<em>rab</em><br>");

    let xhtml = transform_with_content_type(
        &ContentType::from("application/xhtml+xml"),
        text_reverser,
        input,
    )
    .unwrap();
    assert_eq!(xhtml, " *oof*<em>rab</em><br/>");

    let plain = transform_with_content_type(&ContentType::from("text/plain"), text_reverser, input)
        .unwrap();
    assert_eq!(plain, ">rb<>me/<rab>me< *oof*");

    let markdown =
        transform_with_content_type(&ContentType::from("text/markdown"), text_reverser, input)
            .unwrap();
    assert_eq!(markdown, "*oof* <em>rab</em><br>\n");
}
