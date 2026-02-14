use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use seonbi::{
    ArrowOption, CiteOption, Configuration, HanjaDictionary, HanjaOption, HanjaReadingOption,
    HanjaRenderingOption, HtmlEntity, QuoteOption, StopOption, parse_content_type, presets,
    read_dictionary_file, south_korean_dictionary, supported_content_types, transform_html_text,
};

#[derive(Debug, Clone)]
struct HanjaReadingArg {
    hanja: String,
    hangul: String,
}

fn parse_hanja_reading_arg(arg: &str) -> Result<HanjaReadingArg, String> {
    let Some((hanja, hangul)) = arg.split_once(':') else {
        return Err(format!("colon is missing: \"{arg}\""));
    };
    if hanja.is_empty() {
        return Err(format!("hanja writing is missing: \"{arg}\""));
    }
    if hangul.is_empty() {
        return Err(format!("phonetic reading is missing: \"{arg}\""));
    }
    Ok(HanjaReadingArg { hanja: hanja.to_string(), hangul: hangul.to_string() })
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum QuoteStyleArg {
    CurvedQuotes,
    VerticalCornerBrackets,
    HorizontalCornerBrackets,
    Guillemets,
    CurvedSingleQuotesWithQ,
    VerticalCornerBracketsWithQ,
    HorizontalCornerBracketsWithQ,
}

impl From<QuoteStyleArg> for QuoteOption {
    fn from(value: QuoteStyleArg) -> Self {
        match value {
            QuoteStyleArg::CurvedQuotes => QuoteOption::CurvedQuotes,
            QuoteStyleArg::VerticalCornerBrackets => QuoteOption::VerticalCornerBrackets,
            QuoteStyleArg::HorizontalCornerBrackets => QuoteOption::HorizontalCornerBrackets,
            QuoteStyleArg::Guillemets => QuoteOption::Guillemets,
            QuoteStyleArg::CurvedSingleQuotesWithQ => QuoteOption::CurvedSingleQuotesWithQ,
            QuoteStyleArg::VerticalCornerBracketsWithQ => QuoteOption::VerticalCornerBracketsWithQ,
            QuoteStyleArg::HorizontalCornerBracketsWithQ => {
                QuoteOption::HorizontalCornerBracketsWithQ
            }
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CiteStyleArg {
    AngleQuotes,
    CornerBrackets,
    AngleQuotesWithCite,
    CornerBracketsWithCite,
}

impl From<CiteStyleArg> for CiteOption {
    fn from(value: CiteStyleArg) -> Self {
        match value {
            CiteStyleArg::AngleQuotes => CiteOption::AngleQuotes,
            CiteStyleArg::CornerBrackets => CiteOption::CornerBrackets,
            CiteStyleArg::AngleQuotesWithCite => CiteOption::AngleQuotesWithCite,
            CiteStyleArg::CornerBracketsWithCite => CiteOption::CornerBracketsWithCite,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum StopStyleArg {
    Horizontal,
    HorizontalWithSlashes,
    Vertical,
}

impl From<StopStyleArg> for StopOption {
    fn from(value: StopStyleArg) -> Self {
        match value {
            StopStyleArg::Horizontal => StopOption::Horizontal,
            StopStyleArg::HorizontalWithSlashes => StopOption::HorizontalWithSlashes,
            StopStyleArg::Vertical => StopOption::Vertical,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RenderHanjaStyleArg {
    HangulOnly,
    HanjaInParentheses,
    DisambiguatingHanjaInParentheses,
    HanjaInRuby,
}

impl From<RenderHanjaStyleArg> for HanjaRenderingOption {
    fn from(value: RenderHanjaStyleArg) -> Self {
        match value {
            RenderHanjaStyleArg::HangulOnly => HanjaRenderingOption::HangulOnly,
            RenderHanjaStyleArg::HanjaInParentheses => HanjaRenderingOption::HanjaInParentheses,
            RenderHanjaStyleArg::DisambiguatingHanjaInParentheses => {
                HanjaRenderingOption::DisambiguatingHanjaInParentheses
            }
            RenderHanjaStyleArg::HanjaInRuby => HanjaRenderingOption::HanjaInRuby,
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "seonbi-cli")]
#[command(about = "Korean typographic adjustment processor")]
struct Args {
    #[arg(short = 'o', long, default_value = "-")]
    output: String,

    #[arg(short = 'e', long, default_value = "")]
    encoding: String,

    #[arg(short = 'p', long)]
    preset: Option<String>,

    #[arg(long = "quote", short = 'q', value_enum, conflicts_with = "no_quote")]
    quote: Option<QuoteStyleArg>,

    #[arg(long = "no-quote", short = 'Q')]
    no_quote: bool,

    #[arg(long = "cite", short = 'c', value_enum)]
    cite: Option<CiteStyleArg>,

    #[arg(long = "no-arrow", short = 'A', conflicts_with_all = ["bidir_arrow", "double_arrow"])]
    no_arrow: bool,

    #[arg(long = "bidir-arrow", short = 'b')]
    bidir_arrow: bool,

    #[arg(long = "double-arrow", short = 'd')]
    double_arrow: bool,

    #[arg(long = "no-ellipsis", short = 'E')]
    no_ellipsis: bool,

    // Original seonbi 0.5.0 uses `-D` for both `--no-em-dash` and `--dict`.
    // In clap this is ambiguous, so we keep `-D` for `--dict` and use `-M`
    // for `--no-em-dash`.
    #[arg(long = "no-em-dash", short = 'M')]
    no_em_dash: bool,

    #[arg(long = "stop", short = 's', value_enum)]
    stop: Option<StopStyleArg>,

    #[arg(long = "maintain-hanja", short = 'H', conflicts_with_all = ["render_hanja", "no_initial_sound_law"])]
    maintain_hanja: bool,

    #[arg(long = "render-hanja", short = 'r', value_enum)]
    render_hanja: Option<RenderHanjaStyleArg>,

    #[arg(long = "no-initial-sound-law", short = 'I')]
    no_initial_sound_law: bool,

    #[arg(long = "read-hanja", short = 'R', value_parser = parse_hanja_reading_arg)]
    read_hanja: Vec<HanjaReadingArg>,

    #[arg(long = "dict", short = 'D')]
    dict: Vec<String>,

    #[arg(long = "no-kr-stdict", short = 'S')]
    no_kr_stdict: bool,

    #[arg(short = 't', long = "content-type", default_value = "text/html")]
    content_type: String,

    #[arg(long = "debug", hide = true)]
    debug: bool,

    #[arg(short = 'v', long = "version", hide = true)]
    version: bool,

    #[arg(value_name = "FILE", default_value = "-")]
    input: String,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), String> {
    reject_preset_with_style_options(&args)?;

    if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let input_bytes = read_input_bytes(&args.input)?;
    let encoding_name = if args.encoding.is_empty() {
        detect_encoding_name(&input_bytes).to_string()
    } else {
        args.encoding.clone()
    };
    let input_text = decode_text(&encoding_name, &input_bytes)?;

    let mut config = build_configuration(&args)?;
    if args.debug {
        eprintln!("encoding: {encoding_name}");
        config.debug_logger = Some(debug_logger);
    }

    let output_text = transform_html_text(&config, &input_text).map_err(|e| e.to_string())?;
    let output_bytes = encode_text(&encoding_name, &output_text)?;
    write_output_bytes(&args.output, &output_bytes)?;
    Ok(())
}

fn reject_preset_with_style_options(args: &Args) -> Result<(), String> {
    if args.preset.is_none() {
        return Ok(());
    }

    let has_style_overrides = args.quote.is_some()
        || args.no_quote
        || args.cite.is_some()
        || args.no_arrow
        || args.bidir_arrow
        || args.double_arrow
        || args.no_ellipsis
        || args.no_em_dash
        || args.stop.is_some()
        || args.maintain_hanja
        || args.render_hanja.is_some()
        || args.no_initial_sound_law
        || !args.read_hanja.is_empty();

    if has_style_overrides { Err("--preset rejects style options".to_string()) } else { Ok(()) }
}

fn read_input_bytes(path: &str) -> Result<Vec<u8>, String> {
    if path == "-" {
        let mut input = Vec::new();
        io::stdin().read_to_end(&mut input).map_err(|e| format!("failed to read stdin: {e}"))?;
        Ok(input)
    } else {
        fs::read(path).map_err(|e| format!("failed to read {path}: {e}"))
    }
}

fn write_output_bytes(path: &str, bytes: &[u8]) -> Result<(), String> {
    if path == "-" {
        io::stdout().write_all(bytes).map_err(|e| format!("failed to write stdout: {e}"))
    } else {
        fs::write(path, bytes).map_err(|e| format!("failed to write {path}: {e}"))
    }
}

fn build_configuration(args: &Args) -> Result<Configuration, String> {
    let mut config = if let Some(preset_name) = &args.preset {
        resolve_preset(preset_name)?
    } else {
        default_configuration()
    };

    let content_type = parse_content_type(&args.content_type).ok_or_else(|| {
        let available = supported_content_types()
            .into_iter()
            .map(|ct| ct.as_str().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("unknown content type: {}; available content types: {available}", args.content_type)
    })?;
    config.content_type = content_type;

    if args.preset.is_none() {
        if args.no_quote {
            config.quote = None;
        } else if let Some(quote) = args.quote {
            config.quote = Some(quote.into());
        }

        if let Some(cite) = args.cite {
            config.cite = Some(cite.into());
        }

        if args.no_arrow {
            config.arrow = None;
        } else if let Some(arrow) = &mut config.arrow {
            arrow.bidir_arrow = args.bidir_arrow;
            arrow.double_arrow = args.double_arrow;
        } else if args.bidir_arrow || args.double_arrow {
            config.arrow = Some(ArrowOption {
                bidir_arrow: args.bidir_arrow,
                double_arrow: args.double_arrow,
            });
        }

        if args.no_ellipsis {
            config.ellipsis = false;
        }
        if args.no_em_dash {
            config.em_dash = false;
        }

        if let Some(stop) = args.stop {
            config.stop = Some(stop.into());
        }

        if args.maintain_hanja {
            config.hanja = None;
        } else {
            let rendering = args
                .render_hanja
                .map(HanjaRenderingOption::from)
                .unwrap_or(HanjaRenderingOption::DisambiguatingHanjaInParentheses);
            config.hanja = Some(HanjaOption {
                rendering,
                reading: HanjaReadingOption {
                    initial_sound_law: !args.no_initial_sound_law,
                    dictionary: BTreeMap::new(),
                },
            });
        }
    }

    if let Some(hanja) = &mut config.hanja {
        if args.no_initial_sound_law {
            hanja.reading.initial_sound_law = false;
        }
        hanja.reading.dictionary = load_dictionary(
            &args.read_hanja,
            &args.dict,
            hanja.reading.initial_sound_law,
            args.no_kr_stdict,
        )?;
    }

    Ok(config)
}

fn load_dictionary(
    read_hanja: &[HanjaReadingArg],
    dict_paths: &[String],
    initial_sound_law: bool,
    no_kr_stdict: bool,
) -> Result<HanjaDictionary, String> {
    let mut dictionary = BTreeMap::new();

    for entry in read_hanja {
        dictionary.insert(entry.hanja.clone(), entry.hangul.clone());
    }

    for path in dict_paths {
        let loaded = read_dictionary_file(path.as_ref())
            .map_err(|e| format!("failed to load dictionary {path}: {e}"))?;
        for (hanja, hangul) in loaded {
            dictionary.entry(hanja).or_insert(hangul);
        }
    }

    if initial_sound_law && !no_kr_stdict {
        for (hanja, hangul) in south_korean_dictionary() {
            dictionary.entry(hanja).or_insert(hangul);
        }
    }

    Ok(dictionary)
}

fn resolve_preset(name: &str) -> Result<Configuration, String> {
    let normalized = name.to_ascii_lowercase().replace('_', "-");
    presets().get(&normalized).cloned().ok_or_else(|| format!("no such preset: {name}"))
}

fn default_configuration() -> Configuration {
    Configuration {
        debug_logger: None,
        content_type: parse_content_type("text/html").expect("known content type"),
        quote: Some(QuoteOption::CurvedQuotes),
        cite: None,
        arrow: Some(ArrowOption { bidir_arrow: false, double_arrow: false }),
        ellipsis: true,
        em_dash: true,
        stop: None,
        hanja: Some(HanjaOption {
            rendering: HanjaRenderingOption::DisambiguatingHanjaInParentheses,
            reading: HanjaReadingOption { initial_sound_law: true, dictionary: BTreeMap::new() },
        }),
    }
}

fn normalize_encoding_name(encoding: &str) -> String {
    encoding.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_lowercase()).collect()
}

fn detect_encoding_name(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return "utf8";
    }
    if bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
        return "utf32le";
    }
    if bytes.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
        return "utf32be";
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return "utf16le";
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return "utf16be";
    }

    let head_len = bytes.len().min(4096);
    let mut head = String::with_capacity(head_len);
    for &b in &bytes[..head_len] {
        if b.is_ascii() {
            head.push((b as char).to_ascii_lowercase());
        } else {
            head.push(' ');
        }
    }

    if let Some(enc) = find_charset_in_html_head(&head) {
        match enc.as_str() {
            "utf8" => return "utf8",
            "utf16le" => return "utf16le",
            "utf16be" => return "utf16be",
            "utf32le" => return "utf32le",
            "utf32be" => return "utf32be",
            _ => {}
        }
    }

    "utf8"
}

fn find_charset_in_html_head(head: &str) -> Option<String> {
    let charset_pos = head.find("charset")?;
    let after = &head[charset_pos + "charset".len()..];
    let eq_pos = after.find('=')?;
    let mut value = after[eq_pos + 1..].trim_start();

    if value.starts_with('"') {
        value = &value[1..];
        let end = value.find('"')?;
        return Some(normalize_encoding_name(&value[..end]));
    }

    if value.starts_with('\'') {
        value = &value[1..];
        let end = value.find('\'')?;
        return Some(normalize_encoding_name(&value[..end]));
    }

    let end = value
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .unwrap_or(value.len());
    if end == 0 {
        return None;
    }
    Some(normalize_encoding_name(&value[..end]))
}

fn decode_text(encoding_name: &str, bytes: &[u8]) -> Result<String, String> {
    let normalized = normalize_encoding_name(encoding_name);
    match normalized.as_str() {
        "utf8" => {
            let slice = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) { &bytes[3..] } else { bytes };
            std::str::from_utf8(slice)
                .map(|s| s.to_string())
                .map_err(|e| format!("failed to decode utf-8 input: {e}"))
        }
        "utf16le" | "utf16be" => decode_utf16(&normalized, bytes),
        "utf32le" | "utf32be" => decode_utf32(&normalized, bytes),
        _ => Err("Only UTF-{8,16,32} encodings are supported (e.g., UTF-8, UTF-16LE, UTF-32BE)"
            .to_string()),
    }
}

fn decode_utf16(encoding: &str, bytes: &[u8]) -> Result<String, String> {
    if bytes.len() % 2 != 0 {
        return Err("invalid UTF-16 byte length".to_string());
    }

    let mut units = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let value = if encoding.ends_with("le") {
            u16::from_le_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], chunk[1]])
        };
        units.push(value);
    }

    if units.first() == Some(&0xFEFF) {
        units.remove(0);
    }

    String::from_utf16(&units).map_err(|e| format!("failed to decode UTF-16 input: {e}"))
}

fn decode_utf32(encoding: &str, bytes: &[u8]) -> Result<String, String> {
    if bytes.len() % 4 != 0 {
        return Err("invalid UTF-32 byte length".to_string());
    }

    let mut out = String::new();
    for (idx, chunk) in bytes.chunks_exact(4).enumerate() {
        let code = if encoding.ends_with("le") {
            u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        } else {
            u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        };

        if idx == 0 && code == 0xFEFF {
            continue;
        }

        let ch = char::from_u32(code)
            .ok_or_else(|| format!("invalid UTF-32 scalar value: 0x{code:X}"))?;
        out.push(ch);
    }

    Ok(out)
}

fn encode_text(encoding_name: &str, text: &str) -> Result<Vec<u8>, String> {
    let normalized = normalize_encoding_name(encoding_name);
    match normalized.as_str() {
        "utf8" => Ok(text.as_bytes().to_vec()),
        "utf16le" => {
            let mut out = Vec::with_capacity(text.len() * 2);
            for unit in text.encode_utf16() {
                out.extend_from_slice(&unit.to_le_bytes());
            }
            Ok(out)
        }
        "utf16be" => {
            let mut out = Vec::with_capacity(text.len() * 2);
            for unit in text.encode_utf16() {
                out.extend_from_slice(&unit.to_be_bytes());
            }
            Ok(out)
        }
        "utf32le" => {
            let mut out = Vec::with_capacity(text.len() * 4);
            for ch in text.chars() {
                out.extend_from_slice(&(ch as u32).to_le_bytes());
            }
            Ok(out)
        }
        "utf32be" => {
            let mut out = Vec::with_capacity(text.len() * 4);
            for ch in text.chars() {
                out.extend_from_slice(&(ch as u32).to_be_bytes());
            }
            Ok(out)
        }
        _ => Err("Only UTF-{8,16,32} encodings are supported (e.g., UTF-8, UTF-16LE, UTF-32BE)"
            .to_string()),
    }
}

fn debug_logger(entity: &HtmlEntity) {
    eprintln!("{}", format_entity(entity));
}

fn format_entity(entity: &HtmlEntity) -> String {
    match entity {
        HtmlEntity::StartTag { tag, raw_attributes, .. } => {
            if raw_attributes.is_empty() {
                format!("<{}>", tag.name())
            } else if raw_attributes.chars().next().is_some_and(char::is_whitespace) {
                format!("<{}{}>", tag.name(), raw_attributes)
            } else {
                format!("<{} {}>", tag.name(), raw_attributes)
            }
        }
        HtmlEntity::EndTag { tag, .. } => format!("</{}>", tag.name()),
        HtmlEntity::Text { raw_text, .. } => format!("!text  {raw_text}"),
        HtmlEntity::Cdata { text, .. } => format!("!cdata {text}"),
        HtmlEntity::Comment { comment, .. } => format!("<!-- {comment} -->"),
    }
}
