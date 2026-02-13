use std::collections::{BTreeMap, BTreeSet};

use crate::hangul::{from_jamo_triple, to_jamo_triple};
use crate::html::{HtmlEntity, HtmlTagStack};
use crate::unihan::k_hangul::k_hangul_data;

pub type HanjaDictionary = BTreeMap<String, String>;
pub type HanjaWordPhoneticizer = fn(&str) -> String;
pub type HanjaWordRenderer = fn(&HtmlTagStack, &str, &str) -> Vec<HtmlEntity>;

#[derive(Clone)]
pub struct HanjaPhoneticization {
    pub phoneticizer: HanjaWordPhoneticizer,
    pub word_renderer: HanjaWordRenderer,
    pub homophone_renderer: HanjaWordRenderer,
    pub debug_comment: bool,
}

impl Default for HanjaPhoneticization {
    fn default() -> Self {
        Self {
            phoneticizer: phoneticize_hanja_word_with_initial_sound_law,
            word_renderer: hangul_only,
            homophone_renderer: hanja_in_parentheses,
            debug_comment: false,
        }
    }
}

pub fn hangul_only(stack: &HtmlTagStack, _hanja: &str, hangul: &str) -> Vec<HtmlEntity> {
    vec![HtmlEntity::Cdata {
        tag_stack: stack.clone(),
        text: hangul.to_string(),
    }]
}

pub fn hanja_in_parentheses(stack: &HtmlTagStack, hanja: &str, hangul: &str) -> Vec<HtmlEntity> {
    vec![HtmlEntity::Cdata {
        tag_stack: stack.clone(),
        text: format!("{hangul}({hanja})"),
    }]
}

pub fn hanja_in_ruby(stack: &HtmlTagStack, hanja: &str, hangul: &str) -> Vec<HtmlEntity> {
    let _ = (hanja, hangul);
    hangul_only(stack, hanja, hangul)
}

pub fn phoneticize_hanja(_cfg: &HanjaPhoneticization, entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    entities
}

pub fn phoneticize_hanja_word(word: &str) -> String {
    word.chars().map(phoneticize_hanja_char).collect()
}

pub fn phoneticize_hanja_word_with_initial_sound_law(word: &str) -> String {
    let mut chars: Vec<char> = phoneticize_hanja_word(word).chars().collect();
    if let Some(first) = chars.first_mut() {
        *first = convert_initial_sound_law(*first);
    }
    chars.into_iter().collect()
}

pub fn with_dictionary(
    dictionary: &HanjaDictionary,
    fallback: HanjaWordPhoneticizer,
    text: &str,
) -> String {
    if text.is_empty() {
        return String::new();
    }
    if let Some(value) = dictionary.get(text) {
        return value.clone();
    }
    fallback(text)
}

pub fn phoneticize_hanja_char(c: char) -> char {
    if let Some(readings) = k_hangul_data().get(&c)
        && let Some((&reading, _)) = readings.iter().next()
    {
        return reading;
    }
    c
}

pub fn initial_sound_law_table() -> BTreeMap<char, char> {
    BTreeMap::from([
        ('녀', '여'),
        ('뇨', '요'),
        ('뉴', '유'),
        ('니', '이'),
        ('랴', '야'),
        ('려', '여'),
        ('례', '예'),
        ('료', '요'),
        ('류', '유'),
        ('리', '이'),
        ('라', '나'),
        ('래', '내'),
        ('로', '노'),
        ('뢰', '뇌'),
        ('루', '누'),
        ('르', '느'),
    ])
}

pub fn convert_initial_sound_law(sound: char) -> char {
    let Some((without_batchim, final_jamo)) = without_batchim(sound) else {
        return sound;
    };
    let converted = initial_sound_law_table()
        .get(&without_batchim)
        .copied()
        .unwrap_or(without_batchim);
    with_batchim(converted, final_jamo).unwrap_or(sound)
}

pub fn revert_initial_sound_law(sound: char) -> BTreeSet<char> {
    let mut rev_table: BTreeMap<char, BTreeSet<char>> = BTreeMap::new();
    for (original, converted) in initial_sound_law_table() {
        rev_table.entry(converted).or_default().insert(original);
    }

    let Some((without_batchim, final_jamo)) = without_batchim(sound) else {
        return BTreeSet::new();
    };

    rev_table
        .get(&without_batchim)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|candidate| with_batchim(candidate, final_jamo))
        .collect()
}

fn without_batchim(hangul: char) -> Option<(char, Option<char>)> {
    let (initial, vowel, final_jamo) = to_jamo_triple(hangul)?;
    Some((from_jamo_triple((initial, vowel, None))?, final_jamo))
}

fn with_batchim(hangul: char, final_jamo: Option<char>) -> Option<char> {
    let (initial, vowel, _) = to_jamo_triple(hangul)?;
    from_jamo_triple((initial, vowel, final_jamo))
}
