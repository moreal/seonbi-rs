use std::collections::BTreeMap;
use std::io::{Error, ErrorKind};
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

fn parse_dictionary_data(data: &str) -> Result<HanjaDictionary, Error> {
    let mut dict = BTreeMap::new();
    for (line_no, line) in data.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }
        let mut columns = line.splitn(3, '\t');
        let hanja = columns.next().unwrap_or_default();
        let Some(hangul) = columns.next() else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "invalid dictionary TSV at line {}: expected two tab-separated columns",
                    line_no + 1
                ),
            ));
        };
        if columns.next().is_some() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "invalid dictionary TSV at line {}: expected exactly two columns",
                    line_no + 1
                ),
            ));
        }
        if hanja.is_empty() || hangul.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "invalid dictionary TSV at line {}: empty hanja or hangul column",
                    line_no + 1
                ),
            ));
        }
        dict.insert(hanja.to_string(), hangul.to_string());
    }
    Ok(dict)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_dictionary_file(path: &Path) -> Result<HanjaDictionary, std::io::Error> {
    let data = std::fs::read_to_string(path)?;
    parse_dictionary_data(&data)
}

#[cfg(target_arch = "wasm32")]
pub fn read_dictionary_file(path: &Path) -> Result<HanjaDictionary, std::io::Error> {
    let _ = path;
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "reading dictionary files is not supported on wasm32",
    ))
}

/// Loads South Korean Hanja readings from `data/ko-kr-stdict.tsv`.
///
/// The dataset is large and typically tracked via Git LFS. If LFS assets are not
/// present in a local checkout, this file may contain an unresolved pointer text
/// and parsing fails. In that case this returns an empty dictionary, matching
/// the original Haskell implementation's fail-safe behavior.
pub fn south_korean_dictionary() -> HanjaDictionary {
    {
        #[cfg(any(feature = "freeze-dict", target_arch = "wasm32"))]
        {
            parse_dictionary_data(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/data/ko-kr-stdict.tsv"
            )))
            .unwrap_or_default()
        }
        #[cfg(not(any(feature = "freeze-dict", target_arch = "wasm32")))]
        {
            let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/ko-kr-stdict.tsv");
            read_dictionary_file(&path).unwrap_or_default()
        }
    }
}

pub fn supported_content_types() -> std::collections::BTreeSet<ContentType> {
    content_types()
}

pub fn parse_content_type(text: &str) -> Option<ContentType> {
    content_type_from_text(text)
}

#[cfg(test)]
mod tests {
    use super::parse_dictionary_data;

    #[test]
    fn dictionary_parser_rejects_lfs_pointer_text() {
        let pointer = "version https://git-lfs.github.com/spec/v1\n\
oid sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\n\
size 1234\n";
        let err = parse_dictionary_data(pointer).expect_err("LFS pointer must be rejected");
        assert!(err.to_string().contains("invalid dictionary TSV at line 1"));
    }

    #[test]
    fn dictionary_parser_requires_exactly_two_columns() {
        let err = parse_dictionary_data("漢字\t한자\textra\n").expect_err("invalid row");
        assert!(err.to_string().contains("expected exactly two columns"));
    }

    #[test]
    fn dictionary_parser_parses_valid_tsv() {
        let dict = parse_dictionary_data("漢字\t한자\n孫文\t쑨원\n").expect("valid dictionary");
        assert_eq!(dict.get("漢字"), Some(&"한자".to_string()));
        assert_eq!(dict.get("孫文"), Some(&"쑨원".to_string()));
    }
}
