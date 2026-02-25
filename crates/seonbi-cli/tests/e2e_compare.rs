use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use proptest::prelude::*;
use proptest::sample::select;
use proptest::test_runner::{Config as ProptestConfig, FileFailurePersistence, TestCaseError};

#[derive(Debug, Clone, Copy)]
enum ArrowMode {
    None,
    Bidir,
    Double,
    BidirAndDouble,
}

#[derive(Debug, Clone, Copy)]
enum HanjaMode {
    Default,
    Maintain,
    Render(&'static str),
}

#[derive(Debug, Clone, Copy)]
struct HanjaVariant {
    mode: HanjaMode,
    no_initial_sound_law: bool,
    no_kr_stdict: bool,
    custom_reading: bool,
}

const HANJA: &[&str] = &[
    "國漢文混用體",
    "漢字",
    "平壤",
    "冷麵",
    "孫文",
    "困難",
    "大韓民國",
    "許諾",
    "六",
    "共和國",
    "無情",
];

const HANGUL: &[&str] = &[
    "국한문",
    "혼용체",
    "한글",
    "문장",
    "변환",
    "평양",
    "냉면",
    "곤란",
    "대한민국",
    "테스트",
    "문맥",
];

const LATIN: &[&str] = &["alpha", "beta", "gamma", "A", "B", "C", "123", "xyz"];
const ENTITIES: &[&str] =
    &["&quot;", "&lt;", "&gt;", "&#60;", "&#61;", "&#62;", "&lt;&lt;", "&gt;&gt;"];
const PUNCT: &[&str] =
    &["\"", "'", "<<", ">>", "...", "--", "<->", "=>", "。", "、", "·", "!", "?", "(", ")"];
const SEPARATORS: &[&str] = &[" ", " ", " ", "  ", "\n", "\t"];

fn run_binary<S: AsRef<OsStr>>(bin_path: &Path, args: &[S], stdin_input: &str) -> Output {
    let mut cmd = Command::new(bin_path);
    cmd.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn binary");
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        stdin.write_all(stdin_input.as_bytes()).expect("write stdin");
    }
    child.wait_with_output().expect("wait binary")
}

fn run_both<S: AsRef<OsStr>>(
    original_bin: &Path,
    ported_bin: &Path,
    args: &[S],
    stdin_input: &str,
) -> (Output, Output) {
    let original = run_binary(original_bin, args, stdin_input);
    let ported = run_binary(ported_bin, args, stdin_input);
    (original, ported)
}

fn current_bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_seonbi-cli"))
}

fn original_bin_path() -> Option<PathBuf> {
    let path = env::var("SEONBI_ORIGINAL_BIN").ok()?;
    let path = PathBuf::from(path);
    if path.exists() { Some(path) } else { None }
}

fn kr_stdict_is_lfs_pointer() -> bool {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../seonbi/data/ko-kr-stdict.tsv");
    let Ok(contents) = fs::read_to_string(path) else {
        return false;
    };
    contents.starts_with("version https://git-lfs.github.com/spec/v1")
}

fn temp_path(name: &str) -> std::path::PathBuf {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    std::env::temp_dir().join(format!("seonbi_cli_compare_{name}_{ts}.html"))
}

fn with_original_bin<F>(f: F)
where
    F: FnOnce(&Path),
{
    let Some(original_bin) = original_bin_path() else {
        eprintln!("skipping: set SEONBI_ORIGINAL_BIN to run comparison E2E");
        return;
    };
    if kr_stdict_is_lfs_pointer() {
        eprintln!(
            "skipping: crates/seonbi/data/ko-kr-stdict.tsv is an unresolved Git LFS pointer; run git lfs pull"
        );
        return;
    }
    f(&original_bin);
}

fn assert_equivalent<S: AsRef<OsStr> + std::fmt::Debug>(
    original_bin: &Path,
    ported_bin: &Path,
    args: &[S],
    stdin_input: &str,
) {
    if let Err(message) = equivalence_error(original_bin, ported_bin, args, stdin_input) {
        panic!("{message}");
    }
}

