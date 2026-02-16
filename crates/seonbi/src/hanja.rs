use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::hangul::{from_jamo_triple, to_jamo_triple};
use crate::html::{
    HtmlEntity, HtmlTag, HtmlTagStack, annotate_with_lang, is_never_korean, is_preserved_tag_stack,
};
use crate::unihan::k_hangul::k_hangul_data;

pub type HanjaDictionary = BTreeMap<String, String>;
pub type HanjaWordPhoneticizer = Arc<dyn Fn(&str) -> String + Send + Sync>;
pub type HanjaWordRenderer =
    Arc<dyn Fn(&HtmlTagStack, &str, &str) -> Vec<HtmlEntity> + Send + Sync>;

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
            phoneticizer: Arc::new(phoneticize_hanja_word_with_initial_sound_law),
            word_renderer: Arc::new(hangul_only),
            homophone_renderer: Arc::new(hanja_in_parentheses),
            debug_comment: false,
        }
    }
}

pub fn def() -> HanjaPhoneticization {
    HanjaPhoneticization::default()
}

pub fn hangul_only(stack: &HtmlTagStack, _hanja: &str, hangul: &str) -> Vec<HtmlEntity> {
    vec![HtmlEntity::Cdata { tag_stack: stack.clone(), text: hangul.to_string() }]
}

pub fn hanja_in_parentheses(stack: &HtmlTagStack, hanja: &str, hangul: &str) -> Vec<HtmlEntity> {
    vec![HtmlEntity::Cdata { tag_stack: stack.clone(), text: format!("{hangul}({hanja})") }]
}

