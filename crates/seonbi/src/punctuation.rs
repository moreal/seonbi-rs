use std::collections::BTreeSet;

use crate::html::HtmlEntity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArrowTransformationOption {
    LeftRight,
    DoubleArrow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitationQuotes {
    pub title: (String, String),
    pub subtitle: (String, String),
    pub html_element: Option<(crate::html::HtmlTag, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuotePair {
    QuotePair(String, String),
    HtmlElement(crate::html::HtmlTag, String),
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
        html_element: Some((crate::html::HtmlTag::Cite, String::new())),
    }
}

pub fn corner_brackets() -> CitationQuotes {
    CitationQuotes {
        title: ("&#12302;".to_string(), "&#12303;".to_string()),
        subtitle: ("&#12300;".to_string(), "&#12301;".to_string()),
        html_element: Some((crate::html::HtmlTag::Cite, String::new())),
    }
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
        double_quotes: QuotePair::HtmlElement(crate::html::HtmlTag::Q, String::new()),
    }
}

pub fn vertical_corner_brackets_with_q() -> Quotes {
    Quotes {
        single_quotes: QuotePair::QuotePair("&#xfe41;".to_string(), "&#xfe42;".to_string()),
        double_quotes: QuotePair::HtmlElement(crate::html::HtmlTag::Q, String::new()),
    }
}

pub fn horizontal_corner_brackets_with_q() -> Quotes {
    Quotes {
        single_quotes: QuotePair::QuotePair("&#x300c;".to_string(), "&#x300d;".to_string()),
        double_quotes: QuotePair::HtmlElement(crate::html::HtmlTag::Q, String::new()),
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
    Stops {
        interpunct: "/".to_string(),
        ..horizontal_stops()
    }
}

pub fn quote_citation(_quotes: &CitationQuotes, entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    entities
}

pub fn transform_quote(_quotes: &Quotes, entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    entities
}

pub fn transform_arrow(
    _options: &BTreeSet<ArrowTransformationOption>,
    entities: Vec<HtmlEntity>,
) -> Vec<HtmlEntity> {
    entities
}

pub fn transform_ellipsis(entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    entities
}

pub fn transform_em_dash(entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    entities
}

pub fn normalize_stops(_stops: &Stops, entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    entities
}
