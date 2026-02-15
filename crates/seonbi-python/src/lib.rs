use std::collections::BTreeMap;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use seonbi::{
    ArrowOption as InternalArrowOption, CiteOption as InternalCiteOption,
    Configuration as InternalConfiguration, HanjaOption as InternalHanjaOption,
    HanjaReadingOption as InternalHanjaReadingOption,
    HanjaRenderingOption as InternalHanjaRenderingOption, QuoteOption as InternalQuoteOption,
    StopOption as InternalStopOption, parse_content_type, presets, south_korean_dictionary,
    supported_content_types, transform_html_text,
};

#[derive(Debug)]
struct ConfigurationInput {
    content_type: Option<String>,
    content_type_camel: Option<String>,
    preset: Option<String>,
    quote: Option<String>,
    cite: Option<String>,
    arrow: Option<ArrowInput>,
    ellipsis: Option<bool>,
    em_dash: Option<bool>,
    em_dash_camel: Option<bool>,
    stop: Option<String>,
    hanja: Option<HanjaInput>,
}

impl ConfigurationInput {
    fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref().or(self.content_type_camel.as_deref())
    }

    fn em_dash(&self) -> Option<bool> {
        self.em_dash.or(self.em_dash_camel)
    }
}

#[derive(Debug)]
struct ArrowInput {
    bidir_arrow: Option<bool>,
    bidir_arrow_camel: Option<bool>,
    double_arrow: Option<bool>,
    double_arrow_camel: Option<bool>,
}

impl ArrowInput {
    fn bidir_arrow(&self) -> bool {
        self.bidir_arrow.or(self.bidir_arrow_camel).unwrap_or(false)
    }

    fn double_arrow(&self) -> bool {
        self.double_arrow.or(self.double_arrow_camel).unwrap_or(false)
    }
}

#[derive(Debug)]
struct HanjaInput {
    rendering: String,
    reading: HanjaReadingInput,
}

#[derive(Debug)]
struct HanjaReadingInput {
    initial_sound_law: Option<bool>,
    initial_sound_law_camel: Option<bool>,
    dictionary: Option<BTreeMap<String, String>>,
    use_dictionaries: Option<Vec<String>>,
    use_dictionaries_camel: Option<Vec<String>>,
}

impl HanjaReadingInput {
    fn initial_sound_law(&self) -> bool {
        self.initial_sound_law.or(self.initial_sound_law_camel).unwrap_or(false)
    }

    fn dictionary(&self) -> BTreeMap<String, String> {
        self.dictionary.clone().unwrap_or_default()
    }

    fn use_dictionaries(&self) -> Vec<String> {
        self.use_dictionaries.clone().or(self.use_dictionaries_camel.clone()).unwrap_or_default()
    }
}

