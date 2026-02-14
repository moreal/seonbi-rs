use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use crate::content_types::{
    ContentType, ContentTypeError, content_type_from_text, content_type_text, content_types,
    transform_with_content_type,
};
use crate::hanja::{
    HanjaDictionary, HanjaPhoneticization, hangul_only, hanja_in_parentheses, hanja_in_ruby,
    phoneticize_hanja, phoneticize_hanja_word, phoneticize_hanja_word_with_initial_sound_law,
    with_dictionary,
};
use crate::html::HtmlEntity;
use crate::punctuation::{
    ArrowTransformationOption, angle_quotes, corner_brackets, curved_quotes,
    curved_single_quotes_with_q, guillemets, horizontal_corner_brackets,
    horizontal_corner_brackets_with_q, horizontal_stops, horizontal_stops_with_slashes,
    normalize_stops, quote_citation, transform_arrow, transform_ellipsis, transform_em_dash,
    transform_quote, vertical_corner_brackets, vertical_corner_brackets_with_q, vertical_stops,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuoteOption {
    CurvedQuotes,
    VerticalCornerBrackets,
    HorizontalCornerBrackets,
    Guillemets,
    CurvedSingleQuotesWithQ,
    VerticalCornerBracketsWithQ,
    HorizontalCornerBracketsWithQ,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CiteOption {
    AngleQuotes,
    CornerBrackets,
    AngleQuotesWithCite,
    CornerBracketsWithCite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrowOption {
    pub bidir_arrow: bool,
    pub double_arrow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StopOption {
    Horizontal,
    HorizontalWithSlashes,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HanjaRenderingOption {
    HangulOnly,
    HanjaInParentheses,
    DisambiguatingHanjaInParentheses,
    HanjaInRuby,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HanjaReadingOption {
    pub initial_sound_law: bool,
    pub dictionary: HanjaDictionary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HanjaOption {
    pub rendering: HanjaRenderingOption,
    pub reading: HanjaReadingOption,
}

#[derive(Clone)]
pub struct Configuration {
    pub debug_logger: Option<fn(&HtmlEntity)>,
    pub content_type: ContentType,
    pub quote: Option<QuoteOption>,
    pub cite: Option<CiteOption>,
    pub arrow: Option<ArrowOption>,
    pub ellipsis: bool,
    pub em_dash: bool,
    pub stop: Option<StopOption>,
    pub hanja: Option<HanjaOption>,
}

impl std::fmt::Debug for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Configuration")
            .field("debug_logger", &self.debug_logger.is_some())
            .field("content_type", &content_type_text(&self.content_type))
            .field("quote", &self.quote)
            .field("cite", &self.cite)
            .field("arrow", &self.arrow)
            .field("ellipsis", &self.ellipsis)
            .field("em_dash", &self.em_dash)
            .field("stop", &self.stop)
            .field("hanja", &self.hanja)
            .finish()
    }
}

pub fn transform_html_text(
    config: &Configuration,
    input: &str,
) -> Result<String, ContentTypeError> {
    transform_with_content_type(
        &config.content_type,
        |entities| to_transformer(config, entities),
        input,
    )
}

fn to_transformer(config: &Configuration, mut entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    if let Some(logger) = config.debug_logger {
        for entity in &entities {
            logger(entity);
        }
    }

    // Haskell's foldl (.) pipeline applies transformers in reverse declaration order.
    // Effective order must be: hanja -> em-dash -> ellipsis -> stop -> arrow -> cite -> quote.
    if let Some(hanja) = &config.hanja {
        let renderer = match hanja.rendering {
            HanjaRenderingOption::HangulOnly => hangul_only,
            HanjaRenderingOption::HanjaInParentheses => hanja_in_parentheses,
            HanjaRenderingOption::DisambiguatingHanjaInParentheses => hangul_only,
            HanjaRenderingOption::HanjaInRuby => hanja_in_ruby,
        };
        let homophone_renderer = match hanja.rendering {
            HanjaRenderingOption::HangulOnly => hangul_only,
            HanjaRenderingOption::HanjaInParentheses => hanja_in_parentheses,
            HanjaRenderingOption::DisambiguatingHanjaInParentheses => hanja_in_parentheses,
            HanjaRenderingOption::HanjaInRuby => hanja_in_ruby,
        };

        let fallback = if hanja.reading.initial_sound_law {
            phoneticize_hanja_word_with_initial_sound_law
        } else {
            phoneticize_hanja_word
        };
        let dictionary = hanja.reading.dictionary.clone();

        let cfg = HanjaPhoneticization {
            phoneticizer: Arc::new(move |word: &str| {
                if dictionary.is_empty() {
                    fallback(word)
                } else {
                    with_dictionary(&dictionary, &fallback, word)
                }
            }),
            word_renderer: Arc::new(renderer),
            homophone_renderer: Arc::new(homophone_renderer),
            debug_comment: false,
        };
        entities = phoneticize_hanja(&cfg, entities);
    }

    if config.em_dash {
        entities = transform_em_dash(entities);
    }
    if config.ellipsis {
        entities = transform_ellipsis(entities);
    }

    if let Some(stop) = &config.stop {
        let stops = match stop {
            StopOption::Horizontal => horizontal_stops(),
            StopOption::HorizontalWithSlashes => horizontal_stops_with_slashes(),
            StopOption::Vertical => vertical_stops(),
        };
        entities = normalize_stops(&stops, entities);
    }

    if let Some(arrow) = &config.arrow {
        let mut options = std::collections::BTreeSet::new();
        if arrow.bidir_arrow {
            options.insert(ArrowTransformationOption::LeftRight);
        }
        if arrow.double_arrow {
            options.insert(ArrowTransformationOption::DoubleArrow);
        }
        entities = transform_arrow(&options, entities);
    }

    if let Some(cite) = &config.cite {
        let mut quotes = match cite {
            CiteOption::AngleQuotes | CiteOption::AngleQuotesWithCite => angle_quotes(),
            CiteOption::CornerBrackets | CiteOption::CornerBracketsWithCite => corner_brackets(),
        };
        if matches!(cite, CiteOption::AngleQuotes | CiteOption::CornerBrackets) {
            quotes.html_element = None;
        }
        entities = quote_citation(&quotes, entities);
    }

    if let Some(quote) = &config.quote {
        entities = transform_quote(
            &match quote {
                QuoteOption::CurvedQuotes => curved_quotes(),
                QuoteOption::VerticalCornerBrackets => vertical_corner_brackets(),
                QuoteOption::HorizontalCornerBrackets => horizontal_corner_brackets(),
                QuoteOption::Guillemets => guillemets(),
                QuoteOption::CurvedSingleQuotesWithQ => curved_single_quotes_with_q(),
                QuoteOption::VerticalCornerBracketsWithQ => vertical_corner_brackets_with_q(),
                QuoteOption::HorizontalCornerBracketsWithQ => horizontal_corner_brackets_with_q(),
            },
            entities,
        );
    }

    entities
}

pub fn ko_kr() -> Configuration {
    Configuration {
        debug_logger: None,
        quote: Some(QuoteOption::CurvedQuotes),
        cite: Some(CiteOption::AngleQuotes),
        arrow: Some(ArrowOption { bidir_arrow: true, double_arrow: true }),
        ellipsis: true,
        em_dash: true,
        stop: Some(StopOption::Horizontal),
        hanja: Some(HanjaOption {
            rendering: HanjaRenderingOption::DisambiguatingHanjaInParentheses,
            reading: HanjaReadingOption {
                dictionary: south_korean_dictionary(),
                initial_sound_law: true,
            },
        }),
        content_type: ContentType::from("text/html"),
    }
}

pub fn ko_kp() -> Configuration {
    let mut config = ko_kr();
    config.quote = Some(QuoteOption::Guillemets);
    config.hanja = Some(HanjaOption {
        rendering: HanjaRenderingOption::HangulOnly,
        reading: HanjaReadingOption { dictionary: BTreeMap::new(), initial_sound_law: false },
    });
    config
}

pub fn presets() -> BTreeMap<String, Configuration> {
    BTreeMap::from([("ko-kp".to_string(), ko_kp()), ("ko-kr".to_string(), ko_kr())])
}

fn parse_dictionary_data(data: &str) -> HanjaDictionary {
    let mut dict = BTreeMap::new();
    for line in data.lines() {
        let mut columns = line.split('\t');
        if let (Some(hanja), Some(hangul)) = (columns.next(), columns.next()) {
            dict.insert(hanja.to_string(), hangul.to_string());
        }
    }
    dict
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_dictionary_file(path: &Path) -> Result<HanjaDictionary, std::io::Error> {
    let data = std::fs::read_to_string(path)?;
    Ok(parse_dictionary_data(&data))
}

#[cfg(target_arch = "wasm32")]
pub fn read_dictionary_file(_path: &Path) -> Result<HanjaDictionary, std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "read_dictionary_file is unavailable on wasm32 targets",
    ))
}

pub fn south_korean_dictionary() -> HanjaDictionary {
    let mut dict = {
        #[cfg(any(feature = "freeze-dict", target_arch = "wasm32"))]
        {
            parse_dictionary_data(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/ko-kr-stdict.tsv"
            )))
        }
        #[cfg(not(any(feature = "freeze-dict", target_arch = "wasm32")))]
        {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/ko-kr-stdict.tsv");
            read_dictionary_file(&path).unwrap_or_default()
        }
    };
    for (k, v) in builtin_dictionary() {
        dict.entry(k).or_insert(v);
    }
    dict
}

pub fn supported_content_types() -> std::collections::BTreeSet<ContentType> {
    content_types()
}

pub fn parse_content_type(text: &str) -> Option<ContentType> {
    content_type_from_text(text)
}

fn builtin_dictionary() -> HanjaDictionary {
    BTreeMap::from([
        ("困難".to_string(), "곤란".to_string()),
        ("國漢文混用體".to_string(), "국한문 혼용체".to_string()),
        ("大韓民國憲法".to_string(), "대한민국 헌법".to_string()),
        ("大韓民國臨時政府".to_string(), "대한민국 임시 정부".to_string()),
        ("臨時政府".to_string(), "임시 정부".to_string()),
        ("理念".to_string(), "이념".to_string()),
        ("國民投票".to_string(), "국민 투표".to_string()),
    ])
}
