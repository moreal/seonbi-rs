use std::fs;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn run_cli(args: &[&str], stdin_input: &str) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_seonbi-cli"));
    cmd.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn cli");
    if !stdin_input.is_empty() {
        let stdin = child.stdin.as_mut().expect("stdin");
        use std::io::Write;
        stdin
            .write_all(stdin_input.as_bytes())
            .expect("write stdin to cli");
    }
    child.wait_with_output().expect("wait cli")
}

fn temp_path(name: &str) -> std::path::PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("seonbi_cli_{name}_{ts}.html"))
}

#[test]
fn transforms_stdin_to_stdout() {
    let output = run_cli(&["-p", "ko-kr"], "<p>漢字</p>");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    assert_eq!(stdout, "<p>한자</p>");
}

#[test]
fn normalizes_preset_name_with_underscore() {
    let output = run_cli(&["-p", "ko_kp"], "<p>六</p>");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    assert_eq!(stdout, "<p>륙</p>");
}

#[test]
fn reads_and_writes_files() {
    let input_path = temp_path("in");
    let output_path = temp_path("out");
    fs::write(&input_path, "<p>平壤 冷麵</p>").expect("write input fixture");

    let output = run_cli(
        &[
            "-p",
            "ko-kr",
            "-o",
            output_path.to_str().expect("output path"),
            input_path.to_str().expect("input path"),
        ],
        "",
    );
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let written = fs::read_to_string(&output_path).expect("read output");
    assert_eq!(written, "<p>평양 냉면</p>");

    let _ = fs::remove_file(input_path);
    let _ = fs::remove_file(output_path);
}
