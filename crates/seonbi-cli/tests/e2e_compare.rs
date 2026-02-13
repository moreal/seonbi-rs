use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn run_binary<S: AsRef<OsStr>>(bin_path: &Path, args: &[S], stdin_input: &str) -> Output {
    let mut cmd = Command::new(bin_path);
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn binary");
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        use std::io::Write;
        stdin.write_all(stdin_input.as_bytes()).expect("write stdin");
    }
    child.wait_with_output().expect("wait binary")
}

fn current_bin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_seonbi-cli"))
}

fn original_bin_path() -> Option<PathBuf> {
    let path = env::var("SEONBI_ORIGINAL_BIN").ok()?;
    let path = PathBuf::from(path);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn temp_path(name: &str) -> std::path::PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("seonbi_cli_compare_{name}_{ts}.html"))
}

fn assert_equivalent(args: &[&str], stdin_input: &str) {
    let Some(original_bin) = original_bin_path() else {
        eprintln!("skipping: set SEONBI_ORIGINAL_BIN to run comparison E2E");
        return;
    };
    let ported_bin = current_bin_path();

    let original = run_binary(&original_bin, args, stdin_input);
    let ported = run_binary(&ported_bin, args, stdin_input);

    assert_eq!(
        original.status.code(),
        ported.status.code(),
        "exit code differs\noriginal stderr:\n{}\nported stderr:\n{}",
        String::from_utf8_lossy(&original.stderr),
        String::from_utf8_lossy(&ported.stderr),
    );
    assert_eq!(
        original.stdout, ported.stdout,
        "stdout differs\noriginal stderr:\n{}\nported stderr:\n{}",
        String::from_utf8_lossy(&original.stderr),
        String::from_utf8_lossy(&ported.stderr),
    );
}

#[test]
fn compare_stdin_stdout_ko_kr() {
    assert_equivalent(&["-p", "ko-kr"], "<p>漢字</p>");
}

#[test]
fn compare_stdin_stdout_ko_kp() {
    assert_equivalent(&["-p", "ko-kp"], "<p>平壤 冷麵</p>");
}

#[test]
fn compare_file_io() {
    let Some(original_bin) = original_bin_path() else {
        eprintln!("skipping: set SEONBI_ORIGINAL_BIN to run comparison E2E");
        return;
    };
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

    let original = run_binary(&original_bin, &args_original, "");
    let ported = run_binary(&ported_bin, &args_ported, "");

    assert_eq!(
        original.status.code(),
        ported.status.code(),
        "exit code differs\noriginal stderr:\n{}\nported stderr:\n{}",
        String::from_utf8_lossy(&original.stderr),
        String::from_utf8_lossy(&ported.stderr),
    );

    let written_original = fs::read_to_string(&output_path_original).expect("read original output");
    let written_ported = fs::read_to_string(&output_path_ported).expect("read ported output");
    assert_eq!(written_original, written_ported, "file output differs");

    let _ = fs::remove_file(&input_path);
    let _ = fs::remove_file(&output_path_original);
    let _ = fs::remove_file(&output_path_ported);
}
