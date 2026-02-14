use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn run_binary<S: AsRef<OsStr>>(bin_path: &Path, args: &[S], stdin_input: &str) -> Output {
    let mut cmd = Command::new(bin_path);
    cmd.args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
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
    if path.exists() { Some(path) } else { None }
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
    f(&original_bin);
}

fn assert_equivalent(original_bin: &Path, args: &[&str], stdin_input: &str) {
    let ported_bin = current_bin_path();

    let original = run_binary(&original_bin, args, stdin_input);
    let ported = run_binary(&ported_bin, args, stdin_input);

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

#[test]
fn compare_stdin_stdout_matrix() {
    with_original_bin(|original_bin| {
        let cases: &[(&str, &[&str], &str)] = &[
            ("preset ko-kr", &["-p", "ko-kr"], "<p>漢字</p>"),
            ("preset ko-kp", &["-p", "ko-kp"], "<p>平壤 冷麵</p>"),
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
        ];

        for (name, args, stdin_input) in cases {
            assert_equivalent(original_bin, args, stdin_input);
            eprintln!("ok: {name}");
        }
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

        let original = run_binary(&original_bin, &args_original, "");
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

        let original = run_binary(&original_bin, &args_original, "");
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