fn equivalence_error<S: AsRef<OsStr> + std::fmt::Debug>(
    original_bin: &Path,
    ported_bin: &Path,
    args: &[S],
    stdin_input: &str,
) -> Result<(), String> {
    let (original, ported) = run_both(original_bin, ported_bin, args, stdin_input);
    if original.status.code() != ported.status.code() {
        return Err(format!(
            "exit code differs\nargs: {:?}\nstdin: {:?}\noriginal stderr:\n{}\nported stderr:\n{}",
            args,
            stdin_input,
            String::from_utf8_lossy(&original.stderr),
            String::from_utf8_lossy(&ported.stderr),
        ));
    }
    if original.stdout != ported.stdout {
        return Err(format!(
            "stdout differs\nargs: {:?}\nstdin: {:?}\noriginal stderr:\n{}\nported stderr:\n{}",
            args,
            stdin_input,
            String::from_utf8_lossy(&original.stderr),
            String::from_utf8_lossy(&ported.stderr),
        ));
    }
    Ok(())
}

fn pick_token(bucket: usize, index: usize) -> &'static str {
    match bucket {
        0 => HANJA[index % HANJA.len()],
        1 => HANGUL[index % HANGUL.len()],
        2 => LATIN[index % LATIN.len()],
        3 => ENTITIES[index % ENTITIES.len()],
        _ => PUNCT[index % PUNCT.len()],
    }
}

fn text_fragment_strategy(min_tokens: usize, max_tokens: usize) -> impl Strategy<Value = String> {
    (
        0usize..HANJA.len(),
        0usize..HANGUL.len(),
        prop::collection::vec(
            (0usize..5usize, 0usize..64usize, 0usize..SEPARATORS.len()),
            min_tokens..=max_tokens,
        ),
    )
        .prop_map(|(first_hanja, first_hangul, steps)| {
            let mut out = String::new();
            out.push_str(HANJA[first_hanja]);
            out.push(' ');
            out.push_str(HANGUL[first_hangul]);

            for (bucket, token_index, sep_index) in steps {
                if !out.is_empty() && bucket != 4 {
                    out.push_str(SEPARATORS[sep_index]);
                }
                out.push_str(pick_token(bucket, token_index));
            }

            out
        })
}

fn fuzz_input_strategy() -> impl Strategy<Value = String> {
    (
        text_fragment_strategy(8, 28),
        text_fragment_strategy(6, 20),
        text_fragment_strategy(4, 14),
        0usize..7usize,
    )
        .prop_map(|(a, b, c, shape)| match shape {
            0 => format!("<p>{a}</p>"),
            1 => format!("<div><p>{a}</p><p>{b}</p></div>"),
            2 => format!("<article><h1>{c}</h1><p>{a}</p><blockquote>{b}</blockquote></article>"),
            3 => format!("<p lang=\"ko-Kore\">{a}</p><p>{b}</p>"),
            4 => format!("<p><q>{a}</q> <cite>{b}</cite></p>"),
            5 => format!("<pre>{a}</pre><p>{b}</p>"),
            _ => format!("{a}\n{b}"),
        })
}

fn quote_style_strategy() -> impl Strategy<Value = Option<&'static str>> {
    prop_oneof![
        Just(None::<&'static str>),
        Just(Some("curved-quotes")),
        Just(Some("vertical-corner-brackets")),
        Just(Some("horizontal-corner-brackets")),
        Just(Some("guillemets")),
        Just(Some("curved-single-quotes-with-q")),
        Just(Some("vertical-corner-brackets-with-q")),
        Just(Some("horizontal-corner-brackets-with-q")),
    ]
}

fn cite_style_strategy() -> impl Strategy<Value = Option<&'static str>> {
    prop_oneof![
        Just(None::<&'static str>),
        Just(Some("angle-quotes")),
        Just(Some("corner-brackets")),
        Just(Some("angle-quotes-with-cite")),
        Just(Some("corner-brackets-with-cite")),
    ]
}

fn stop_style_strategy() -> impl Strategy<Value = Option<&'static str>> {
    prop_oneof![
        Just(None::<&'static str>),
        Just(Some("horizontal")),
        Just(Some("horizontal-with-slashes")),
        Just(Some("vertical")),
    ]
}

fn arrow_mode_strategy() -> impl Strategy<Value = ArrowMode> {
    prop_oneof![
        Just(ArrowMode::None),
        Just(ArrowMode::Bidir),
        Just(ArrowMode::Double),
        Just(ArrowMode::BidirAndDouble),
    ]
}

