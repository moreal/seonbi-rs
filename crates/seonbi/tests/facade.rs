use std::fs;
use std::path::PathBuf;

use seonbi::{Configuration, ContentType, HanjaOption, ko_kp, ko_kr, transform_html_text};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn no_op_config() -> Configuration {
    Configuration {
        debug_logger: None,
        content_type: ContentType::from("text/html"),
        quote: None,
        cite: None,
        arrow: None,
        ellipsis: false,
        em_dash: false,
        stop: None,
        hanja: None,
    }
}

#[test]
fn facade_noop_roundtrip() {
    let dir = fixtures_dir();
    for entry in fs::read_dir(&dir).expect("read fixtures") {
        let path = entry.expect("entry").path();
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        if !file.ends_with(".ko-Kore.html") {
            continue;
        }

        let input = fs::read_to_string(&path).expect("input fixture");
        let out = transform_html_text(&no_op_config(), &input).expect("transform");
        assert_eq!(out, input, "no-op mismatch for {}", file);
    }
}

#[test]
fn facade_ko_kr_fixtures() {
    let dir = fixtures_dir();
    for entry in fs::read_dir(&dir).expect("read fixtures") {
        let path = entry.expect("entry").path();
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        if !file.ends_with(".ko-Kore.html") {
            continue;
        }
        let output_name = file.replace(".ko-Kore.html", ".ko-KR.html");
        let output_path = dir.join(&output_name);
        if !output_path.exists() {
            continue;
        }

        let input = fs::read_to_string(&path).expect("input fixture");
        let expected = fs::read_to_string(&output_path).expect("expected fixture");
        let out = transform_html_text(&ko_kr(), &input).expect("transform");
        assert_eq!(out, expected, "ko-kr mismatch for {}", file);
    }
}

#[test]
fn facade_ko_kp_fixtures() {
    let dir = fixtures_dir();
    for entry in fs::read_dir(&dir).expect("read fixtures") {
        let path = entry.expect("entry").path();
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        if !file.ends_with(".ko-Kore.html") {
            continue;
        }
        let output_name = file.replace(".ko-Kore.html", ".ko-KP.html");
        let output_path = dir.join(&output_name);
        if !output_path.exists() {
            continue;
        }

        let input = fs::read_to_string(&path).expect("input fixture");
        let expected = fs::read_to_string(&output_path).expect("expected fixture");
        let out = transform_html_text(&ko_kp(), &input).expect("transform");
        assert_eq!(out, expected, "ko-kp mismatch for {}", file);
    }
}

#[test]
fn facade_can_disable_hanja() {
    let mut cfg = ko_kr();
    cfg.hanja = None::<HanjaOption>;
    let input = "<p>漢字</p>";
    let out = transform_html_text(&cfg, input).expect("transform");
    assert!(out.contains("漢字"));
}
