use std::fs;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn run_cli(args: &[&str], stdin_input: &[u8]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_seonbi-cli"));
    cmd.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn cli");
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        stdin.write_all(stdin_input).expect("write stdin to cli");
    }
    child.wait_with_output().expect("wait cli")
}

fn temp_path(name: &str, ext: &str) -> std::path::PathBuf {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    std::env::temp_dir().join(format!("seonbi_cli_{name}_{ts}.{ext}"))
}

fn decode_utf16le(bytes: &[u8]) -> String {
    let body = if bytes.starts_with(&[0xFF, 0xFE]) { &bytes[2..] } else { bytes };
    let mut units = Vec::new();
    for chunk in body.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16(&units).expect("utf16le decode")
}

#[test]
fn transforms_stdin_to_stdout() {
    let output = run_cli(&["-p", "ko-kr"], "<p>漢字</p>".as_bytes());
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    assert_eq!(stdout, "<p>한자</p>");
}

#[test]
fn normalizes_preset_name_with_underscore() {
    let output = run_cli(&["-p", "ko_kp"], "<p>六</p>".as_bytes());
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    assert_eq!(stdout, "<p>륙</p>");
}

#[test]
fn reads_and_writes_files() {
    let input_path = temp_path("in", "html");
    let output_path = temp_path("out", "html");
    fs::write(&input_path, "<p>平壤 冷麵</p>").expect("write input fixture");

    let output = run_cli(
        &[
            "-p",
            "ko-kr",
            "-o",
            output_path.to_str().expect("output path"),
            input_path.to_str().expect("input path"),
        ],
        b"",
    );
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let written = fs::read_to_string(&output_path).expect("read output");
    assert_eq!(written, "<p>평양 냉면</p>");

    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn content_type_plain_text_and_markdown_work() {
    let plain = run_cli(&["-t", "text/plain"], "漢字".as_bytes());
    assert!(plain.status.success(), "stderr: {}", String::from_utf8_lossy(&plain.stderr));
    assert_eq!(String::from_utf8(plain.stdout).unwrap(), "한자");

    let markdown = run_cli(&["-t", "text/markdown"], "*漢字*".as_bytes());
    assert!(markdown.status.success(), "stderr: {}", String::from_utf8_lossy(&markdown.stderr));
    let out = String::from_utf8(markdown.stdout).unwrap();
    assert!(out.contains("한자"), "output: {out}");
}

#[test]
fn quote_cite_and_stop_options_work() {
    let quote = run_cli(&["-e", "utf-8", "-q", "guillemets"], "'a' \"b\" c".as_bytes());
    assert!(quote.status.success(), "stderr: {}", String::from_utf8_lossy(&quote.stderr));
    let out = String::from_utf8(quote.stdout).unwrap();
    assert!(out.contains("&#x3008;a&#x3009; &#x300a;b&#x300b; c"), "output: {out}");

    let cite = run_cli(&["-c", "angle-quotes-with-cite"], "<p>&lt;&lt;無情&gt;&gt;</p>".as_bytes());
    assert!(cite.status.success(), "stderr: {}", String::from_utf8_lossy(&cite.stderr));
    let out = String::from_utf8(cite.stdout).unwrap();
    assert!(out.contains("<cite>무정</cite>"), "output: {out}");

    let stop = run_cli(&["-s", "horizontal"], "봄。 여름".as_bytes());
    assert!(stop.status.success(), "stderr: {}", String::from_utf8_lossy(&stop.stderr));
    let out = String::from_utf8(stop.stdout).unwrap();
    assert!(out.contains("봄. 여름"), "output: {out}");
}

#[test]
fn no_quote_and_maintain_hanja_work() {
    let no_quote = run_cli(&["-e", "utf-8", "--no-quote"], "\"A\"".as_bytes());
    assert!(no_quote.status.success(), "stderr: {}", String::from_utf8_lossy(&no_quote.stderr));
    assert_eq!(String::from_utf8(no_quote.stdout).unwrap(), "\"A\"");

    let maintain_hanja = run_cli(&["--maintain-hanja"], "<p>漢字</p>".as_bytes());
    assert!(
        maintain_hanja.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&maintain_hanja.stderr)
    );
    assert_eq!(String::from_utf8(maintain_hanja.stdout).unwrap(), "<p>漢字</p>");
}

