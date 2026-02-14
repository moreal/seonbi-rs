use std::collections::BTreeMap;

use napi::Error;
use napi_derive::napi;
use seonbi::{
    ArrowOption as InternalArrowOption, CiteOption as InternalCiteOption,
    Configuration as InternalConfiguration, HanjaOption as InternalHanjaOption,
    HanjaReadingOption as InternalHanjaReadingOption,
    HanjaRenderingOption as InternalHanjaRenderingOption, QuoteOption as InternalQuoteOption,
    StopOption as InternalStopOption, parse_content_type, presets, south_korean_dictionary,
    supported_content_types, transform_html_text,
};

#[napi(string_enum)]
pub enum QuoteOption {
    CurvedQuotes,
    VerticalCornerBrackets,
    HorizontalCornerBrackets,
    Guillemets,
    CurvedSingleQuotesWithQ,
    VerticalCornerBracketsWithQ,
    HorizontalCornerBracketsWithQ,
}

#[napi(string_enum)]
pub enum CiteOption {
    AngleQuotes,
    CornerBrackets,
    AngleQuotesWithCite,
    CornerBracketsWithCite,
}

#[napi(string_enum)]
pub enum StopOption {
    Horizontal,
    HorizontalWithSlashes,
    Vertical,
}

#[napi(string_enum)]
pub enum HanjaRenderingOption {
    HangulOnly,
    HanjaInParentheses,
    DisambiguatingHanjaInParentheses,
    HanjaInRuby,
}

#[napi(object)]
pub struct ArrowOption {
    #[napi(js_name = "bidirArrow")]
    pub bidir_arrow: bool,
    #[napi(js_name = "doubleArrow")]
    pub double_arrow: bool,
}

#[napi(object)]
pub struct HanjaReadingOption {
    #[napi(js_name = "initialSoundLaw")]
    pub initial_sound_law: bool,
    #[napi(js_name = "useDictionaries")]
    pub use_dictionaries: Vec<String>,
}

#[napi(object)]
pub struct HanjaOption {
    pub rendering: HanjaRenderingOption,
    pub reading: HanjaReadingOption,
}

#[napi(object)]
pub struct Configuration {
    #[napi(js_name = "contentType")]
    pub content_type: Option<String>,
    pub preset: Option<String>,
    pub quote: Option<QuoteOption>,
    pub cite: Option<CiteOption>,
    pub arrow: Option<ArrowOption>,
    pub ellipsis: Option<bool>,
    #[napi(js_name = "emDash")]
    pub em_dash: Option<bool>,
    pub stop: Option<StopOption>,
    pub hanja: Option<HanjaOption>,
}

#[napi]
pub fn transform(config: Configuration, input: String) -> napi::Result<String> {
    let internal_config = to_internal_config(config)?;
    transform_html_text(&internal_config, &input).map_err(|e| Error::from_reason(e.to_string()))
}

#[napi(js_name = "koKr")]
pub fn ko_kr() -> Configuration {
    from_internal_config(seonbi::ko_kr(), Some("ko-kr"))
}

#[napi(js_name = "koKp")]
pub fn ko_kp() -> Configuration {
    from_internal_config(seonbi::ko_kp(), Some("ko-kp"))
}

fn to_internal_config(config: Configuration) -> napi::Result<InternalConfiguration> {
    let mut internal = if let Some(preset_name) = config.preset.as_ref() {
        preset_by_name(preset_name)?
    } else {
        InternalConfiguration {
            debug_logger: None,
            content_type: parse_content_type("text/html")
                .expect("text/html must be a supported content type"),
            quote: None,
            cite: None,
            arrow: None,
            ellipsis: false,
            em_dash: false,
            stop: None,
            hanja: None,
        }
    };

    if let Some(content_type_text) = config.content_type {
        internal.content_type = parse_content_type(&content_type_text)
            .ok_or_else(|| invalid_content_type_error(&content_type_text))?;
    }

    if let Some(quote) = config.quote {
        internal.quote = Some(quote.into());
    }
    if let Some(cite) = config.cite {
        internal.cite = Some(cite.into());
    }
    if let Some(arrow) = config.arrow {
        internal.arrow = Some(arrow.into());
    }
    if let Some(ellipsis) = config.ellipsis {
        internal.ellipsis = ellipsis;
    }
    if let Some(em_dash) = config.em_dash {
        internal.em_dash = em_dash;
    }
    if let Some(stop) = config.stop {
        internal.stop = Some(stop.into());
    }
    if let Some(hanja) = config.hanja {
        internal.hanja = Some(to_internal_hanja_option(hanja)?);
    }

    internal.debug_logger = None;
    Ok(internal)
}

