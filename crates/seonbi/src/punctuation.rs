use std::collections::BTreeSet;

use crate::html::{
    HtmlEntity, HtmlTag, annotate_with_lang, clip_text, is_never_korean, is_preserved_tag_stack,
    is_wrapped_by, is_wrapped_by_exact, normalize_text, wrap,
};
use crate::paired_transformer::{PairedTransformer, transform_pairs};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArrowTransformationOption {
    LeftRight,
    DoubleArrow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationQuotes {
    pub title: (String, String),
    pub subtitle: (String, String),
    pub html_element: Option<(HtmlTag, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuotePair {
    QuotePair(String, String),
    HtmlElement(HtmlTag, String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Quotes {
    pub single_quotes: QuotePair,
    pub double_quotes: QuotePair,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stops {
    pub period: String,
    pub comma: String,
    pub interpunct: String,
    pub question_mark: String,
    pub exclamation_mark: String,
}

pub fn angle_quotes() -> CitationQuotes {
    CitationQuotes {
        title: ("&#12298;".to_string(), "&#12299;".to_string()),
        subtitle: ("&#12296;".to_string(), "&#12297;".to_string()),
        html_element: Some((HtmlTag::Cite, String::new())),
    }
}

pub fn corner_brackets() -> CitationQuotes {
    CitationQuotes {
        title: ("&#12302;".to_string(), "&#12303;".to_string()),
        subtitle: ("&#12300;".to_string(), "&#12301;".to_string()),
        html_element: Some((HtmlTag::Cite, String::new())),
    }
}

pub fn quote_citation(quotes: &CitationQuotes, entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    let quotes = quotes.clone();
    let paired = PairedTransformer {
        ignores_tag_stack: Box::new(is_preserved_tag_stack),
        match_start: Box::new(|_, text| match_title_start(text)),
        match_end: Box::new(match_title_end),
        are_matches_paired: Box::new(|a, b| a == b),
        transform_pair: Box::new(move |start, _, buffer| {
            transform_title_pair(&quotes, start, buffer)
        }),
    };
    transform_pairs(&paired, entities)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TitlePunct {
    DoubleAngle,
    Angle,
    DoubleCorner,
    DoubleInequal,
    Inequal,
}

fn transform_title_pair(
    quotes: &CitationQuotes,
    punct: &TitlePunct,
    buffer: Vec<HtmlEntity>,
) -> Vec<HtmlEntity> {
    if buffer.len() < 2 {
        return Vec::new();
    }
    let middle = buffer[1..buffer.len() - 1].to_vec();

    let cited = if middle.is_empty() {
        Vec::new()
    } else if let Some((tag, attrs)) = &quotes.html_element {
        let base_stack = middle[0].tag_stack().clone();
        if attrs.is_empty() {
            if is_wrapped_by(&middle, *tag) {
                middle
            } else {
                wrap(&base_stack, *tag, attrs, middle)
            }
        } else if is_wrapped_by_exact(&middle, *tag, Some(attrs)) {
            middle
        } else {
            wrap(&base_stack, *tag, attrs, middle)
        }
    } else {
        middle
    };

    if cited.is_empty() {
        return Vec::new();
    }

    let stack = cited[0].tag_stack().clone();
    let (open, close) = match punct {
        TitlePunct::DoubleAngle | TitlePunct::DoubleCorner | TitlePunct::DoubleInequal => {
            (&quotes.title.0, &quotes.title.1)
        }
        _ => (&quotes.subtitle.0, &quotes.subtitle.1),
    };

    let mut out = Vec::with_capacity(cited.len() + 2);
    out.push(HtmlEntity::Text { tag_stack: stack.clone(), raw_text: open.clone() });
    out.extend(cited);
    out.push(HtmlEntity::Text { tag_stack: stack, raw_text: close.clone() });
    out
}

fn match_title_start(text: &str) -> Option<(TitlePunct, String, String, String)> {
    let candidates = [
        find_first(text, &["\u{300a}", "&#12298;", "&#x300a;"], true)
            .map(|(pre, tok, post)| (TitlePunct::DoubleAngle, pre, tok, post)),
        find_first(text, &["\u{300e}", "&#12302;", "&#x300e;"], true)
            .map(|(pre, tok, post)| (TitlePunct::DoubleCorner, pre, tok, post)),
        find_double(text, parse_lt)
            .map(|(pre, tok, post)| (TitlePunct::DoubleInequal, pre, tok, post)),
        find_first(text, &["\u{3008}", "&#12296;", "&#x3008;"], true)
            .map(|(pre, tok, post)| (TitlePunct::Angle, pre, tok, post)),
        find_single_parser(text, parse_lt)
            .map(|(pre, tok, post)| (TitlePunct::Inequal, pre, tok, post)),
    ];

    pick_earliest(candidates)
}

fn match_title_end(text: &str) -> Option<(TitlePunct, String, String, String)> {
    let candidates = [
        find_first(text, &["\u{300b}", "&#12299;", "&#x300b;"], true)
            .map(|(pre, tok, post)| (TitlePunct::DoubleAngle, pre, tok, post)),
        find_first(text, &["\u{300f}", "&#12303;", "&#x300f;"], true)
            .map(|(pre, tok, post)| (TitlePunct::DoubleCorner, pre, tok, post)),
        find_double(text, parse_gt)
            .map(|(pre, tok, post)| (TitlePunct::DoubleInequal, pre, tok, post)),
        find_first(text, &["\u{3009}", "&#12297;", "&#x3009;"], true)
            .map(|(pre, tok, post)| (TitlePunct::Angle, pre, tok, post)),
        find_single_parser(text, parse_gt)
            .map(|(pre, tok, post)| (TitlePunct::Inequal, pre, tok, post)),
    ];

    pick_earliest(candidates)
}

pub fn curved_quotes() -> Quotes {
    Quotes {
        single_quotes: QuotePair::QuotePair("&lsquo;".to_string(), "&rsquo;".to_string()),
        double_quotes: QuotePair::QuotePair("&ldquo;".to_string(), "&rdquo;".to_string()),
    }
}

pub fn vertical_corner_brackets() -> Quotes {
    Quotes {
        single_quotes: QuotePair::QuotePair("&#xfe41;".to_string(), "&#xfe42;".to_string()),
        double_quotes: QuotePair::QuotePair("&#xfe43;".to_string(), "&#xfe44;".to_string()),
    }
}

pub fn horizontal_corner_brackets() -> Quotes {
    Quotes {
        single_quotes: QuotePair::QuotePair("&#x300c;".to_string(), "&#x300d;".to_string()),
        double_quotes: QuotePair::QuotePair("&#x300e;".to_string(), "&#x300f;".to_string()),
    }
}

pub fn guillemets() -> Quotes {
    Quotes {
        single_quotes: QuotePair::QuotePair("&#x3008;".to_string(), "&#x3009;".to_string()),
        double_quotes: QuotePair::QuotePair("&#x300a;".to_string(), "&#x300b;".to_string()),
    }
}

pub fn curved_single_quotes_with_q() -> Quotes {
    Quotes {
        single_quotes: QuotePair::QuotePair("&lsquo;".to_string(), "&rsquo;".to_string()),
        double_quotes: QuotePair::HtmlElement(HtmlTag::Q, String::new()),
    }
}

pub fn vertical_corner_brackets_with_q() -> Quotes {
    Quotes {
        single_quotes: QuotePair::QuotePair("&#xfe41;".to_string(), "&#xfe42;".to_string()),
        double_quotes: QuotePair::HtmlElement(HtmlTag::Q, String::new()),
    }
}

pub fn horizontal_corner_brackets_with_q() -> Quotes {
    Quotes {
        single_quotes: QuotePair::QuotePair("&#x300c;".to_string(), "&#x300d;".to_string()),
        double_quotes: QuotePair::HtmlElement(HtmlTag::Q, String::new()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum QuotePunct {
    DoubleQuote,
    Apostrophe,
    OpeningSingleQuote,
    ClosingSingleQuote,
    OpeningDoubleQuote,
    ClosingDoubleQuote,
}

fn opens(p: QuotePunct) -> bool {
    matches!(
        p,
        QuotePunct::DoubleQuote
            | QuotePunct::Apostrophe
            | QuotePunct::OpeningSingleQuote
            | QuotePunct::OpeningDoubleQuote
    )
}

fn closes(p: QuotePunct) -> bool {
    matches!(
        p,
        QuotePunct::DoubleQuote
            | QuotePunct::Apostrophe
            | QuotePunct::ClosingSingleQuote
            | QuotePunct::ClosingDoubleQuote
    )
}

pub fn transform_quote(quotes: &Quotes, entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    let quotes = quotes.clone();
    let paired = PairedTransformer {
        ignores_tag_stack: Box::new(is_preserved_tag_stack),
        match_start: Box::new(|prev, text| match_quote_start(prev, text)),
        match_end: Box::new(match_quote_end),
        are_matches_paired: Box::new(|(a, _), (b, _)| are_quote_pairs(*a, *b)),
        transform_pair: Box::new(move |start, end, buffer| {
            transform_quote_pair(&quotes, start.0, &start.1, &end.1, buffer)
        }),
    };
    transform_pairs(&paired, entities)
}

fn are_quote_pairs(start: QuotePunct, end: QuotePunct) -> bool {
    match start {
        QuotePunct::OpeningSingleQuote => end == QuotePunct::ClosingSingleQuote,
        QuotePunct::OpeningDoubleQuote => end == QuotePunct::ClosingDoubleQuote,
        _ => start == end,
    }
}

fn quote_punctuations() -> Vec<(QuotePunct, Vec<&'static str>, bool)> {
    vec![
        (QuotePunct::Apostrophe, vec!["'", "&apos;", "&#39;", "&#x27;", "&#X27;"], false),
        (
            QuotePunct::DoubleQuote,
            vec!["\"", "&quot;", "&QUOT;", "&#34;", "&#x22;", "&#X22;"],
            false,
        ),
        (
            QuotePunct::OpeningSingleQuote,
            vec!["\u{2018}", "&lsquo;", "&OpenCurlyQuote;", "&#8216;", "&#x2018;", "&#X2018;"],
            false,
        ),
        (
            QuotePunct::ClosingSingleQuote,
            vec![
                "\u{2019}",
                "&rsquo;",
                "&rsquor;",
                "&CloseCurlyQuote;",
                "&#8217;",
                "&#x2019;",
                "&#X2019;",
            ],
            false,
        ),
        (
            QuotePunct::OpeningDoubleQuote,
            vec![
                "\u{201c}",
                "&ldquo;",
                "&OpenCurlyDoubleQuote;",
                "&#8220;",
                "&#x201c;",
                "&#x201C;",
                "&#X201c;",
                "&#X201C;",
            ],
            false,
        ),
        (
            QuotePunct::ClosingDoubleQuote,
            vec![
                "\u{201d}",
                "&rdquo;",
                "&rdquor;",
                "&CloseCurlyDoubleQuote;",
                "&#8221;",
                "&#x201d;",
                "&#x201D;",
                "&#X201d;",
                "&#X201D;",
            ],
            false,
        ),
    ]
}

fn match_quote_start(
    prev_matches: &[(QuotePunct, String)],
    text: &str,
) -> Option<((QuotePunct, String), String, String, String)> {
    let prev: BTreeSet<QuotePunct> = prev_matches.iter().map(|(p, _)| *p).collect();
    let mut candidates: Vec<(usize, QuotePunct, String)> = Vec::new();

    for (punct, entities, ci) in quote_punctuations() {
        if !opens(punct) || prev.contains(&punct) {
            continue;
        }
        for token in entities {
            if let Some((idx, len)) = find_token(text, token, ci) {
                candidates.push((idx, punct, text[idx..idx + len].to_string()));
            }
        }
    }

    let (idx, punct, token) = candidates.into_iter().min_by_key(|(idx, _, _)| *idx)?;
    if idx + token.len() >= text.len() {
        return None;
    }

    Some((
        (punct, token.clone()),
        text[..idx].to_string(),
        token.clone(),
        text[idx + token.len()..].to_string(),
    ))
}

fn match_quote_end(text: &str) -> Option<((QuotePunct, String), String, String, String)> {
    let mut candidates: Vec<(usize, QuotePunct, String)> = Vec::new();

    for (punct, entities, ci) in quote_punctuations() {
        if !closes(punct) {
            continue;
        }
        for token in entities {
            if let Some((idx, len)) = find_token(text, token, ci) {
                candidates.push((idx, punct, text[idx..idx + len].to_string()));
            }
        }
    }

    let (idx, punct, token) = candidates.into_iter().min_by_key(|(idx, _, _)| *idx)?;
    if idx + token.len() >= text.len() {
        return None;
    }

    Some((
        (punct, token.clone()),
        text[..idx].to_string(),
        token.clone(),
        text[idx + token.len()..].to_string(),
    ))
}

fn transform_quote_pair(
    quotes: &Quotes,
    punct: QuotePunct,
    start: &str,
    end: &str,
    buffer: Vec<HtmlEntity>,
) -> Vec<HtmlEntity> {
    let Some(middle) = clip_text(start, end, &buffer) else {
        return buffer;
    };

    let pair = match punct {
        QuotePunct::DoubleQuote
        | QuotePunct::OpeningDoubleQuote
        | QuotePunct::ClosingDoubleQuote => &quotes.double_quotes,
        _ => &quotes.single_quotes,
    };

    let Some(first) = middle.first() else {
        return middle;
    };
    let stack = first.tag_stack().clone();

    match pair {
        QuotePair::QuotePair(open, close) => {
            let mut out = Vec::with_capacity(middle.len() + 2);
            out.push(HtmlEntity::Text { tag_stack: stack.clone(), raw_text: open.clone() });
            out.extend(middle);
            out.push(HtmlEntity::Text { tag_stack: stack, raw_text: close.clone() });
            out
        }
        QuotePair::HtmlElement(tag, attrs) => wrap(&stack, *tag, attrs, middle),
    }
}

pub fn horizontal_stops() -> Stops {
    Stops {
        period: ". ".to_string(),
        comma: ", ".to_string(),
        interpunct: "·".to_string(),
        question_mark: "? ".to_string(),
        exclamation_mark: "! ".to_string(),
    }
}

pub fn vertical_stops() -> Stops {
    Stops {
        period: "。".to_string(),
        comma: "、".to_string(),
        interpunct: "·".to_string(),
        question_mark: "？".to_string(),
        exclamation_mark: "！".to_string(),
    }
}

pub fn horizontal_stops_with_slashes() -> Stops {
    Stops { interpunct: "/".to_string(), ..horizontal_stops() }
}

pub fn normalize_stops(stops: &Stops, input: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    let annotated = annotate_with_lang(normalize_text(input));
    annotated
        .into_iter()
        .map(|annot| match annot.entity {
            HtmlEntity::Text { tag_stack, raw_text } => {
                if is_preserved_tag_stack(&tag_stack) || is_never_korean(&annot.lang) {
                    HtmlEntity::Text { tag_stack, raw_text }
                } else {
                    HtmlEntity::Text { tag_stack, raw_text: replace_stops(stops, &raw_text) }
                }
            }
            other => other,
        })
        .collect()
}

#[derive(Debug)]
enum Ending {
    TrailingChars(String),
    TrailingSpaces(String),
    End,
}

fn replace_stops(stops: &Stops, txt: &str) -> String {
    let mut out = String::new();
    let mut pos = 0usize;

    while pos < txt.len() {
        if let Some((consumed, ending, repl)) = parse_stop(txt, pos, stops) {
            out.push_str(&to_entity(&adjust_ending(&ending, &repl)));
            pos += consumed;
            continue;
        }

        let ch = txt[pos..].chars().next().unwrap_or('\0');
        out.push(ch);
        pos += ch.len_utf8();
    }

    out
}

fn parse_stop(txt: &str, pos: usize, stops: &Stops) -> Option<(usize, Ending, String)> {
    if let Some((len, ending)) = parse_period(txt, pos) {
        return Some((len, ending, stops.period.clone()));
    }
    if let Some((len, ending)) = parse_comma(txt, pos) {
        return Some((len, ending, stops.comma.clone()));
    }
    if let Some((len, ending)) = parse_interpunct(txt, pos) {
        return Some((len, ending, stops.interpunct.clone()));
    }
    if let Some((len, ending)) = parse_question_mark(txt, pos) {
        return Some((len, ending, stops.question_mark.clone()));
    }
    if let Some((len, ending)) = parse_exclamation_mark(txt, pos) {
        return Some((len, ending, stops.exclamation_mark.clone()));
    }
    None
}

fn parse_period(txt: &str, pos: usize) -> Option<(usize, Ending)> {
    let boundary = [(".", false), ("&period;", false), ("&#46;", false), ("&#x2e;", true)];
    let trailing = [("。", false), ("&#12290;", false), ("&#x3002;", true)];

    for (tok, ci) in boundary {
        if let Some(len) = match_token_at(txt, pos, tok, ci)
            && let Some((ending, consumed_end)) = parse_boundary(txt, pos + len)
        {
            return Some((len + consumed_end, ending));
        }
    }
    for (tok, ci) in trailing {
        if let Some(len) = match_token_at(txt, pos, tok, ci) {
            let (ending, consumed_end) = parse_trailing_spaces(txt, pos + len);
            return Some((len + consumed_end, ending));
        }
    }
    None
}

fn parse_comma(txt: &str, pos: usize) -> Option<(usize, Ending)> {
    let boundary = [(",", false), ("&comma;", false), ("&#44;", false), ("&#x2c;", true)];
    let trailing = [("、", false), ("&#12289;", false), ("&#x3001;", true)];

    for (tok, ci) in boundary {
        if let Some(len) = match_token_at(txt, pos, tok, ci)
            && let Some((ending, consumed_end)) = parse_boundary(txt, pos + len)
        {
            return Some((len + consumed_end, ending));
        }
    }
    for (tok, ci) in trailing {
        if let Some(len) = match_token_at(txt, pos, tok, ci) {
            let (ending, consumed_end) = parse_trailing_spaces(txt, pos + len);
            return Some((len + consumed_end, ending));
        }
    }
    None
}

fn parse_interpunct(txt: &str, pos: usize) -> Option<(usize, Ending)> {
    let tokens = [
        ("·", false),
        ("&middot;", false),
        ("&centerdot;", false),
        ("&CenterDot;", false),
        ("&#183;", false),
        ("&#xb7;", true),
    ];
    for (tok, ci) in tokens {
        if let Some(len) = match_token_at(txt, pos, tok, ci) {
            return Some((len, Ending::End));
        }
    }
    None
}

fn parse_question_mark(txt: &str, pos: usize) -> Option<(usize, Ending)> {
    let boundary = [("?", false), ("&quest;", false), ("&#63;", false), ("&#x3f;", true)];
    let trailing = [("？", false), ("&#65311;", false), ("&#xff1f;", true)];

    for (tok, ci) in boundary {
        if let Some(len) = match_token_at(txt, pos, tok, ci)
            && let Some((ending, consumed_end)) = parse_boundary(txt, pos + len)
        {
            return Some((len + consumed_end, ending));
        }
    }
    for (tok, ci) in trailing {
        if let Some(len) = match_token_at(txt, pos, tok, ci) {
            let (ending, consumed_end) = parse_trailing_spaces(txt, pos + len);
            return Some((len + consumed_end, ending));
        }
    }
    None
}

fn parse_exclamation_mark(txt: &str, pos: usize) -> Option<(usize, Ending)> {
    let boundary = [("!", false), ("&excl;", false), ("&#33;", false), ("&#x21;", true)];
    let trailing = [("！", false), ("&#65281;", false), ("&#xff01;", true)];

    for (tok, ci) in boundary {
        if let Some(len) = match_token_at(txt, pos, tok, ci)
            && let Some((ending, consumed_end)) = parse_boundary(txt, pos + len)
        {
            return Some((len + consumed_end, ending));
        }
    }
    for (tok, ci) in trailing {
        if let Some(len) = match_token_at(txt, pos, tok, ci) {
            let (ending, consumed_end) = parse_trailing_spaces(txt, pos + len);
            return Some((len + consumed_end, ending));
        }
    }
    None
}

fn parse_boundary(txt: &str, pos: usize) -> Option<(Ending, usize)> {
    if pos >= txt.len() {
        return Some((Ending::End, 0));
    }
    if let Some((closing, len)) = parse_closing(txt, pos) {
        return Some((Ending::TrailingChars(closing), len));
    }
    if let Some((spaces, len)) = parse_spaces(txt, pos)
        && !spaces.is_empty()
    {
        return Some((Ending::TrailingSpaces(spaces), len));
    }
    None
}

fn parse_trailing_spaces(txt: &str, pos: usize) -> (Ending, usize) {
    parse_boundary(txt, pos).unwrap_or((Ending::TrailingSpaces(" ".to_string()), 0))
}

fn parse_closing(txt: &str, pos: usize) -> Option<(String, usize)> {
    const CLOSING_CHARS: [char; 22] = [
        '"', '”', '\'', '’', ')', ']', '}', '」', '』', '〉', '》', '）', '〕', '］', '｝', '｠',
        '】', '〗', '〙', '〛', '›', '»',
    ];
    const CLOSING_ENTITIES: [&str; 12] = [
        "&quot;",
        "&QUOT;",
        "&apos;",
        "&rpar;",
        "&rsqb;",
        "&rbrack;",
        "&rcub;",
        "&rbrace;",
        "&raquo;",
        "&rsquo;",
        "&rsquor;",
        "&CloseCurlyQuote;",
    ];
    const CLOSING_ENTITIES_2: [&str; 3] = ["&rdquo;", "&rdquor;", "&CloseCurlyDoubleQuote;"];

    if let Some(ch) = txt[pos..].chars().next()
        && CLOSING_CHARS.contains(&ch)
    {
        let len = ch.len_utf8();
        return Some((ch.to_string(), len));
    }

    for entity in CLOSING_ENTITIES {
        if let Some(len) = match_token_at(txt, pos, entity, false) {
            return Some((txt[pos..pos + len].to_string(), len));
        }
    }
    for entity in CLOSING_ENTITIES_2 {
        if let Some(len) = match_token_at(txt, pos, entity, false) {
            return Some((txt[pos..pos + len].to_string(), len));
        }
    }
    if let Some(len) = match_token_at(txt, pos, "&rsaquo;", false) {
        return Some((txt[pos..pos + len].to_string(), len));
    }

    for ch in CLOSING_CHARS {
        let decimal = format!("&#{};", ch as u32);
        if let Some(len) = match_token_at(txt, pos, &decimal, false) {
            return Some((txt[pos..pos + len].to_string(), len));
        }
        let hex = format!("&#x{:x};", ch as u32);
        if let Some(len) = match_token_at(txt, pos, &hex, true) {
            return Some((txt[pos..pos + len].to_string(), len));
        }
    }

    None
}

fn parse_spaces(txt: &str, pos: usize) -> Option<(String, usize)> {
    if pos >= txt.len() {
        return None;
    }
    let mut i = pos;
    while i < txt.len() {
        let ch = txt[i..].chars().next()?;
        if !ch.is_whitespace() {
            break;
        }
        i += ch.len_utf8();
    }
    if i == pos { None } else { Some((txt[pos..i].to_string(), i - pos)) }
}

fn adjust_ending(ending: &Ending, text: &str) -> String {
    if text.chars().last().is_some_and(char::is_whitespace) {
        let stripped = text.trim_end_matches(char::is_whitespace);
        match ending {
            Ending::TrailingChars(c) => format!("{stripped}{c}"),
            Ending::TrailingSpaces(s) => format!("{stripped}{s}"),
            Ending::End => stripped.to_string(),
        }
    } else {
        match ending {
            Ending::TrailingChars(c) => format!("{text}{c}"),
            _ => text.to_string(),
        }
    }
}

fn to_entity(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        if c.is_ascii() {
            out.push(c);
        } else {
            out.push_str(&format!("&#x{:x};", c as u32));
        }
    }
    out
}

pub fn transform_arrow(
    options: &BTreeSet<ArrowTransformationOption>,
    input: Vec<HtmlEntity>,
) -> Vec<HtmlEntity> {
    normalize_text(input)
        .into_iter()
        .map(|entity| match entity {
            HtmlEntity::Text { tag_stack, raw_text } => {
                if is_preserved_tag_stack(&tag_stack) {
                    HtmlEntity::Text { tag_stack, raw_text }
                } else {
                    HtmlEntity::Text { tag_stack, raw_text: replace_arrows(options, &raw_text) }
                }
            }
            other => other,
        })
        .collect()
}

fn replace_arrows(options: &BTreeSet<ArrowTransformationOption>, txt: &str) -> String {
    let mut out = String::new();
    let mut pos = 0usize;

    while pos < txt.len() {
        let mut matched: Option<(usize, &'static str)> = None;

        if options.contains(&ArrowTransformationOption::DoubleArrow)
            && options.contains(&ArrowTransformationOption::LeftRight)
            && let Some(len) = parse_lt_equals_gt(txt, pos)
        {
            matched = Some((len, "&hArr;"));
        }
        if matched.is_none()
            && options.contains(&ArrowTransformationOption::DoubleArrow)
            && let Some(len) = parse_lt_equals(txt, pos)
        {
            matched = Some((len, "&lArr;"));
        }
        if matched.is_none()
            && options.contains(&ArrowTransformationOption::DoubleArrow)
            && let Some(len) = parse_equals_gt(txt, pos)
        {
            matched = Some((len, "&rArr;"));
        }
        if matched.is_none()
            && options.contains(&ArrowTransformationOption::LeftRight)
            && let Some(len) = parse_lt_hyphen_gt(txt, pos)
        {
            matched = Some((len, "&harr;"));
        }
        if matched.is_none()
            && let Some(len) = parse_lt_hyphen(txt, pos)
        {
            matched = Some((len, "&larr;"));
        }
        if matched.is_none()
            && let Some(len) = parse_hyphen_gt(txt, pos)
        {
            matched = Some((len, "&rarr;"));
        }

        if let Some((len, repl)) = matched {
            out.push_str(repl);
            pos += len;
            continue;
        }

        let ch = txt[pos..].chars().next().unwrap_or('\0');
        out.push(ch);
        pos += ch.len_utf8();
    }

    out
}

pub fn transform_ellipsis(input: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    transform_text(input, |txt| replace_ellipsis(txt))
}

fn replace_ellipsis(txt: &str) -> String {
    let mut out = String::new();
    let mut pos = 0usize;

    while pos < txt.len() {
        if let Some(len1) = parse_period_token(txt, pos)
            && let Some(len2) = parse_period_token(txt, pos + len1)
            && let Some(len3) = parse_period_token(txt, pos + len1 + len2)
        {
            out.push_str("&hellip;");
            pos += len1 + len2 + len3;
            continue;
        }
        if let Some(len1) = parse_chinese_period_token(txt, pos)
            && let Some(len2) = parse_chinese_period_token(txt, pos + len1)
            && let Some(len3) = parse_chinese_period_token(txt, pos + len1 + len2)
        {
            out.push_str("&hellip;");
            pos += len1 + len2 + len3;
            continue;
        }

        let ch = txt[pos..].chars().next().unwrap_or('\0');
        out.push(ch);
        pos += ch.len_utf8();
    }

    out
}

pub fn transform_em_dash(input: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    transform_text(input, |txt| replace_em_dash(txt))
}

fn replace_em_dash(txt: &str) -> String {
    let mut out = String::new();
    let mut pos = 0usize;

    while pos < txt.len() {
        if let Some(len1) = parse_hyphen(txt, pos)
            && let Some(len2) = parse_hyphen(txt, pos + len1)
        {
            let len3 = parse_hyphen(txt, pos + len1 + len2).unwrap_or(0);
            out.push_str("&mdash;");
            pos += len1 + len2 + len3;
            continue;
        }

        if let Some((spaces1, len_spaces1)) = parse_spaces(txt, pos)
            && !spaces1.is_empty()
        {
            let pos2 = pos + len_spaces1;
            let mut symbol_len = parse_eu(txt, pos2);
            if symbol_len.is_none() {
                symbol_len = parse_hyphen(txt, pos2);
            }
            if let Some(len_symbol) = symbol_len
                && let Some((spaces2, len_spaces2)) = parse_spaces(txt, pos2 + len_symbol)
                && !spaces2.is_empty()
            {
                out.push_str("&mdash;");
                pos = pos2 + len_symbol + len_spaces2;
                continue;
            }
        }

        let ch = txt[pos..].chars().next().unwrap_or('\0');
        out.push(ch);
        pos += ch.len_utf8();
    }

    out
}

fn transform_text<F>(entities: Vec<HtmlEntity>, mut replace: F) -> Vec<HtmlEntity>
where
    F: FnMut(&str) -> String,
{
    entities
        .into_iter()
        .map(|entity| match entity {
            HtmlEntity::Text { tag_stack, raw_text } => {
                if is_preserved_tag_stack(&tag_stack) {
                    HtmlEntity::Text { tag_stack, raw_text }
                } else {
                    HtmlEntity::Text { tag_stack, raw_text: replace(&raw_text) }
                }
            }
            other => other,
        })
        .collect()
}

fn parse_lt(input: &str, pos: usize) -> Option<usize> {
    match_any_at(input, pos, &[("<", false), ("&lt;", false), ("&#60;", false), ("&#x3c;", true)])
}

fn parse_gt(input: &str, pos: usize) -> Option<usize> {
    match_any_at(input, pos, &[(">", false), ("&gt;", false), ("&#62;", false), ("&#x3e;", true)])
}

fn parse_hyphen(input: &str, pos: usize) -> Option<usize> {
    match_any_at(
        input,
        pos,
        &[("-", false), ("&hyphen;", false), ("&dash;", false), ("&#45;", false), ("&#x2d;", true)],
    )
}

fn parse_equals(input: &str, pos: usize) -> Option<usize> {
    match_any_at(
        input,
        pos,
        &[("=", false), ("&equals;", false), ("&61;", false), ("&#x3d;", true)],
    )
}

fn parse_period_token(input: &str, pos: usize) -> Option<usize> {
    match_any_at(
        input,
        pos,
        &[(".", false), ("&period;", false), ("&#46;", false), ("&#x2e;", true)],
    )
}

fn parse_chinese_period_token(input: &str, pos: usize) -> Option<usize> {
    match_any_at(input, pos, &[("。", false), ("&#12290;", false), ("&#x3002;", true)])
}

fn parse_eu(input: &str, pos: usize) -> Option<usize> {
    match_any_at(input, pos, &[("\u{3161}", false), ("&#12641;", false), ("&#x3161;", true)])
}

fn parse_lt_hyphen_gt(input: &str, pos: usize) -> Option<usize> {
    let l = parse_lt(input, pos)?;
    let h = parse_hyphen(input, pos + l)?;
    let g = parse_gt(input, pos + l + h)?;
    Some(l + h + g)
}

fn parse_lt_hyphen(input: &str, pos: usize) -> Option<usize> {
    let l = parse_lt(input, pos)?;
    let h = parse_hyphen(input, pos + l)?;
    Some(l + h)
}

fn parse_hyphen_gt(input: &str, pos: usize) -> Option<usize> {
    let h = parse_hyphen(input, pos)?;
    let g = parse_gt(input, pos + h)?;
    Some(h + g)
}

fn parse_lt_equals_gt(input: &str, pos: usize) -> Option<usize> {
    let l = parse_lt(input, pos)?;
    let e = parse_equals(input, pos + l)?;
    let g = parse_gt(input, pos + l + e)?;
    Some(l + e + g)
}

fn parse_lt_equals(input: &str, pos: usize) -> Option<usize> {
    let l = parse_lt(input, pos)?;
    let e = parse_equals(input, pos + l)?;
    Some(l + e)
}

fn parse_equals_gt(input: &str, pos: usize) -> Option<usize> {
    let e = parse_equals(input, pos)?;
    let g = parse_gt(input, pos + e)?;
    Some(e + g)
}

fn find_single_parser(
    text: &str,
    parser: fn(&str, usize) -> Option<usize>,
) -> Option<(String, String, String)> {
    let mut i = 0usize;
    while i < text.len() {
        if let Some(len) = parser(text, i) {
            let pre = text[..i].to_string();
            let tok = text[i..i + len].to_string();
            let post = text[i + len..].to_string();
            return Some((pre, tok, post));
        }
        i += text[i..].chars().next()?.len_utf8();
    }
    None
}

fn find_double(
    text: &str,
    parser: fn(&str, usize) -> Option<usize>,
) -> Option<(String, String, String)> {
    let mut i = 0usize;
    while i < text.len() {
        if let Some(len1) = parser(text, i)
            && let Some(len2) = parser(text, i + len1)
        {
            let len = len1 + len2;
            let pre = text[..i].to_string();
            let tok = text[i..i + len].to_string();
            let post = text[i + len..].to_string();
            return Some((pre, tok, post));
        }
        i += text[i..].chars().next()?.len_utf8();
    }
    None
}

fn find_first(text: &str, tokens: &[&str], ci_for_hex: bool) -> Option<(String, String, String)> {
    let mut best: Option<(usize, usize)> = None;

    for token in tokens {
        let ci = ci_for_hex && token.contains("&#x");
        if let Some((idx, len)) = find_token(text, token, ci) {
            if best.is_none_or(|(b, _)| idx < b) {
                best = Some((idx, len));
            }
        }
    }

    let (idx, len) = best?;
    Some((text[..idx].to_string(), text[idx..idx + len].to_string(), text[idx + len..].to_string()))
}

fn pick_earliest<T: Clone>(
    items: [Option<(T, String, String, String)>; 5],
) -> Option<(T, String, String, String)> {
    items.into_iter().flatten().min_by_key(|(_, pre, _, _)| pre.len())
}

fn match_any_at(input: &str, pos: usize, tokens: &[(&str, bool)]) -> Option<usize> {
    for (tok, ci) in tokens {
        if let Some(len) = match_token_at(input, pos, tok, *ci) {
            return Some(len);
        }
    }
    None
}

fn match_token_at(input: &str, pos: usize, token: &str, case_insensitive: bool) -> Option<usize> {
    let len = token.len();
    if pos + len > input.len() {
        return None;
    }
    if !input.is_char_boundary(pos) || !input.is_char_boundary(pos + len) {
        return None;
    }

    let s = &input[pos..pos + len];
    if case_insensitive {
        if eq_ascii_case_insensitive(s, token) { Some(len) } else { None }
    } else if s == token {
        Some(len)
    } else {
        None
    }
}

fn find_token(input: &str, token: &str, case_insensitive: bool) -> Option<(usize, usize)> {
    if !case_insensitive {
        return input.find(token).map(|idx| (idx, token.len()));
    }

    let t = token.as_bytes();
    if t.is_empty() {
        return None;
    }

    let bytes = input.as_bytes();
    if t.len() > bytes.len() {
        return None;
    }

    for i in 0..=bytes.len() - t.len() {
        if !input.is_char_boundary(i) || !input.is_char_boundary(i + t.len()) {
            continue;
        }
        if eq_ascii_case_insensitive_bytes(&bytes[i..i + t.len()], t) {
            return Some((i, t.len()));
        }
    }
    None
}

fn eq_ascii_case_insensitive(a: &str, b: &str) -> bool {
    eq_ascii_case_insensitive_bytes(a.as_bytes(), b.as_bytes())
}

fn eq_ascii_case_insensitive_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (x, y) in a.iter().zip(b.iter()) {
        if x.eq_ignore_ascii_case(y) {
            continue;
        }
        return false;
    }
    true
}