pub fn hanja_in_ruby(stack: &HtmlTagStack, hanja: &str, hangul: &str) -> Vec<HtmlEntity> {
    let ruby_stack = stack.push(HtmlTag::Ruby);
    vec![
        HtmlEntity::StartTag {
            tag_stack: stack.clone(),
            tag: HtmlTag::Ruby,
            raw_attributes: String::new(),
        },
        HtmlEntity::Cdata { tag_stack: ruby_stack.clone(), text: hanja.to_string() },
        HtmlEntity::StartTag {
            tag_stack: ruby_stack.clone(),
            tag: HtmlTag::RP,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text { tag_stack: ruby_stack.push(HtmlTag::RP), raw_text: "(".to_string() },
        HtmlEntity::EndTag { tag_stack: ruby_stack.clone(), tag: HtmlTag::RP },
        HtmlEntity::StartTag {
            tag_stack: ruby_stack.clone(),
            tag: HtmlTag::RT,
            raw_attributes: String::new(),
        },
        HtmlEntity::Cdata { tag_stack: ruby_stack.push(HtmlTag::RT), text: hangul.to_string() },
        HtmlEntity::EndTag { tag_stack: ruby_stack.clone(), tag: HtmlTag::RT },
        HtmlEntity::StartTag {
            tag_stack: ruby_stack.clone(),
            tag: HtmlTag::RP,
            raw_attributes: String::new(),
        },
        HtmlEntity::Text { tag_stack: ruby_stack.push(HtmlTag::RP), raw_text: ")".to_string() },
        HtmlEntity::EndTag { tag_stack: ruby_stack.clone(), tag: HtmlTag::RP },
        HtmlEntity::EndTag { tag_stack: stack.clone(), tag: HtmlTag::Ruby },
    ]
}

pub fn phoneticize_hanja(cfg: &HanjaPhoneticization, entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    let mut normalized: Vec<EitherEntity> = Vec::new();

    for annotated in annotate_with_lang(entities) {
        match annotated.entity {
            HtmlEntity::Text { tag_stack, raw_text } => {
                if is_preserved_tag_stack(&tag_stack) || is_never_korean(&annotated.lang) {
                    normalized.push(EitherEntity::Left(HtmlEntity::Text { tag_stack, raw_text }));
                    continue;
                }

                match analyze_hanja_text(&raw_text) {
                    None => normalized
                        .push(EitherEntity::Left(HtmlEntity::Text { tag_stack, raw_text })),
                    Some(pairs) => {
                        for (is_hanja, text) in pairs {
                            if !is_hanja {
                                normalized.push(EitherEntity::Left(HtmlEntity::Text {
                                    tag_stack: tag_stack.clone(),
                                    raw_text: text,
                                }));
                            } else {
                                let hangul = (cfg.phoneticizer)(&text);
                                normalized.push(EitherEntity::Right((
                                    tag_stack.clone(),
                                    text,
                                    hangul,
                                )));
                            }
                        }
                    }
                }
            }
            other => normalized.push(EitherEntity::Left(other)),
        }
    }

    let mut expanded: Vec<EitherEntity> = Vec::new();
    for entry in normalized {
        match entry {
            EitherEntity::Left(entity) => expanded.push(EitherEntity::Left(entity)),
            EitherEntity::Right((stack, hanja, hangul)) => {
                let hanja_parts = split_by_digits(&hanja);
                let hangul_parts = split_by_digits(&hangul);
                if hanja_parts.len() != hangul_parts.len() {
                    expanded.push(EitherEntity::Right((stack, hanja, hangul)));
                    continue;
                }

                for (hanj, hang) in hanja_parts.into_iter().zip(hangul_parts) {
                    if hanj.chars().any(|c| c.is_ascii_digit()) {
                        expanded.push(EitherEntity::Left(HtmlEntity::Text {
                            tag_stack: stack.clone(),
                            raw_text: hanj,
                        }));
                    } else {
                        expanded.push(EitherEntity::Right((stack.clone(), hanj, hang)));
                    }
                }
            }
        }
    }

    let mut frequency: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for entry in &expanded {
        if let EitherEntity::Right((_, hanja, hangul)) = entry {
            frequency.entry(hangul.clone()).or_default().insert(hanja.clone());
        }
    }

    let mut out = Vec::new();
    for entry in expanded {
        match entry {
            EitherEntity::Left(entity) => out.push(entity),
            EitherEntity::Right((stack, hanja, hangul)) => {
                let ambiguous = frequency.get(&hangul).map(|set| set.len() > 1).unwrap_or(false);
                let renderer = if ambiguous { &cfg.homophone_renderer } else { &cfg.word_renderer };

                if cfg.debug_comment {
                    out.push(HtmlEntity::Comment {
                        tag_stack: stack.clone(),
                        comment: format!(" Hanja: {hanja}"),
                    });
                }
                out.extend(renderer(&stack, &hanja, &hangul));
                if cfg.debug_comment {
                    out.push(HtmlEntity::Comment {
                        tag_stack: stack,
                        comment: " /Hanja ".to_string(),
                    });
                }
            }
        }
    }

    out
}

enum EitherEntity {
    Left(HtmlEntity),
    Right((HtmlTagStack, String, String)),
}

fn split_by_digits(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut prev_is_digit: Option<bool> = None;

    for ch in text.chars() {
        let now = ch.is_ascii_digit();
        if let Some(prev) = prev_is_digit
            && prev != now
        {
            result.push(std::mem::take(&mut current));
        }
        current.push(ch);
        prev_is_digit = Some(now);
    }

    if !current.is_empty() {
        result.push(current);
    }
    result
}

pub fn phoneticize_hanja_word(word: &str) -> String {
    word.chars().map(phoneticize_hanja_char).collect()
}

pub fn phoneticize_hanja_word_with_initial_sound_law(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = word.chars().collect();
    let mut i = 0usize;
    let mut out = String::new();

    while i < chars.len() {
        if let Some((s, consumed)) = parse_yeol_yul(&chars, i) {
            out.push_str(&s);
            i += consumed;
            continue;
        }
        if let Some((s, consumed)) = parse_prefixed_number(&chars, i) {
            out.push_str(&s);
            i += consumed;
            continue;
        }
        if let Some((s, consumed)) = parse_han_number(&chars, i) {
            out.push_str(&s);
            i += consumed;
            continue;
        }

        out.push(phoneticize_hanja_char(chars[i]));
        i += 1;
    }

    let mut out_chars: Vec<char> = out.chars().collect();
    if let Some(first) = out_chars.first_mut() {
        *first = convert_initial_sound_law(*first);
    }
    out_chars.into_iter().collect()
}

fn parse_yeol_yul(chars: &[char], i: usize) -> Option<(String, usize)> {
    if i + 1 >= chars.len() {
        return None;
    }
    let former = chars[i];
    let later = phoneticize_hanja_char(chars[i + 1]);

    if later != '렬' && later != '률' {
        return None;
    }

    let former_phone = phoneticize_hanja_char(former);
    let has_target_batchim = match to_jamo_triple(former_phone) {
        Some((_, _, final_jamo)) => final_jamo.is_none() || final_jamo == Some('\u{11ab}'),
        None => false,
    };
    if !has_target_batchim {
        return None;
    }

    Some((format!("{}{}", former_phone, convert_initial_sound_law(later)), 2))
}

fn parse_prefixed_number(chars: &[char], i: usize) -> Option<(String, usize)> {
    if chars.get(i) != Some(&'第') || i + 1 >= chars.len() {
        return None;
    }
    if !is_han_digit(chars[i + 1]) {
        return None;
    }

    let mut j = i + 1;
    while j < chars.len() && is_han_digit(chars[j]) {
        j += 1;
    }

    let mut out = String::new();
    out.push(phoneticize_hanja_char('第'));
    for c in &chars[i + 1..j] {
        out.push(convert_initial_sound_law(phoneticize_digit(*c)));
    }

    Some((out, j - i))
}

fn parse_han_number(chars: &[char], i: usize) -> Option<(String, usize)> {
    if i + 1 >= chars.len() || !is_han_digit(chars[i]) || !is_han_digit(chars[i + 1]) {
        return None;
    }

    let mut j = i;
    while j < chars.len() && is_han_digit(chars[j]) {
        j += 1;
    }

    let out: String =
        chars[i..j].iter().map(|c| convert_initial_sound_law(phoneticize_digit(*c))).collect();
    Some((out, j - i))
}

fn phoneticize_digit(c: char) -> char {
    match c {
        '參' | '叁' | '参' | '叄' => '삼',
        '拾' => '십',
        _ => phoneticize_hanja_char(c),
    }
}

fn is_han_digit(c: char) -> bool {
    "零一壹壱弌夁二貳贰弐弍貮三參叁参弎叄四肆䦉五伍六陸陆陸七柒漆八捌九玖十拾百佰陌千仟阡萬万億兆京垓秭穰溝澗"
        .contains(c)
}

pub fn with_dictionary<F>(dictionary: &HanjaDictionary, fallback: &F, text: &str) -> String
where
    F: Fn(&str) -> String,
{
    if text.is_empty() {
        return String::new();
    }

    let char_len = text.chars().count();
    let mut matches = Vec::new();

    for pos in 0..=char_len {
        let (unmatched, wd) = split_at_chars(text, pos);
        let mut pattern = wd.to_string();
        while !pattern.is_empty() {
            if let Some(matched) = dictionary.get(&pattern) {
                let rest = &wd[pattern.len()..];
                matches.push((format!("{}{}", fallback(unmatched), matched), rest.to_string()));
                break;
            }
            pattern.pop();
        }
    }

    if matches.is_empty() {
        return fallback(text);
    }

    let (replaced, rest) = matches.remove(0);
    if rest.is_empty() {
        replaced
    } else {
        replaced + &with_dictionary(dictionary, fallback, &rest)
    }
}

fn split_at_chars(text: &str, n: usize) -> (&str, &str) {
    if n == 0 {
        return ("", text);
    }
    let mut idx = text.len();
    for (count, (i, _)) in text.char_indices().enumerate() {
        if count == n {
            idx = i;
            break;
        }
    }
    text.split_at(idx)
}

pub fn phoneticize_hanja_char(c: char) -> char {
    let Some(readings) = k_hangul_data().get(&c) else {
        return c;
    };

    let Some((&sound, _)) = readings.iter().min_by_key(|(_, cite)| (*cite).clone()) else {
        return c;
    };

    let reverted = revert_initial_sound_law(sound)
        .into_iter()
        .filter(|candidate| readings.contains_key(candidate))
        .collect::<BTreeSet<_>>();

    reverted.into_iter().next().unwrap_or(sound)
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
    let Some((pattern, final_jamo)) = without_batchim(sound) else {
        return sound;
    };
    let converted = initial_sound_law_table().get(&pattern).copied().unwrap_or(pattern);
    with_batchim(converted, final_jamo).unwrap_or(sound)
}

pub fn revert_initial_sound_law(sound: char) -> BTreeSet<char> {
    let mut reverse: BTreeMap<char, BTreeSet<char>> = BTreeMap::new();
    for (orig, converted) in initial_sound_law_table() {
        reverse.entry(converted).or_default().insert(orig);
    }

    let Some((pattern, final_jamo)) = without_batchim(sound) else {
        return BTreeSet::new();
    };

    reverse
        .get(&pattern)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|c| with_batchim(c, final_jamo))
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

fn analyze_hanja_text(text: &str) -> Option<Vec<(bool, String)>> {
    let mut tokens: Vec<(char, String)> = Vec::new();
    let mut i = 0usize;

    while i < text.len() {
        if let Some((ch, consumed)) = parse_numeric_char_ref(&text[i..]) {
            tokens.push((ch, text[i..i + consumed].to_string()));
            i += consumed;
            continue;
        }

        let ch = text[i..].chars().next()?;
        tokens.push((ch, ch.to_string()));
        i += ch.len_utf8();
    }

    if tokens.is_empty() {
        return None;
    }

    let mut grouped: Vec<(bool, String)> = Vec::new();
    let mut current_kind: Option<bool> = None;
    let mut current_text = String::new();

    for (ch, s) in tokens {
        let is_hanja_or_digit = ch.is_ascii_digit() || is_hanja(ch);
        match current_kind {
            Some(kind) if kind == is_hanja_or_digit => current_text.push_str(&s),
            Some(kind) => {
                grouped.push((kind, std::mem::take(&mut current_text)));
                current_text.push_str(&s);
                current_kind = Some(is_hanja_or_digit);
            }
            None => {
                current_kind = Some(is_hanja_or_digit);
                current_text.push_str(&s);
            }
        }
    }

    if let Some(kind) = current_kind
        && !current_text.is_empty()
    {
        grouped.push((kind, current_text));
    }

    Some(grouped)
}

fn parse_numeric_char_ref(input: &str) -> Option<(char, usize)> {
    let rest = input.strip_prefix("&#")?;
    let mut idx = 0usize;
    let mut hex = false;
    let mut chars = rest.chars();

    if let Some(c) = chars.next()
        && (c == 'x' || c == 'X')
    {
        hex = true;
        idx += c.len_utf8();
    }

    let digits_start = idx;
    while idx < rest.len() {
        let ch = rest[idx..].chars().next()?;
        if ch == ';' {
            break;
        }
        let valid = if hex { ch.is_ascii_hexdigit() } else { ch.is_ascii_digit() };
        if !valid {
            return None;
        }
        idx += ch.len_utf8();
    }

    if idx == digits_start || idx >= rest.len() || !rest[idx..].starts_with(';') {
        return None;
    }

    let num_str = &rest[digits_start..idx];
    let codepoint =
        if hex { u32::from_str_radix(num_str, 16).ok()? } else { num_str.parse::<u32>().ok()? };
    let ch = char::from_u32(codepoint)?;

    Some((ch, 2 + idx + 1))
}

fn is_hanja(c: char) -> bool {
    ('\u{2f00}'..='\u{2fff}').contains(&c)
        || c == '\u{3007}'
        || ('\u{3400}'..='\u{4dbf}').contains(&c)
        || ('\u{4e00}'..='\u{9fcc}').contains(&c)
        || ('\u{f900}'..='\u{faff}').contains(&c)
        || ('\u{20000}'..='\u{2a6d6}').contains(&c)
        || ('\u{2a700}'..='\u{2b734}').contains(&c)
        || ('\u{2b740}'..='\u{2b81d}').contains(&c)
        || ('\u{2b820}'..='\u{2cea1}').contains(&c)
        || ('\u{2ceb0}'..='\u{2ebe0}').contains(&c)
        || ('\u{2f800}'..='\u{2fa1f}').contains(&c)
}