fn to_internal_hanja_option(option: HanjaOption) -> napi::Result<InternalHanjaOption> {
    let mut dictionary = BTreeMap::new();
    for dict_id in option.reading.use_dictionaries {
        match dict_id.as_str() {
            "kr-stdict" => {
                for (k, v) in south_korean_dictionary() {
                    dictionary.entry(k).or_insert(v);
                }
            }
            _ => return Err(Error::from_reason(format!("No such dictionary ID: {dict_id}"))),
        }
    }

    Ok(InternalHanjaOption {
        rendering: option.rendering.into(),
        reading: InternalHanjaReadingOption {
            initial_sound_law: option.reading.initial_sound_law,
            dictionary,
        },
    })
}

fn preset_by_name(name: &str) -> napi::Result<InternalConfiguration> {
    let preset_key = name.to_ascii_lowercase().replace('_', "-");
    let preset_map = presets();
    preset_map.get(&preset_key).cloned().ok_or_else(|| {
        Error::from_reason(format!(
            "No such preset: {name}; available presets: {}",
            preset_map.keys().cloned().collect::<Vec<_>>().join(", ")
        ))
    })
}

fn invalid_content_type_error(value: &str) -> Error {
    let available = supported_content_types()
        .into_iter()
        .map(|v| v.as_str().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Error::from_reason(format!(
        "Invalid content type: {value}; available content types: {available}"
    ))
}

fn from_internal_config(config: InternalConfiguration, preset: Option<&str>) -> Configuration {
    Configuration {
        content_type: Some(config.content_type.as_str().to_string()),
        preset: preset.map(ToString::to_string),
        quote: config.quote.map(Into::into),
        cite: config.cite.map(Into::into),
        arrow: config.arrow.map(Into::into),
        ellipsis: Some(config.ellipsis),
        em_dash: Some(config.em_dash),
        stop: config.stop.map(Into::into),
        hanja: config.hanja.map(from_internal_hanja_option),
    }
}

fn from_internal_hanja_option(option: InternalHanjaOption) -> HanjaOption {
    let use_dictionaries = if option.reading.dictionary.is_empty() {
        Vec::new()
    } else {
        vec!["kr-stdict".to_string()]
    };

    HanjaOption {
        rendering: option.rendering.into(),
        reading: HanjaReadingOption {
            initial_sound_law: option.reading.initial_sound_law,
            use_dictionaries,
        },
    }
}

impl From<QuoteOption> for InternalQuoteOption {
    fn from(value: QuoteOption) -> Self {
        match value {
            QuoteOption::CurvedQuotes => Self::CurvedQuotes,
            QuoteOption::VerticalCornerBrackets => Self::VerticalCornerBrackets,
            QuoteOption::HorizontalCornerBrackets => Self::HorizontalCornerBrackets,
            QuoteOption::Guillemets => Self::Guillemets,
            QuoteOption::CurvedSingleQuotesWithQ => Self::CurvedSingleQuotesWithQ,
            QuoteOption::VerticalCornerBracketsWithQ => Self::VerticalCornerBracketsWithQ,
            QuoteOption::HorizontalCornerBracketsWithQ => Self::HorizontalCornerBracketsWithQ,
        }
    }
}

impl From<InternalQuoteOption> for QuoteOption {
    fn from(value: InternalQuoteOption) -> Self {
        match value {
            InternalQuoteOption::CurvedQuotes => Self::CurvedQuotes,
            InternalQuoteOption::VerticalCornerBrackets => Self::VerticalCornerBrackets,
            InternalQuoteOption::HorizontalCornerBrackets => Self::HorizontalCornerBrackets,
            InternalQuoteOption::Guillemets => Self::Guillemets,
            InternalQuoteOption::CurvedSingleQuotesWithQ => Self::CurvedSingleQuotesWithQ,
            InternalQuoteOption::VerticalCornerBracketsWithQ => Self::VerticalCornerBracketsWithQ,
            InternalQuoteOption::HorizontalCornerBracketsWithQ => {
                Self::HorizontalCornerBracketsWithQ
            }
        }
    }
}

impl From<CiteOption> for InternalCiteOption {
    fn from(value: CiteOption) -> Self {
        match value {
            CiteOption::AngleQuotes => Self::AngleQuotes,
            CiteOption::CornerBrackets => Self::CornerBrackets,
            CiteOption::AngleQuotesWithCite => Self::AngleQuotesWithCite,
            CiteOption::CornerBracketsWithCite => Self::CornerBracketsWithCite,
        }
    }
}

impl From<InternalCiteOption> for CiteOption {
    fn from(value: InternalCiteOption) -> Self {
        match value {
            InternalCiteOption::AngleQuotes => Self::AngleQuotes,
            InternalCiteOption::CornerBrackets => Self::CornerBrackets,
            InternalCiteOption::AngleQuotesWithCite => Self::AngleQuotesWithCite,
            InternalCiteOption::CornerBracketsWithCite => Self::CornerBracketsWithCite,
        }
    }
}

impl From<StopOption> for InternalStopOption {
    fn from(value: StopOption) -> Self {
        match value {
            StopOption::Horizontal => Self::Horizontal,
            StopOption::HorizontalWithSlashes => Self::HorizontalWithSlashes,
            StopOption::Vertical => Self::Vertical,
        }
    }
}

impl From<InternalStopOption> for StopOption {
    fn from(value: InternalStopOption) -> Self {
        match value {
            InternalStopOption::Horizontal => Self::Horizontal,
            InternalStopOption::HorizontalWithSlashes => Self::HorizontalWithSlashes,
            InternalStopOption::Vertical => Self::Vertical,
        }
    }
}

impl From<HanjaRenderingOption> for InternalHanjaRenderingOption {
    fn from(value: HanjaRenderingOption) -> Self {
        match value {
            HanjaRenderingOption::HangulOnly => Self::HangulOnly,
            HanjaRenderingOption::HanjaInParentheses => Self::HanjaInParentheses,
            HanjaRenderingOption::DisambiguatingHanjaInParentheses => {
                Self::DisambiguatingHanjaInParentheses
            }
            HanjaRenderingOption::HanjaInRuby => Self::HanjaInRuby,
        }
    }
}

impl From<InternalHanjaRenderingOption> for HanjaRenderingOption {
    fn from(value: InternalHanjaRenderingOption) -> Self {
        match value {
            InternalHanjaRenderingOption::HangulOnly => Self::HangulOnly,
            InternalHanjaRenderingOption::HanjaInParentheses => Self::HanjaInParentheses,
            InternalHanjaRenderingOption::DisambiguatingHanjaInParentheses => {
                Self::DisambiguatingHanjaInParentheses
            }
            InternalHanjaRenderingOption::HanjaInRuby => Self::HanjaInRuby,
        }
    }
}

impl From<ArrowOption> for InternalArrowOption {
    fn from(value: ArrowOption) -> Self {
        Self { bidir_arrow: value.bidir_arrow, double_arrow: value.double_arrow }
    }
}

impl From<InternalArrowOption> for ArrowOption {
    fn from(value: InternalArrowOption) -> Self {
        Self { bidir_arrow: value.bidir_arrow, double_arrow: value.double_arrow }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_without_preset_uses_defaults() {
        let config = Configuration {
            content_type: None,
            preset: None,
            quote: None,
            cite: None,
            arrow: None,
            ellipsis: None,
            em_dash: None,
            stop: None,
            hanja: None,
        };

        let parsed = to_internal_config(config).expect("must parse default config");
        assert_eq!(parsed.content_type.as_str(), "text/html");
        assert!(!parsed.ellipsis);
        assert!(!parsed.em_dash);
    }

    #[test]
    fn config_with_preset_and_overrides() {
        let config = Configuration {
            content_type: Some("text/plain".to_string()),
            preset: Some("ko-kr".to_string()),
            quote: Some(QuoteOption::Guillemets),
            cite: None,
            arrow: None,
            ellipsis: Some(false),
            em_dash: None,
            stop: None,
            hanja: None,
        };

        let parsed = to_internal_config(config).expect("must parse preset config");
        assert_eq!(parsed.content_type.as_str(), "text/plain");
        assert!(matches!(parsed.quote, Some(InternalQuoteOption::Guillemets)));
        assert!(!parsed.ellipsis);
    }

    #[test]
    fn rejects_unknown_dictionary_id() {
        let result = to_internal_hanja_option(HanjaOption {
            rendering: HanjaRenderingOption::HangulOnly,
            reading: HanjaReadingOption {
                initial_sound_law: false,
                use_dictionaries: vec!["nope".to_string()],
            },
        });

        assert!(result.is_err());
    }
}
