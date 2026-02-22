use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const DEFAULT_FUZZ_ITERS: usize = 300;
const DEFAULT_FUZZ_SEED: u64 = 0x5e0b1;
const DEFAULT_FUZZ_RECORD_FAIL: bool = true;

#[derive(Debug, Clone, Copy)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        // XorShift64 with a zero state stays at zero forever, so we remap it.
        let state = if seed == 0 { 0x9e37_79b9_7f4a_7c15 } else { seed };
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn index(&mut self, upper_exclusive: usize) -> usize {
        assert!(upper_exclusive > 0, "upper_exclusive must be > 0");
        (self.next_u64() % upper_exclusive as u64) as usize
    }

    fn range_inclusive(&mut self, start: usize, end: usize) -> usize {
        assert!(start <= end, "invalid inclusive range");
        start + self.index(end - start + 1)
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FuzzFailureRecord {
    seed: u64,
    case_index: usize,
    args: Vec<String>,
    input: String,
    original_status: Option<i32>,
    ported_status: Option<i32>,
    #[serde(default)]
    original_stdout: String,
    #[serde(default)]
    ported_stdout: String,
    #[serde(default)]
    original_stderr: String,
    #[serde(default)]
    ported_stderr: String,
}

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
    let (original, ported) = run_both(original_bin, ported_bin, args, stdin_input);
    assert_eq!(
        original.status.code(),
        ported.status.code(),
        "exit code differs\nargs: {:?}\nstdin: {:?}\noriginal stderr:\n{}\nported stderr:\n{}",
        args,
        stdin_input,
        String::from_utf8_lossy(&original.stderr),
        String::from_utf8_lossy(&ported.stderr),
    );
    assert_eq!(
        original.stdout,
        ported.stdout,
        "stdout differs\nargs: {:?}\nstdin: {:?}\noriginal stderr:\n{}\nported stderr:\n{}",
        args,
        stdin_input,
        String::from_utf8_lossy(&original.stderr),
        String::from_utf8_lossy(&ported.stderr),
    );
}

fn fuzzy_fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fuzz_regressions.jsonl")
}

fn parse_u64_env(name: &str, default: u64) -> u64 {
    let Ok(value) = env::var(name) else {
        return default;
    };
    let trimmed = value.trim();
    let parsed =
        if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
            u64::from_str_radix(hex, 16)
        } else {
            trimmed.parse::<u64>()
        };
    match parsed {
        Ok(parsed) => parsed,
        Err(err) => {
            eprintln!("invalid {name}={trimmed:?}: {err}; using default {default}");
            default
        }
    }
}

fn parse_usize_env(name: &str, default: usize) -> usize {
    let Ok(value) = env::var(name) else {
        return default;
    };
    match value.trim().parse::<usize>() {
        Ok(parsed) if parsed > 0 => parsed,
        Ok(_) => {
            eprintln!("invalid {name}={:?}: must be > 0; using default {default}", value);
            default
        }
        Err(err) => {
            eprintln!("invalid {name}={:?}: {err}; using default {default}", value);
            default
        }
    }
}

fn parse_bool_env(name: &str, default: bool) -> bool {
    let Ok(value) = env::var(name) else {
        return default;
    };
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => {
            eprintln!("invalid {name}={:?}; using default {default}", value);
            default
        }
    }
}

