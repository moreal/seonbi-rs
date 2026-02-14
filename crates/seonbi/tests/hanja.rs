use std::collections::BTreeMap;
use std::sync::Arc;

use seonbi::{
    HtmlEntity, HtmlTag, convert_initial_sound_law, def, hangul_only, hanja_in_parentheses,
    hanja_in_ruby, phoneticize_hanja, phoneticize_hanja_char, phoneticize_hanja_word,
    phoneticize_hanja_word_with_initial_sound_law, revert_initial_sound_law, with_dictionary,
};

#[test]
fn phoneticize_hanja_char_works() {
    assert_eq!(phoneticize_hanja_char('A'), 'A');
    assert_eq!(phoneticize_hanja_char('가'), '가');
    assert_eq!(phoneticize_hanja_char('金'), '금');
    assert_eq!(phoneticize_hanja_char('北'), '북');
    assert_eq!(phoneticize_hanja_char('六'), '륙');
    assert_eq!(phoneticize_hanja_char('禮'), '례');
}

#[test]
fn phoneticize_hanja_word_works() {
    assert_eq!(phoneticize_hanja_word("漢字"), "한자");
    assert_eq!(phoneticize_hanja_word("言文"), "언문");
    assert_eq!(phoneticize_hanja_word("餘念"), "여념");
    assert_eq!(phoneticize_hanja_word("來日"), "래일");
    assert_eq!(phoneticize_hanja_word("未來"), "미래");
}

#[test]
fn phoneticize_hanja_word_with_initial_sound_law_works() {
    assert_eq!(phoneticize_hanja_word_with_initial_sound_law("漢字"), "한자");
    assert_eq!(phoneticize_hanja_word_with_initial_sound_law("來日"), "내일");
    assert_eq!(phoneticize_hanja_word_with_initial_sound_law("良質"), "양질");
    assert_eq!(phoneticize_hanja_word_with_initial_sound_law("法律"), "법률");
    assert_eq!(phoneticize_hanja_word_with_initial_sound_law("第六共和國"), "제육공화국");
}

#[test]
fn with_dictionary_works() {
    let dict = BTreeMap::from([
        ("標識".to_string(), "표지".to_string()),
        ("毛澤東".to_string(), "마오쩌둥".to_string()),
        ("交通".to_string(), "교통".to_string()),
    ]);

    let fallback = |s: &str| phoneticize_hanja_word_with_initial_sound_law(s);
    assert_eq!(with_dictionary(&dict, &fallback, "標識"), "표지");
    assert_eq!(with_dictionary(&dict, &fallback, "交通標識"), "교통표지");
    assert_eq!(with_dictionary(&dict, &fallback, "知識"), "지식");
    assert_eq!(with_dictionary(&dict, &fallback, "安全標識"), "안전표지");
}

#[test]
fn phoneticize_hanja_entities_works() {
    let mut cfg = def();
    cfg.phoneticizer = Arc::new(phoneticize_hanja_word_with_initial_sound_law);
    cfg.word_renderer = Arc::new(hangul_only);
    cfg.homophone_renderer = Arc::new(hanja_in_parentheses);

    let input = vec![HtmlEntity::Text {
        tag_stack: [].into(),
        raw_text: "1996年 그들이 地球를 支配했을 때".to_string(),
    }];

    assert_eq!(
        seonbi::normalize_text(phoneticize_hanja(&cfg, input)),
        vec![HtmlEntity::Text {
            tag_stack: [].into(),
            raw_text: "1996년 그들이 지구를 지배했을 때".to_string(),
        }]
    );
}

#[test]
fn hanja_renderers_work() {
    let stack = [].into();
    assert_eq!(
        hangul_only(&stack, "地球", "지구"),
        vec![HtmlEntity::Cdata { tag_stack: [].into(), text: "지구".to_string() }]
    );
    assert_eq!(
        hanja_in_parentheses(&stack, "地球", "지구"),
        vec![HtmlEntity::Cdata { tag_stack: [].into(), text: "지구(地球)".to_string() }]
    );

    let ruby = seonbi::normalize_text(hanja_in_ruby(&stack, "地球", "지구"));
    assert!(matches!(ruby.first(), Some(HtmlEntity::StartTag { tag: HtmlTag::Ruby, .. })));
}

#[test]
fn skips_preserved_and_non_korean() {
    let cfg = def();
    let input = vec![
        HtmlEntity::Text {
            tag_stack: [HtmlTag::Pre].into(), raw_text: "1996年 地球".to_string()
        },
        HtmlEntity::StartTag {
            tag_stack: [].into(),
            tag: HtmlTag::Span,
            raw_attributes: "lang=ja".to_string(),
        },
        HtmlEntity::Text {
            tag_stack: [HtmlTag::Span].into(),
            raw_text: "誰も知らない".to_string(),
        },
        HtmlEntity::EndTag { tag_stack: [].into(), tag: HtmlTag::Span },
    ];

    assert_eq!(phoneticize_hanja(&cfg, input.clone()), input);
}

#[test]
fn initial_sound_law_helpers() {
    assert_eq!(convert_initial_sound_law('녀'), '여');
    assert_eq!(convert_initial_sound_law('량'), '양');
    assert_eq!(convert_initial_sound_law('락'), '낙');

    let reverted = revert_initial_sound_law('여');
    assert!(reverted.contains(&'녀'));
    assert!(reverted.contains(&'려'));
}
