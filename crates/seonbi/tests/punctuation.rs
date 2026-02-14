use std::collections::BTreeSet;

use seonbi::{
    ArrowTransformationOption, CitationQuotes, HtmlEntity, HtmlTag, angle_quotes, corner_brackets,
    curved_quotes, curved_single_quotes_with_q, guillemets, horizontal_stops,
    horizontal_stops_with_slashes, normalize_stops, normalize_text, quote_citation,
    transform_arrow, transform_ellipsis, transform_em_dash, transform_quote, vertical_stops,
};

fn arrow_sample(tag: HtmlTag) -> Vec<HtmlEntity> {
    vec![
        HtmlEntity::StartTag { tag_stack: [].into(), tag, raw_attributes: String::new() },
        HtmlEntity::Text {
            tag_stack: [tag].into(),
            raw_text: "A -&gt; B, B &lt;- A, C &lt;-&gt; D".to_string(),
        },
        HtmlEntity::StartTag {
            tag_stack: [tag].into(),
            tag: HtmlTag::BR,
            raw_attributes: String::new(),
        },
        HtmlEntity::EndTag { tag_stack: [tag].into(), tag: HtmlTag::BR },
        HtmlEntity::Text {
            tag_stack: [tag].into(),
            raw_text: "a =&#62; b, b &#60;= a, c &#X3C;=&#x3e; d".to_string(),
        },
        HtmlEntity::EndTag { tag_stack: [].into(), tag },
    ]
}

#[test]
fn quote_citation_works() {
    let title_input = vec![
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::P,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text {
            tag_stack: [HtmlTag::P].into(),
            raw_text: "&lt;&lt;無情&gt;&gt;".to_string(),
        },
        HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
    ];

    assert_eq!(
        quote_citation(&angle_quotes(), title_input.clone()),
        vec![
            HtmlEntity::StartTag {
                tag_stack: [].into(),
                tag: HtmlTag::P,
                raw_attributes: String::new(),
            },
            HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "&#12298;".to_string() },
            HtmlEntity::StartTag {
                tag_stack: [HtmlTag::P].into(),
                tag: HtmlTag::Cite,
                raw_attributes: String::new(),
            },
            HtmlEntity::Text {
                tag_stack: [HtmlTag::P, HtmlTag::Cite].into(),
                raw_text: "無情".to_string(),
            },
            HtmlEntity::EndTag { tag_stack: [HtmlTag::P].into(), tag: HtmlTag::Cite },
            HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "&#12299;".to_string() },
            HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
        ]
    );

    let mut without_cite: CitationQuotes = corner_brackets();
    without_cite.html_element = None;
    let out = quote_citation(&without_cite, title_input);
    assert!(!out.iter().any(|e| matches!(e, HtmlEntity::StartTag { tag: HtmlTag::Cite, .. })));
}

#[test]
fn transform_arrow_works() {
    let none = transform_arrow(&BTreeSet::new(), arrow_sample(HtmlTag::P));
    assert_eq!(
        none,
        vec![
            HtmlEntity::StartTag {
                tag_stack: [].into(),
                tag: HtmlTag::P,
                raw_attributes: String::new(),
            },
            HtmlEntity::Text {
                tag_stack: [HtmlTag::P].into(),
                raw_text: "A &rarr; B, B &larr; A, C &larr;&gt; D".to_string(),
            },
            HtmlEntity::StartTag {
                tag_stack: [HtmlTag::P].into(),
                tag: HtmlTag::BR,
                raw_attributes: String::new(),
            },
            HtmlEntity::EndTag { tag_stack: [HtmlTag::P].into(), tag: HtmlTag::BR },
            HtmlEntity::Text {
                tag_stack: [HtmlTag::P].into(),
                raw_text: "a =&#62; b, b &#60;= a, c &#X3C;=&#x3e; d".to_string(),
            },
            HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
        ]
    );

    let mut lr = BTreeSet::new();
    lr.insert(ArrowTransformationOption::LeftRight);
    let out = transform_arrow(&lr, arrow_sample(HtmlTag::P));
    assert!(matches!(
        &out[1],
        HtmlEntity::Text { raw_text, .. } if raw_text.contains("&harr;")
    ));

    let mut both = BTreeSet::new();
    both.insert(ArrowTransformationOption::LeftRight);
    both.insert(ArrowTransformationOption::DoubleArrow);
    let out = transform_arrow(&both, arrow_sample(HtmlTag::P));
    assert!(matches!(
        &out[4],
        HtmlEntity::Text { raw_text, .. } if raw_text.contains("&hArr;") && raw_text.contains("&lArr;") && raw_text.contains("&rArr;")
    ));
}

