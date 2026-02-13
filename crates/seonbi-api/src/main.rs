use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::process::ExitCode;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::header::{
    ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE, SERVER, VARY,
};
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::{Json, Router};
use clap::Parser;
use seonbi::{
    ArrowOption, CiteOption, Configuration, HanjaOption, HanjaReadingOption, HanjaRenderingOption,
    QuoteOption, StopOption, parse_content_type, presets, south_korean_dictionary,
    supported_content_types, transform_html_text,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(name = "seonbi-api")]
#[command(about = "Seonbi HTTP API server")]
struct Args {
    #[arg(short = 'H', long, default_value = "*")]
    host: String,
    #[arg(short = 'p', long, default_value_t = 3800)]
    port: u16,
    #[arg(short = 'o', long = "allow-origin")]
    allow_origin: Option<String>,
    #[arg(long = "debug-delay", default_value_t = 0)]
    debug_delay: u64,
}

#[derive(Clone)]
struct AppState {
    allow_origin: Option<HeaderValue>,
    debug_delay_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiInput {
    content: Option<String>,
    source_html: Option<String>,
    preset: Option<String>,
    content_type: Option<String>,
    xhtml: Option<bool>,
    quote: Option<String>,
    cite: Option<String>,
    arrow: Option<ArrowInput>,
    ellipsis: Option<bool>,
    em_dash: Option<bool>,
    stop: Option<String>,
    hanja: Option<HanjaInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArrowInput {
    #[serde(default)]
    bidir: bool,
    #[serde(default)]
    double: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HanjaInput {
    rendering: String,
    reading: HanjaReadingInput,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HanjaReadingInput {
    #[serde(default)]
    initial_sound_law: bool,
    #[serde(default)]
    dictionary: BTreeMap<String, String>,
    #[serde(default)]
    use_dictionaries: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SuccessResponse {
    success: bool,
    content: String,
    warnings: Vec<String>,
    content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result_html: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    success: bool,
    message: String,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    let allow_origin = match args.allow_origin {
        Some(origin) => match HeaderValue::from_str(&origin) {
            Ok(v) => Some(v),
            Err(err) => {
                eprintln!("invalid allow-origin header value: {err}");
                return ExitCode::FAILURE;
            }
        },
        None => None,
    };

    let state = AppState { allow_origin, debug_delay_ms: args.debug_delay };
    let app = router(state);

    let host = normalize_host_for_bind(&args.host);
    let addr: SocketAddr = match format!("{host}:{}", args.port).parse() {
        Ok(addr) => addr,
        Err(err) => {
            eprintln!("invalid host/port: {err}");
            return ExitCode::FAILURE;
        }
    };

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("failed to bind: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) = axum::serve(listener, app).await {
        eprintln!("server error: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn router(state: AppState) -> Router {
    Router::new().route("/", any(handle_root)).fallback(any(handle_root)).with_state(state)
}

async fn handle_root(State(state): State<AppState>, method: Method, body: Bytes) -> Response {
    match method {
        Method::POST => handle_post(state, body).await,
        Method::OPTIONS => handle_options(state).await,
        _ => handle_method_not_allowed(state, method).await,
    }
}

async fn handle_post(state: AppState, body: Bytes) -> Response {
    if state.debug_delay_ms > 0 {
        tokio::time::sleep(std::time::Duration::from_millis(state.debug_delay_ms)).await;
    }

    let input: ApiInput = match serde_json::from_slice(&body) {
        Ok(input) => input,
        Err(err) => {
            return json_response(
                &state,
                StatusCode::BAD_REQUEST,
                &ErrorResponse { success: false, message: err.to_string() },
            );
        }
    };

    let parsed = match parse_input(input) {
        Ok(parsed) => parsed,
        Err(msg) => {
            return json_response(
                &state,
                StatusCode::BAD_REQUEST,
                &ErrorResponse { success: false, message: msg },
            );
        }
    };

    let ParsedInput { content, config, warnings } = parsed;
    let transformed = match transform_html_text(&config, &content) {
        Ok(result) => result,
        Err(err) => {
            return json_response(
                &state,
                StatusCode::BAD_REQUEST,
                &ErrorResponse { success: false, message: err.to_string() },
            );
        }
    };

    let ctype = config.content_type.as_str().to_string();
    let result_html = if ctype == "text/html" || ctype == "application/xhtml+xml" {
        Some(apply_warning_comments(&warnings, &transformed))
    } else {
        None
    };
    let response = SuccessResponse {
        success: true,
        content: transformed,
        warnings,
        content_type: ctype,
        result_html,
    };
    json_response(&state, StatusCode::OK, &response)
}

async fn handle_options(state: AppState) -> Response {
    json_response(&state, StatusCode::OK, &serde_json::Value::Null)
}

async fn handle_method_not_allowed(state: AppState, method: Method) -> Response {
    json_response(
        &state,
        StatusCode::METHOD_NOT_ALLOWED,
        &ErrorResponse { success: false, message: format!("Unsupported method: {method}") },
    )
}

fn json_response<T: Serialize>(state: &AppState, status: StatusCode, value: &T) -> Response {
    let mut response = Json(value).into_response();
    *response.status_mut() = status;

    let headers = response.headers_mut();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(SERVER, HeaderValue::from_static(concat!("Seonbi/", env!("CARGO_PKG_VERSION"))));
    headers.insert(ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("*"));
    headers.insert(VARY, HeaderValue::from_static("origin"));
    if let Some(origin) = &state.allow_origin {
        headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());
    }

    response
}

struct ParsedInput {
    content: String,
    config: Configuration,
    warnings: Vec<String>,
}

fn parse_input(input: ApiInput) -> Result<ParsedInput, String> {
    let mut warnings = Vec::new();
    let content = match (input.content, input.source_html) {
        (Some(content), _) => content,
        (None, Some(source_html)) => {
            warnings.push("key \"sourceHtml\" is deprecated in favour of \"content\"".to_string());
            source_html
        }
        (None, None) => return Err("key \"content\" not present".to_string()),
    };

    let content_type_text = if let Some(content_type) = input.content_type {
        content_type
    } else if let Some(xhtml) = input.xhtml {
        warnings.push("key \"xhtml\" is deprecated in favour of \"contentType\"".to_string());
        if xhtml { "application/xhtml+xml".to_string() } else { "text/html".to_string() }
    } else {
        "text/html".to_string()
    };

    let content_type = parse_content_type(&content_type_text).ok_or_else(|| {
        let available = supported_content_types()
            .into_iter()
            .map(|t| t.as_str().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("Invalid content type: {content_type_text}; available content types: {available}")
    })?;

    let mut config = if let Some(preset) = input.preset {
        let preset_key = preset.to_ascii_lowercase().replace('_', "-");
        let preset_map = presets();
        preset_map.get(&preset_key).cloned().ok_or_else(|| {
            format!(
                "No such preset: {preset}; available presets: {}",
                preset_map.keys().cloned().collect::<Vec<_>>().join(", ")
            )
        })?
    } else {
        Configuration {
            debug_logger: None,
            content_type: content_type.clone(),
            quote: parse_quote_option(input.quote.as_deref())?,
            cite: parse_cite_option(input.cite.as_deref())?,
            arrow: input
                .arrow
                .map(|a| ArrowOption { bidir_arrow: a.bidir, double_arrow: a.double }),
            ellipsis: input.ellipsis.unwrap_or(false),
            em_dash: input.em_dash.unwrap_or(false),
            stop: parse_stop_option(input.stop.as_deref())?,
            hanja: parse_hanja_option(input.hanja)?,
        }
    };
    config.content_type = content_type;

    Ok(ParsedInput { content, config, warnings })
}

fn parse_quote_option(value: Option<&str>) -> Result<Option<QuoteOption>, String> {
    match value {
        None => Ok(None),
        Some("curved-quotes") => Ok(Some(QuoteOption::CurvedQuotes)),
        Some("vertical-corner-brackets") => Ok(Some(QuoteOption::VerticalCornerBrackets)),
        Some("horizontal-corner-brackets") => Ok(Some(QuoteOption::HorizontalCornerBrackets)),
        Some("guillemets") => Ok(Some(QuoteOption::Guillemets)),
        Some("curved-single-quotes-with-q") => Ok(Some(QuoteOption::CurvedSingleQuotesWithQ)),
        Some("vertical-corner-brackets-with-q") => {
            Ok(Some(QuoteOption::VerticalCornerBracketsWithQ))
        }
        Some("horizontal-corner-brackets-with-q") => {
            Ok(Some(QuoteOption::HorizontalCornerBracketsWithQ))
        }
        Some(other) => Err(format!("cannot parse value `{other}` as quote option")),
    }
}

fn parse_cite_option(value: Option<&str>) -> Result<Option<CiteOption>, String> {
    match value {
        None => Ok(None),
        Some("angle-quotes") => Ok(Some(CiteOption::AngleQuotes)),
        Some("corner-brackets") => Ok(Some(CiteOption::CornerBrackets)),
        Some("angle-quotes-with-cite") => Ok(Some(CiteOption::AngleQuotesWithCite)),
        Some("corner-brackets-with-cite") => Ok(Some(CiteOption::CornerBracketsWithCite)),
        Some(other) => Err(format!("cannot parse value `{other}` as cite option")),
    }
}

fn parse_stop_option(value: Option<&str>) -> Result<Option<StopOption>, String> {
    match value {
        None => Ok(None),
        Some("horizontal") => Ok(Some(StopOption::Horizontal)),
        Some("horizontal-with-slashes") => Ok(Some(StopOption::HorizontalWithSlashes)),
        Some("vertical") => Ok(Some(StopOption::Vertical)),
        Some(other) => Err(format!("cannot parse value `{other}` as stop option")),
    }
}

fn parse_hanja_option(input: Option<HanjaInput>) -> Result<Option<HanjaOption>, String> {
    let Some(hanja_input) = input else {
        return Ok(None);
    };

    let rendering = match hanja_input.rendering.as_str() {
        "hangul-only" => HanjaRenderingOption::HangulOnly,
        "hanja-in-parentheses" => HanjaRenderingOption::HanjaInParentheses,
        "disambiguating-hanja-in-parentheses" => {
            HanjaRenderingOption::DisambiguatingHanjaInParentheses
        }
        "hanja-in-ruby" => HanjaRenderingOption::HanjaInRuby,
        other => return Err(format!("cannot parse value `{other}` as hanja rendering option")),
    };

    let mut dictionary = hanja_input.reading.dictionary;
    for dict_id in hanja_input.reading.use_dictionaries {
        match dict_id.as_str() {
            "kr-stdict" => {
                for (k, v) in south_korean_dictionary() {
                    dictionary.entry(k).or_insert(v);
                }
            }
            _ => return Err(format!("No such dictionary ID: {dict_id}")),
        }
    }

    Ok(Some(HanjaOption {
        rendering,
        reading: HanjaReadingOption {
            initial_sound_law: hanja_input.reading.initial_sound_law,
            dictionary,
        },
    }))
}

fn apply_warning_comments(warnings: &[String], content: &str) -> String {
    if warnings.is_empty() {
        return content.to_string();
    }

    format!("<!--\n{}\n-->{content}", warnings.join("\n"))
}

fn normalize_host_for_bind(host: &str) -> String {
    match host.trim() {
        "*" | "[::/0]" | "[::]" => "::".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use serde_json::json;
    use tower::ServiceExt;

    fn test_state() -> AppState {
        AppState { allow_origin: None, debug_delay_ms: 0 }
    }

    #[tokio::test]
    async fn post_with_preset_returns_transformed_content() {
        let app = router(test_state());
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"content":"<p>漢字</p>","preset":"ko-kr"}).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(parsed["success"], true);
        assert_eq!(parsed["content"], "<p>한자</p>");
        assert_eq!(parsed["contentType"], "text/html");
    }

    #[tokio::test]
    async fn post_with_source_html_returns_warning() {
        let app = router(test_state());
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"sourceHtml":"<p>漢字</p>","preset":"ko-kr"}).to_string()))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            parsed["warnings"][0],
            "key \"sourceHtml\" is deprecated in favour of \"content\""
        );
    }

    #[tokio::test]
    async fn options_returns_ok() {
        let app = router(test_state());
        let req = Request::builder().method("OPTIONS").uri("/").body(Body::empty()).unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unsupported_method_returns_405() {
        let app = router(test_state());
        let req = Request::builder().method("PUT").uri("/").body(Body::empty()).unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["message"], "Unsupported method: PUT");
    }
}