fn hanja_variants() -> Vec<HanjaVariant> {
    let mut out = Vec::new();
    let render_modes = [
        HanjaMode::Render("hangul-only"),
        HanjaMode::Render("hanja-in-parentheses"),
        HanjaMode::Render("disambiguating-hanja-in-parentheses"),
        HanjaMode::Render("hanja-in-ruby"),
    ];

    for no_kr_stdict in [false, true] {
        for no_initial_sound_law in [false, true] {
            for custom_reading in [false, true] {
                out.push(HanjaVariant {
                    mode: HanjaMode::Default,
                    no_initial_sound_law,
                    no_kr_stdict,
                    custom_reading,
                });
            }
        }

        out.push(HanjaVariant {
            mode: HanjaMode::Maintain,
            no_initial_sound_law: false,
            no_kr_stdict,
            custom_reading: false,
        });

        for mode in render_modes {
            for no_initial_sound_law in [false, true] {
                for custom_reading in [false, true] {
                    out.push(HanjaVariant {
                        mode,
                        no_initial_sound_law,
                        no_kr_stdict,
                        custom_reading,
                    });
                }
            }
        }
    }

    out
}

fn option_args_strategy() -> impl Strategy<Value = Vec<String>> {
    (
        quote_style_strategy(),
        cite_style_strategy(),
        arrow_mode_strategy(),
        any::<bool>(),
        any::<bool>(),
        stop_style_strategy(),
        select(hanja_variants()),
    )
        .prop_map(|(quote, cite, arrow_mode, no_ellipsis, no_em_dash, stop, hanja)| {
            let mut args = vec!["--encoding".to_string(), "utf-8".to_string()];

            if let Some(style) = quote {
                args.push("--quote".to_string());
                args.push(style.to_string());
            }
            if let Some(style) = cite {
                args.push("--cite".to_string());
                args.push(style.to_string());
            }

            match arrow_mode {
                ArrowMode::None => {}
                ArrowMode::Bidir => args.push("--bidir-arrow".to_string()),
                ArrowMode::Double => args.push("--double-arrow".to_string()),
                ArrowMode::BidirAndDouble => {
                    args.push("--bidir-arrow".to_string());
                    args.push("--double-arrow".to_string());
                }
            }

            if no_ellipsis {
                args.push("--no-ellipsis".to_string());
            }
            if no_em_dash {
                args.push("--no-em-dash".to_string());
            }
            if let Some(style) = stop {
                args.push("--stop".to_string());
                args.push(style.to_string());
            }

            match hanja.mode {
                HanjaMode::Default => {}
                HanjaMode::Maintain => {
                    args.push("--maintain-hanja".to_string());
                }
                HanjaMode::Render(style) => {
                    args.push("--render-hanja".to_string());
                    args.push(style.to_string());
                }
            }

            if hanja.no_initial_sound_law {
                args.push("--no-initial-sound-law".to_string());
            }
            if hanja.no_kr_stdict {
                args.push("--no-kr-stdict".to_string());
            }
            if hanja.custom_reading {
                args.push("--read-hanja".to_string());
                args.push("孫文:쑨원".to_string());
                args.push("--read-hanja".to_string());
                args.push("國漢文混用體:국한문 혼용체".to_string());
            }

            args
        })
}

fn fuzz_case_strategy() -> impl Strategy<Value = (Vec<String>, String)> {
    (option_args_strategy(), fuzz_input_strategy())
}

fn assert_equivalent_proptest(
    original_bin: &Path,
    ported_bin: &Path,
    args: &[String],
    stdin_input: &str,
) -> Result<(), TestCaseError> {
    match equivalence_error(original_bin, ported_bin, args, stdin_input) {
        Ok(()) => Ok(()),
        Err(message) => Err(TestCaseError::fail(message)),
    }
}

#[test]
fn compare_stdin_stdout_matrix() {
    with_original_bin(|original_bin| {
        let ported_bin = current_bin_path();
        let cases: &[(&str, &[&str], &str)] = &[
            ("preset ko-kr", &["-p", "ko-kr"], "<p>漢字</p>"),
            ("preset ko-kp", &["-p", "ko-kp"], "<p>平壤 冷麵</p>"),
            ("quote at text end", &["-e", "utf-8", "-p", "ko-kr"], "<p>\"abc\"</p>"),
            ("quote", &["-q", "guillemets"], "'a' \"b\" c"),
            ("cite", &["-c", "angle-quotes-with-cite"], "<p>&lt;&lt;無情&gt;&gt;</p>"),
            ("arrow bidir+double", &["-b", "-d"], "<p>A <-> B => C</p>"),
            ("stop horizontal", &["-s", "horizontal"], "봄。 여름"),
            ("no quote", &["--no-quote"], "\"A\""),
            ("no arrow", &["--no-arrow"], "<p>A <-> B</p>"),
            ("no ellipsis", &["--no-ellipsis"], "..."),
            ("no em-dash", &["--no-em-dash"], "a -- b"),
            ("maintain hanja", &["--maintain-hanja"], "<p>漢字</p>"),
            ("render hanja", &["--render-hanja", "hanja-in-parentheses"], "<p>漢字</p>"),
            ("no initial sound law", &["--no-initial-sound-law"], "<p>六</p>"),
            ("custom reading", &["-R", "孫文:쑨원"], "<p>孫文</p>"),
            ("no kr stdict", &["-S"], "<p>困難</p>"),
            ("plain text", &["-t", "text/plain"], "漢字"),
            ("markdown", &["-t", "text/markdown"], "*漢字*"),
            ("numeric char ref in hanja", &["-e", "utf-8"], "<p>漢字 &#60;&#61;&#62;</p>"),
            ("empty double quote", &["-e", "utf-8", "-q", "curved-quotes"], "\"\""),
            ("mixed quote spelling", &["-e", "utf-8"], "\"a&quot;"),
        ];

        for (name, args, stdin_input) in cases {
            assert_equivalent(original_bin, &ported_bin, args, stdin_input);
            eprintln!("ok: {name}");
        }
    });
}