fn optional_string(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<String>> {
    match dict.get_item(key)? {
        Some(value) if !value.is_none() => value.extract().map(Some),
        _ => Ok(None),
    }
}

fn optional_bool(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<bool>> {
    match dict.get_item(key)? {
        Some(value) if !value.is_none() => value.extract().map(Some),
        _ => Ok(None),
    }
}

fn optional_dict(
    dict: &Bound<'_, PyDict>,
    key: &str,
) -> PyResult<Option<BTreeMap<String, String>>> {
    match dict.get_item(key)? {
        Some(value) if !value.is_none() => value.extract().map(Some),
        _ => Ok(None),
    }
}

fn optional_string_vec(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<Option<Vec<String>>> {
    match dict.get_item(key)? {
        Some(value) if !value.is_none() => value.extract().map(Some),
        _ => Ok(None),
    }
}

fn optional_arrow(dict: &Bound<'_, PyDict>) -> PyResult<Option<ArrowInput>> {
    match dict.get_item("arrow")? {
        Some(value) if !value.is_none() => parse_arrow_input(&value).map(Some),
        _ => Ok(None),
    }
}

fn optional_hanja(dict: &Bound<'_, PyDict>) -> PyResult<Option<HanjaInput>> {
    match dict.get_item("hanja")? {
        Some(value) if !value.is_none() => parse_hanja_input(&value).map(Some),
        _ => Ok(None),
    }
}

fn parse_configuration_input(config: &Bound<'_, PyAny>) -> PyResult<ConfigurationInput> {
    let dict = config.cast::<PyDict>()?;
    Ok(ConfigurationInput {
        content_type: optional_string(dict, "content_type")?,
        content_type_camel: optional_string(dict, "contentType")?,
        preset: optional_string(dict, "preset")?,
        quote: optional_string(dict, "quote")?,
        cite: optional_string(dict, "cite")?,
        arrow: optional_arrow(dict)?,
        ellipsis: optional_bool(dict, "ellipsis")?,
        em_dash: optional_bool(dict, "em_dash")?,
        em_dash_camel: optional_bool(dict, "emDash")?,
        stop: optional_string(dict, "stop")?,
        hanja: optional_hanja(dict)?,
    })
}

fn parse_arrow_input(value: &Bound<'_, PyAny>) -> PyResult<ArrowInput> {
    let dict = value.cast::<PyDict>()?;
    Ok(ArrowInput {
        bidir_arrow: optional_bool(dict, "bidir_arrow")?,
        bidir_arrow_camel: optional_bool(dict, "bidirArrow")?,
        double_arrow: optional_bool(dict, "double_arrow")?,
        double_arrow_camel: optional_bool(dict, "doubleArrow")?,
    })
}

fn parse_hanja_input(value: &Bound<'_, PyAny>) -> PyResult<HanjaInput> {
    let dict = value.cast::<PyDict>()?;
    let rendering = optional_string(dict, "rendering")?
        .ok_or_else(|| PyValueError::new_err("hanja.rendering is required"))?;

    let reading = match dict.get_item("reading")? {
        Some(value) if !value.is_none() => parse_hanja_reading_input(&value)?,
        _ => HanjaReadingInput {
            initial_sound_law: None,
            initial_sound_law_camel: None,
            dictionary: None,
            use_dictionaries: None,
            use_dictionaries_camel: None,
        },
    };

    Ok(HanjaInput { rendering, reading })
}

fn parse_hanja_reading_input(value: &Bound<'_, PyAny>) -> PyResult<HanjaReadingInput> {
    let dict = value.cast::<PyDict>()?;
    Ok(HanjaReadingInput {
        initial_sound_law: optional_bool(dict, "initial_sound_law")?,
        initial_sound_law_camel: optional_bool(dict, "initialSoundLaw")?,
        dictionary: optional_dict(dict, "dictionary")?,
        use_dictionaries: optional_string_vec(dict, "use_dictionaries")?,
        use_dictionaries_camel: optional_string_vec(dict, "useDictionaries")?,
    })
}

#[pyfunction]
fn transform(config: &Bound<'_, PyAny>, input: &str) -> PyResult<String> {
    let config_input = parse_configuration_input(config)?;
    let internal = to_internal_config(config_input)?;
    transform_html_text(&internal, input).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
fn ko_kr(py: Python<'_>) -> PyResult<Py<PyDict>> {
    from_internal_config(py, seonbi::ko_kr(), Some("ko-kr"))
}

#[pyfunction]
fn ko_kp(py: Python<'_>) -> PyResult<Py<PyDict>> {
    from_internal_config(py, seonbi::ko_kp(), Some("ko-kp"))
}

fn to_internal_config(config: ConfigurationInput) -> PyResult<InternalConfiguration> {
    let em_dash = config.em_dash();

    let mut internal = if let Some(preset_name) = config.preset.as_deref() {
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

    if let Some(content_type_text) = config.content_type() {
        internal.content_type = parse_content_type(content_type_text)
            .ok_or_else(|| invalid_content_type_error(content_type_text))?;
    }

    if let Some(quote) = config.quote.as_deref() {
        internal.quote = Some(parse_quote_option(quote)?);
    }
    if let Some(cite) = config.cite.as_deref() {
        internal.cite = Some(parse_cite_option(cite)?);
    }
    if let Some(arrow) = config.arrow {
        internal.arrow = Some(InternalArrowOption {
            bidir_arrow: arrow.bidir_arrow(),
            double_arrow: arrow.double_arrow(),
        });
    }
    if let Some(ellipsis) = config.ellipsis {
        internal.ellipsis = ellipsis;
    }
    if let Some(em_dash) = em_dash {
        internal.em_dash = em_dash;
    }
    if let Some(stop) = config.stop.as_deref() {
        internal.stop = Some(parse_stop_option(stop)?);
    }
    if let Some(hanja) = config.hanja {
        internal.hanja = Some(to_internal_hanja_option(hanja)?);
    }

    internal.debug_logger = None;
    Ok(internal)
}

fn to_internal_hanja_option(option: HanjaInput) -> PyResult<InternalHanjaOption> {
    let mut dictionary = option.reading.dictionary();
    for dict_id in option.reading.use_dictionaries() {
        match dict_id.as_str() {
            "kr-stdict" => {
                for (k, v) in south_korean_dictionary() {
                    dictionary.entry(k).or_insert(v);
                }
            }
            _ => return Err(PyValueError::new_err(format!("No such dictionary ID: {dict_id}"))),
        }
    }

    Ok(InternalHanjaOption {
        rendering: parse_hanja_rendering_option(&option.rendering)?,
        reading: InternalHanjaReadingOption {
            initial_sound_law: option.reading.initial_sound_law(),
            dictionary,
        },
    })
}

fn preset_by_name(name: &str) -> PyResult<InternalConfiguration> {
    let preset_key = name.to_ascii_lowercase().replace('_', "-");
    let preset_map = presets();
    preset_map.get(&preset_key).cloned().ok_or_else(|| {
        PyValueError::new_err(format!(
            "No such preset: {name}; available presets: {}",
            preset_map.keys().cloned().collect::<Vec<_>>().join(", ")
        ))
    })
}

fn invalid_content_type_error(value: &str) -> PyErr {
    let available = supported_content_types()
        .into_iter()
        .map(|v| v.as_str().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    PyValueError::new_err(format!(
        "Invalid content type: {value}; available content types: {available}"
    ))
}

fn from_internal_config(
    py: Python<'_>,
    config: InternalConfiguration,
    preset: Option<&str>,
) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("content_type", config.content_type.as_str())?;
    if let Some(preset) = preset {
        dict.set_item("preset", preset)?;
    }
    if let Some(quote) = config.quote {
        dict.set_item("quote", quote_option_to_text(quote))?;
    }
    if let Some(cite) = config.cite {
        dict.set_item("cite", cite_option_to_text(cite))?;
    }
    if let Some(stop) = config.stop {
        dict.set_item("stop", stop_option_to_text(stop))?;
    }
    dict.set_item("ellipsis", config.ellipsis)?;
    dict.set_item("em_dash", config.em_dash)?;

    if let Some(arrow) = config.arrow {
        let arrow_dict = PyDict::new(py);
        arrow_dict.set_item("bidir_arrow", arrow.bidir_arrow)?;
        arrow_dict.set_item("double_arrow", arrow.double_arrow)?;
        dict.set_item("arrow", arrow_dict)?;
    }

    if let Some(hanja) = config.hanja {
        let hanja_dict = PyDict::new(py);
        hanja_dict.set_item("rendering", hanja_rendering_option_to_text(hanja.rendering))?;

        let reading_dict = PyDict::new(py);
        reading_dict.set_item("initial_sound_law", hanja.reading.initial_sound_law)?;
        let use_dictionaries = if hanja.reading.dictionary.is_empty() {
            Vec::<String>::new()
        } else {
            vec!["kr-stdict".to_string()]
        };
        reading_dict.set_item("use_dictionaries", use_dictionaries)?;

        hanja_dict.set_item("reading", reading_dict)?;
        dict.set_item("hanja", hanja_dict)?;
    }

    Ok(dict.unbind())
}

fn parse_quote_option(value: &str) -> PyResult<InternalQuoteOption> {
    match normalize_token(value).as_str() {
        "curvedquotes" => Ok(InternalQuoteOption::CurvedQuotes),
        "verticalcornerbrackets" => Ok(InternalQuoteOption::VerticalCornerBrackets),
        "horizontalcornerbrackets" => Ok(InternalQuoteOption::HorizontalCornerBrackets),
        "guillemets" => Ok(InternalQuoteOption::Guillemets),
        "curvedsinglequoteswithq" => Ok(InternalQuoteOption::CurvedSingleQuotesWithQ),
        "verticalcornerbracketswithq" => Ok(InternalQuoteOption::VerticalCornerBracketsWithQ),
        "horizontalcornerbracketswithq" => Ok(InternalQuoteOption::HorizontalCornerBracketsWithQ),
        _ => Err(PyValueError::new_err(format!("cannot parse value `{value}` as quote option"))),
    }
}

fn parse_cite_option(value: &str) -> PyResult<InternalCiteOption> {
    match normalize_token(value).as_str() {
        "anglequotes" => Ok(InternalCiteOption::AngleQuotes),
        "cornerbrackets" => Ok(InternalCiteOption::CornerBrackets),
        "anglequoteswithcite" => Ok(InternalCiteOption::AngleQuotesWithCite),
        "cornerbracketswithcite" => Ok(InternalCiteOption::CornerBracketsWithCite),
        _ => Err(PyValueError::new_err(format!("cannot parse value `{value}` as cite option"))),
    }
}

fn parse_stop_option(value: &str) -> PyResult<InternalStopOption> {
    match normalize_token(value).as_str() {
        "horizontal" => Ok(InternalStopOption::Horizontal),
        "horizontalwithslashes" => Ok(InternalStopOption::HorizontalWithSlashes),
        "vertical" => Ok(InternalStopOption::Vertical),
        _ => Err(PyValueError::new_err(format!("cannot parse value `{value}` as stop option"))),
    }
}

fn parse_hanja_rendering_option(value: &str) -> PyResult<InternalHanjaRenderingOption> {
    match normalize_token(value).as_str() {
        "hangulonly" => Ok(InternalHanjaRenderingOption::HangulOnly),
        "hanjainparentheses" => Ok(InternalHanjaRenderingOption::HanjaInParentheses),
        "disambiguatinghanjainparentheses" => {
            Ok(InternalHanjaRenderingOption::DisambiguatingHanjaInParentheses)
        }
        "hanjainruby" => Ok(InternalHanjaRenderingOption::HanjaInRuby),
        _ => Err(PyValueError::new_err(format!(
            "cannot parse value `{value}` as hanja rendering option"
        ))),
    }
}

fn quote_option_to_text(value: InternalQuoteOption) -> &'static str {
    match value {
        InternalQuoteOption::CurvedQuotes => "curved-quotes",
        InternalQuoteOption::VerticalCornerBrackets => "vertical-corner-brackets",
        InternalQuoteOption::HorizontalCornerBrackets => "horizontal-corner-brackets",
        InternalQuoteOption::Guillemets => "guillemets",
        InternalQuoteOption::CurvedSingleQuotesWithQ => "curved-single-quotes-with-q",
        InternalQuoteOption::VerticalCornerBracketsWithQ => "vertical-corner-brackets-with-q",
        InternalQuoteOption::HorizontalCornerBracketsWithQ => "horizontal-corner-brackets-with-q",
    }
}

fn cite_option_to_text(value: InternalCiteOption) -> &'static str {
    match value {
        InternalCiteOption::AngleQuotes => "angle-quotes",
        InternalCiteOption::CornerBrackets => "corner-brackets",
        InternalCiteOption::AngleQuotesWithCite => "angle-quotes-with-cite",
        InternalCiteOption::CornerBracketsWithCite => "corner-brackets-with-cite",
    }
}

fn stop_option_to_text(value: InternalStopOption) -> &'static str {
    match value {
        InternalStopOption::Horizontal => "horizontal",
        InternalStopOption::HorizontalWithSlashes => "horizontal-with-slashes",
        InternalStopOption::Vertical => "vertical",
    }
}

fn hanja_rendering_option_to_text(value: InternalHanjaRenderingOption) -> &'static str {
    match value {
        InternalHanjaRenderingOption::HangulOnly => "hangul-only",
        InternalHanjaRenderingOption::HanjaInParentheses => "hanja-in-parentheses",
        InternalHanjaRenderingOption::DisambiguatingHanjaInParentheses => {
            "disambiguating-hanja-in-parentheses"
        }
        InternalHanjaRenderingOption::HanjaInRuby => "hanja-in-ruby",
    }
}

fn normalize_token(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_alphanumeric()).flat_map(char::to_lowercase).collect()
}

#[pymodule]
fn _seonbi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(transform, m)?)?;
    m.add_function(wrap_pyfunction!(ko_kr, m)?)?;
    m.add_function(wrap_pyfunction!(ko_kp, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_token_accepts_multiple_naming_styles() {
        assert_eq!(normalize_token("curved-quotes"), "curvedquotes");
        assert_eq!(normalize_token("curved_quotes"), "curvedquotes");
        assert_eq!(normalize_token("CurvedQuotes"), "curvedquotes");
    }
}