fn pick<'a>(rng: &mut XorShift64, items: &'a [&'a str]) -> &'a str {
    items[rng.index(items.len())]
}

fn generate_text_fragment(rng: &mut XorShift64, min_tokens: usize, max_tokens: usize) -> String {
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

    let mut out = String::new();
    out.push_str(pick(rng, HANJA));
    out.push(' ');
    out.push_str(pick(rng, HANGUL));

    let token_count = rng.range_inclusive(min_tokens, max_tokens);
    for _ in 0..token_count {
        let bucket = rng.index(5);
        let token = match bucket {
            0 => pick(rng, HANJA),
            1 => pick(rng, HANGUL),
            2 => pick(rng, LATIN),
            3 => pick(rng, ENTITIES),
            _ => pick(rng, PUNCT),
        };
        if !out.is_empty() && bucket != 4 {
            out.push_str(pick(rng, SEPARATORS));
        }
        out.push_str(token);
    }
    out
}

fn generate_fuzz_input(rng: &mut XorShift64) -> String {
    let a = generate_text_fragment(rng, 8, 28);
    let b = generate_text_fragment(rng, 6, 20);
    let c = generate_text_fragment(rng, 4, 14);

    match rng.index(7) {
        0 => format!("<p>{a}</p>"),
        1 => format!("<div><p>{a}</p><p>{b}</p></div>"),
        2 => format!("<article><h1>{c}</h1><p>{a}</p><blockquote>{b}</blockquote></article>"),
        3 => format!("<p lang=\"ko-Kore\">{a}</p><p>{b}</p>"),
        4 => format!("<p><q>{a}</q> <cite>{b}</cite></p>"),
        5 => format!("<pre>{a}</pre><p>{b}</p>"),
        _ => format!("{a}\n{b}"),
    }
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
        // Default rendering path (no explicit hanja flag)
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

        // Maintain-hanja conflicts with --no-initial-sound-law.
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

fn build_option_matrix() -> Vec<Vec<String>> {
    let quote_styles: [Option<&str>; 8] = [
        None,
        Some("curved-quotes"),
        Some("vertical-corner-brackets"),
        Some("horizontal-corner-brackets"),
        Some("guillemets"),
        Some("curved-single-quotes-with-q"),
        Some("vertical-corner-brackets-with-q"),
        Some("horizontal-corner-brackets-with-q"),
    ];
    let cite_styles: [Option<&str>; 5] = [
        None,
        Some("angle-quotes"),
        Some("corner-brackets"),
        Some("angle-quotes-with-cite"),
        Some("corner-brackets-with-cite"),
    ];
    let arrow_modes =
        [ArrowMode::None, ArrowMode::Bidir, ArrowMode::Double, ArrowMode::BidirAndDouble];
    let stop_styles: [Option<&str>; 4] =
        [None, Some("horizontal"), Some("horizontal-with-slashes"), Some("vertical")];
    let hanja_variants = hanja_variants();

    let mut matrix = Vec::new();

    for quote in quote_styles {
        for cite in cite_styles {
            for arrow_mode in arrow_modes {
                for no_ellipsis in [false, true] {
                    for no_em_dash in [false, true] {
                        for stop in stop_styles {
                            for hanja in &hanja_variants {
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

                                matrix.push(args);
                            }
                        }
                    }
                }
            }
        }
    }
    matrix
}

fn shuffle<T>(items: &mut [T], rng: &mut XorShift64) {
    for i in (1..items.len()).rev() {
        let j = rng.index(i + 1);
        items.swap(i, j);
    }
}

fn mismatch_record(
    seed: u64,
    case_index: usize,
    args: Vec<String>,
    input: String,
    original: Output,
    ported: Output,
) -> FuzzFailureRecord {
    FuzzFailureRecord {
        seed,
        case_index,
        args,
        input,
        original_status: original.status.code(),
        ported_status: ported.status.code(),
        original_stdout: String::from_utf8_lossy(&original.stdout).into_owned(),
        ported_stdout: String::from_utf8_lossy(&ported.stdout).into_owned(),
        original_stderr: String::from_utf8_lossy(&original.stderr).into_owned(),
        ported_stderr: String::from_utf8_lossy(&ported.stderr).into_owned(),
    }
}

fn compare_case_or_record(
    original_bin: &Path,
    ported_bin: &Path,
    seed: u64,
    case_index: usize,
    args: Vec<String>,
    input: String,
) -> Option<FuzzFailureRecord> {
    let (original, ported) = run_both(original_bin, ported_bin, &args, &input);
    if original.status.code() == ported.status.code() && original.stdout == ported.stdout {
        None
    } else {
        Some(mismatch_record(seed, case_index, args, input, original, ported))
    }
}

fn failure_key(record: &FuzzFailureRecord) -> String {
    format!("{}\u{001f}{}", record.args.join("\u{001e}"), record.input)
}

fn load_recorded_failures() -> Vec<FuzzFailureRecord> {
    let path = fuzzy_fixture_path();
    let file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(err) => panic!("failed to open {}: {err}", path.display()),
    };

    let mut records = Vec::new();
    for (line_no, line) in BufReader::new(file).lines().enumerate() {
        let line = line.unwrap_or_else(|err| {
            panic!("failed to read {} line {}: {err}", path.display(), line_no + 1)
        });
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let record: FuzzFailureRecord = serde_json::from_str(trimmed).unwrap_or_else(|err| {
            panic!("invalid json at {} line {}: {err}", path.display(), line_no + 1)
        });
        records.push(record);
    }
    records
}

fn append_failure_records(new_records: &[FuzzFailureRecord]) -> usize {
    if new_records.is_empty() {
        return 0;
    }

    let path = fuzzy_fixture_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fuzz fixture parent directory");
    }

    let mut seen = BTreeSet::new();
    for record in load_recorded_failures() {
        seen.insert(failure_key(&record));
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .expect("open fuzz regression fixture");
    let mut appended = 0usize;
    for record in new_records {
        if !seen.insert(failure_key(record)) {
            continue;
        }
        let line = serde_json::to_string(record).expect("serialize failure record");
        writeln!(file, "{line}").expect("append failure record");
        appended += 1;
    }
    appended
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
            // regression: numeric char refs preserved in non-hanja text
            ("numeric char ref in hanja", &["-e", "utf-8"], "<p>漢字 &#60;&#61;&#62;</p>"),
            // regression: empty quoted content preserves delimiters
            ("empty double quote", &["-e", "utf-8", "-q", "curved-quotes"], "\"\""),
            // regression: mixed-spelling quotes not paired
            ("mixed quote spelling", &["-e", "utf-8"], "\"a&quot;"),
        ];

        for (name, args, stdin_input) in cases {
            assert_equivalent(original_bin, &ported_bin, args, stdin_input);
            eprintln!("ok: {name}");
        }
    });
}