#[test]
fn transform_ellipsis_works() {
    let sample = vec![
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::P,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text { tag_stack: [HtmlTag::P].into(), raw_text: "abc...def".to_string() },
        HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::Pre,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text { tag_stack: [HtmlTag::Pre].into(), raw_text: "ignore...".to_string() },
        HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::Pre },
    ];
    let out = transform_ellipsis(sample);
    assert!(matches!(
        &out[1],
        HtmlEntity::Text { raw_text, .. } if raw_text == "abc&hellip;def"
    ));
    assert!(matches!(
        &out[4],
        HtmlEntity::Text { raw_text, .. } if raw_text == "ignore..."
    ));
}

#[test]
fn transform_quote_works() {
    let input =
        vec![HtmlEntity::Text { tag_stack: [].into(), raw_text: "'a' \"b\" c".to_string() }];

    let out = normalize_text(transform_quote(&curved_quotes(), input.clone()));
    assert_eq!(
        out,
        vec![HtmlEntity::Text {
            tag_stack: [].into(),
            raw_text: "&lsquo;a&rsquo; &ldquo;b&rdquo; c".to_string(),
        }]
    );

    let out = normalize_text(transform_quote(&guillemets(), input.clone()));
    assert_eq!(
        out,
        vec![HtmlEntity::Text {
            tag_stack: [].into(),
            raw_text: "&#x3008;a&#x3009; &#x300a;b&#x300b; c".to_string(),
        }]
    );

    let out = normalize_text(transform_quote(&curved_single_quotes_with_q(), input));
    assert_eq!(
        out,
        vec![
            HtmlEntity::Text { tag_stack: [].into(), raw_text: "&lsquo;a&rsquo; ".to_string() },
            HtmlEntity::StartTag {
                tag_stack: [].into(),
                tag: HtmlTag::Q,
                raw_attributes: String::new(),
            },
            HtmlEntity::Text { tag_stack: [HtmlTag::Q].into(), raw_text: "b".to_string() },
            HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::Q },
            HtmlEntity::Text { tag_stack: [].into(), raw_text: " c".to_string() },
        ]
    );
}

#[test]
fn transform_em_dash_works() {
    let out = transform_em_dash(vec![HtmlEntity::Text {
        tag_stack: [].into(),
        raw_text: "A - B -- C ㅡ D".to_string(),
    }]);
    assert_eq!(
        out,
        vec![HtmlEntity::Text {
            tag_stack: [].into(),
            raw_text: "A&mdash;B &mdash; C&mdash;D".to_string(),
        }]
    );
}

#[test]
fn normalize_stops_works() {
    let input = vec![
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::P,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text {
            tag_stack: [HtmlTag::P].into(),
            raw_text: "봄·여름。어제, 오늘? 아침!".to_string(),
        },
        HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::P },
    ];

    let horizontal = normalize_stops(&horizontal_stops(), input.clone());
    assert!(matches!(
        &horizontal[1],
        HtmlEntity::Text { raw_text, .. } if raw_text.contains(". ") && raw_text.contains(", ") && raw_text.contains("? ") && raw_text.contains("!")
    ));

    let vertical = normalize_stops(&vertical_stops(), input.clone());
    assert!(matches!(
        &vertical[1],
        HtmlEntity::Text { raw_text, .. } if raw_text.contains("&#x3002;") && raw_text.contains("&#x3001;") && raw_text.contains("&#xff1f;") && raw_text.contains("&#xff01;")
    ));

    let with_slash = normalize_stops(&horizontal_stops_with_slashes(), input);
    assert!(matches!(
        &with_slash[1],
        HtmlEntity::Text { raw_text, .. } if raw_text.contains("/")
    ));
}