proptest! {
    #![proptest_config(ProptestConfig::with_failure_persistence(
        FileFailurePersistence::Direct(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/proptest-regressions/e2e_compare.txt"
        ))
    ))]

    #[test]
    fn compare_fuzz_with_original_proptest((args, input) in fuzz_case_strategy()) {
        let Some(original_bin) = original_bin_path() else {
            return Ok(());
        };
        if kr_stdict_is_lfs_pointer() {
            return Ok(());
        }

        let ported_bin = current_bin_path();
        assert_equivalent_proptest(&original_bin, &ported_bin, &args, &input)?;
    }
}

#[test]
fn compare_file_io() {
    with_original_bin(|original_bin| {
        let ported_bin = current_bin_path();

        let input_path = temp_path("in");
        let output_path_original = temp_path("out_original");
        let output_path_ported = temp_path("out_ported");
        fs::write(&input_path, "<p>平壤 冷麵</p>").expect("write input fixture");

        let args_original = [
            "-p",
            "ko-kr",
            "-o",
            output_path_original.to_str().expect("output path"),
            input_path.to_str().expect("input path"),
        ];
        let args_ported = [
            "-p",
            "ko-kr",
            "-o",
            output_path_ported.to_str().expect("output path"),
            input_path.to_str().expect("input path"),
        ];

        let original = run_binary(original_bin, &args_original, "");
        let ported = run_binary(&ported_bin, &args_ported, "");

        assert_eq!(
            original.status.code(),
            ported.status.code(),
            "exit code differs\noriginal stderr:\n{}\nported stderr:\n{}",
            String::from_utf8_lossy(&original.stderr),
            String::from_utf8_lossy(&ported.stderr),
        );

        let written_original =
            fs::read_to_string(&output_path_original).expect("read original output");
        let written_ported = fs::read_to_string(&output_path_ported).expect("read ported output");
        assert_eq!(written_original, written_ported, "file output differs");

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&output_path_original);
        let _ = fs::remove_file(&output_path_ported);
    });
}

#[test]
fn compare_dict_short_option() {
    with_original_bin(|original_bin| {
        let ported_bin = current_bin_path();

        let input_path = temp_path("dict_input");
        let dict_path = temp_path("dict");
        fs::write(&input_path, "<p>毛澤東</p>").expect("write input fixture");
        fs::write(&dict_path, "毛澤東\t마오쩌둥\n").expect("write dictionary");

        let args_original = [
            "-D",
            dict_path.to_str().expect("dict path"),
            input_path.to_str().expect("input path"),
        ];
        let args_ported = [
            "-D",
            dict_path.to_str().expect("dict path"),
            input_path.to_str().expect("input path"),
        ];

        let original = run_binary(original_bin, &args_original, "");
        let ported = run_binary(&ported_bin, &args_ported, "");

        assert_eq!(
            original.status.code(),
            ported.status.code(),
            "exit code differs\noriginal stderr:\n{}\nported stderr:\n{}",
            String::from_utf8_lossy(&original.stderr),
            String::from_utf8_lossy(&ported.stderr),
        );
        assert_eq!(
            original.stdout,
            ported.stdout,
            "stdout differs\noriginal stderr:\n{}\nported stderr:\n{}",
            String::from_utf8_lossy(&original.stderr),
            String::from_utf8_lossy(&ported.stderr),
        );

        let _ = fs::remove_file(&input_path);
        let _ = fs::remove_file(&dict_path);
    });
}