#[test]
fn compare_recorded_fuzz_regressions_with_original() {
    with_original_bin(|original_bin| {
        let ported_bin = current_bin_path();
        let records = load_recorded_failures();
        if records.is_empty() {
            eprintln!("no recorded fuzz regressions: {}", fuzzy_fixture_path().display());
            return;
        }

        for record in records {
            assert_equivalent(original_bin, &ported_bin, &record.args, &record.input);
            eprintln!(
                "replayed recorded fuzz case: seed={:#x} case_index={}",
                record.seed, record.case_index
            );
        }
    });
}

#[test]
fn compare_fuzz_matrix_with_original() {
    with_original_bin(|original_bin| {
        let ported_bin = current_bin_path();
        let seed = parse_u64_env("SEONBI_E2E_FUZZ_SEED", DEFAULT_FUZZ_SEED);
        let mut rng = XorShift64::new(seed);
        let requested_iters = parse_usize_env("SEONBI_E2E_FUZZ_ITERS", DEFAULT_FUZZ_ITERS);
        let record_failures =
            parse_bool_env("SEONBI_E2E_FUZZ_RECORD_FAIL", DEFAULT_FUZZ_RECORD_FAIL);

        let mut matrix = build_option_matrix();
        let matrix_size = matrix.len();
        shuffle(&mut matrix, &mut rng);

        let run_count = requested_iters.min(matrix_size);
        if requested_iters > matrix_size {
            eprintln!(
                "SEONBI_E2E_FUZZ_ITERS={requested_iters} exceeds matrix size {matrix_size}; capping to {run_count}"
            );
        }

        eprintln!("fuzz config: seed={seed:#x}, matrix={matrix_size}, run_count={run_count}");

        let mut failures = Vec::new();
        for (case_index, args) in matrix.into_iter().take(run_count).enumerate() {
            let input = generate_fuzz_input(&mut rng);
            if let Some(record) =
                compare_case_or_record(original_bin, &ported_bin, seed, case_index, args, input)
            {
                failures.push(record);
            }

            if (case_index + 1) % 25 == 0 || case_index + 1 == run_count {
                eprintln!("fuzz progress: {}/{}", case_index + 1, run_count);
            }
        }

        if failures.is_empty() {
            return;
        }

        let record_message = if record_failures {
            let appended = append_failure_records(&failures);
            format!("recorded {appended} new case(s) in {}", fuzzy_fixture_path().display())
        } else {
            "recording disabled via SEONBI_E2E_FUZZ_RECORD_FAIL=0".to_string()
        };

        panic!(
            "fuzz original-vs-ported mismatch: {} case(s) out of {}. {}",
            failures.len(),
            run_count,
            record_message
        );
    });
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