#[test]
fn render_hanja_and_initial_sound_law_options_work() {
    let render = run_cli(&["-r", "hanja-in-parentheses"], "<p>漢字</p>".as_bytes());
    assert!(render.status.success(), "stderr: {}", String::from_utf8_lossy(&render.stderr));
    assert_eq!(String::from_utf8(render.stdout).unwrap(), "<p>한자(漢字)</p>");

    let with_law = run_cli(&[], "<p>六</p>".as_bytes());
    assert!(with_law.status.success(), "stderr: {}", String::from_utf8_lossy(&with_law.stderr));
    assert_eq!(String::from_utf8(with_law.stdout).unwrap(), "<p>육</p>");

    let without_law = run_cli(&["-I"], "<p>六</p>".as_bytes());
    assert!(
        without_law.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&without_law.stderr)
    );
    assert_eq!(String::from_utf8(without_law.stdout).unwrap(), "<p>륙</p>");
}

#[test]
fn custom_hanja_readings_and_dictionary_work() {
    let with_reading = run_cli(&["-R", "孫文:쑨원"], "<p>孫文</p>".as_bytes());
    assert!(
        with_reading.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&with_reading.stderr)
    );
    assert_eq!(String::from_utf8(with_reading.stdout).unwrap(), "<p>쑨원</p>");

    let dict_path = temp_path("dict", "tsv");
    fs::write(&dict_path, "毛澤東\t마오쩌둥\n").expect("write dictionary");
    let with_dict = run_cli(&["--dict", dict_path.to_str().unwrap()], "<p>毛澤東</p>".as_bytes());
    assert!(with_dict.status.success(), "stderr: {}", String::from_utf8_lossy(&with_dict.stderr));
    assert_eq!(String::from_utf8(with_dict.stdout).unwrap(), "<p>마오쩌둥</p>");
    let _ = fs::remove_file(dict_path);
}

#[test]
fn no_kr_stdict_works() {
    let with_std = run_cli(&[], "<p>困難</p>".as_bytes());
    assert!(with_std.status.success(), "stderr: {}", String::from_utf8_lossy(&with_std.stderr));
    assert_eq!(String::from_utf8(with_std.stdout).unwrap(), "<p>곤란</p>");

    let without_std = run_cli(&["-S"], "<p>困難</p>".as_bytes());
    assert!(
        without_std.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&without_std.stderr)
    );
    assert_eq!(String::from_utf8(without_std.stdout).unwrap(), "<p>곤난</p>");
}

#[test]
fn short_flags_for_dict_and_no_em_dash_work() {
    let no_em_dash = run_cli(&["-e", "utf-8", "-D"], "a -- b".as_bytes());
    assert!(no_em_dash.status.success(), "stderr: {}", String::from_utf8_lossy(&no_em_dash.stderr));
    assert_eq!(String::from_utf8(no_em_dash.stdout).unwrap(), "a -- b");

    let dict_path = temp_path("dict_short", "tsv");
    fs::write(&dict_path, "毛澤東\t마오쩌둥\n").expect("write dictionary");
    let with_dict = run_cli(&["--dict", dict_path.to_str().unwrap()], "<p>毛澤東</p>".as_bytes());
    assert!(with_dict.status.success(), "stderr: {}", String::from_utf8_lossy(&with_dict.stderr));
    assert_eq!(String::from_utf8(with_dict.stdout).unwrap(), "<p>마오쩌둥</p>");
    let _ = fs::remove_file(dict_path);
}

#[test]
fn version_and_debug_options_work() {
    let version = run_cli(&["-v"], b"");
    assert!(version.status.success(), "stderr: {}", String::from_utf8_lossy(&version.stderr));
    assert_eq!(String::from_utf8(version.stdout).unwrap().trim(), env!("CARGO_PKG_VERSION"));

    let debug = run_cli(&["--debug", "-p", "ko-kr"], "<p>漢字</p>".as_bytes());
    assert!(debug.status.success(), "stderr: {}", String::from_utf8_lossy(&debug.stderr));
    let stderr = String::from_utf8(debug.stderr).unwrap();
    assert!(stderr.contains("encoding:"), "stderr: {stderr}");
}

#[test]
fn utf16_encoding_works_for_file_io() {
    let input_path = temp_path("utf16_in", "html");
    let output_path = temp_path("utf16_out", "html");

    let mut input_bytes = vec![0xFF, 0xFE];
    for unit in "<p>漢字</p>".encode_utf16() {
        input_bytes.extend_from_slice(&unit.to_le_bytes());
    }
    fs::write(&input_path, input_bytes).expect("write utf16 input");

    let output = run_cli(
        &["-e", "utf-16le", "-o", output_path.to_str().unwrap(), input_path.to_str().unwrap()],
        b"",
    );
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let written = fs::read(&output_path).expect("read utf16 output");
    let decoded = decode_utf16le(&written);
    assert_eq!(decoded, "<p>한자</p>");

    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}

#[test]
fn preset_rejects_style_options() {
    let output = run_cli(&["-p", "ko-kr", "-q", "guillemets"], b"");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("--preset rejects style options"), "stderr: {stderr}");
}
