use std::fs;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use clap::Parser;
use seonbi::{
    ArrowOption, Configuration, ContentType, HanjaDictionary, HanjaOption, HanjaReadingOption,
    HanjaRenderingOption, QuoteOption, presets, south_korean_dictionary, transform_html_text,
};

#[derive(Debug, Parser)]
#[command(name = "seonbi-cli")]
#[command(about = "Korean typographic adjustment processor")]
struct Args {
    #[arg(short = 'o', long, default_value = "-")]
    output: String,

    #[arg(short = 'p', long)]
    preset: Option<String>,

    #[arg(short = 't', long = "content-type", default_value = "text/html")]
    content_type: String,

    #[arg(long = "no-quote", short = 'Q')]
    no_quote: bool,

    #[arg(long = "no-arrow", short = 'A')]
    no_arrow: bool,

    #[arg(long = "bidir-arrow", short = 'b')]
    bidir_arrow: bool,

    #[arg(long = "double-arrow", short = 'd')]
    double_arrow: bool,

    #[arg(long = "no-ellipsis", short = 'E')]
    no_ellipsis: bool,

    #[arg(long = "no-em-dash", short = 'D')]
    no_em_dash: bool,

    #[arg(long = "maintain-hanja", short = 'H')]
    maintain_hanja: bool,

    #[arg(long = "no-initial-sound-law", short = 'I')]
    no_initial_sound_law: bool,

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
    let input_text = read_input(&args.input)?;
    let config = build_configuration(&args)?;
    let output_text = transform_html_text(&config, &input_text).map_err(|e| e.to_string())?;
    write_output(&args.output, &output_text)?;
    Ok(())
}

fn read_input(path: &str) -> Result<String, String> {
    if path == "-" {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|e| format!("failed to read stdin: {e}"))?;
        Ok(input)
    } else {
        fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))
    }
}

fn write_output(path: &str, text: &str) -> Result<(), String> {
    if path == "-" {
        io::stdout()
            .write_all(text.as_bytes())
            .map_err(|e| format!("failed to write stdout: {e}"))
    } else {
        fs::write(path, text).map_err(|e| format!("failed to write {path}: {e}"))
    }
}

fn build_configuration(args: &Args) -> Result<Configuration, String> {
    let mut config = if let Some(preset_name) = &args.preset {
        resolve_preset(preset_name)?
    } else {
        default_configuration()
    };

    let content_type = ContentType::from(args.content_type.as_str());
    if !seonbi::supported_content_types().contains(&content_type) {
        return Err(format!("unknown content type: {}", args.content_type));
    }
    config.content_type = content_type;

    if args.no_quote {
        config.quote = None;
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

    if args.maintain_hanja {
        config.hanja = None;
    } else if let Some(hanja) = &mut config.hanja
        && args.no_initial_sound_law
    {
        hanja.reading.initial_sound_law = false;
    }

    Ok(config)
}

fn resolve_preset(name: &str) -> Result<Configuration, String> {
    let normalized = name.to_ascii_lowercase().replace('_', "-");
    presets()
        .get(&normalized)
        .cloned()
        .ok_or_else(|| format!("no such preset: {name}"))
}

fn default_configuration() -> Configuration {
    Configuration {
        debug_logger: None,
        content_type: ContentType::from("text/html"),
        quote: Some(QuoteOption::CurvedQuotes),
        cite: None,
        arrow: Some(ArrowOption {
            bidir_arrow: false,
            double_arrow: false,
        }),
        ellipsis: true,
        em_dash: true,
        stop: None,
        hanja: Some(HanjaOption {
            rendering: HanjaRenderingOption::DisambiguatingHanjaInParentheses,
            reading: HanjaReadingOption {
                initial_sound_law: true,
                dictionary: default_dictionary(),
            },
        }),
    }
}

fn default_dictionary() -> HanjaDictionary {
    south_korean_dictionary()
}

