use std::collections::BTreeMap;
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use serde_json::Value;

struct RunningServer {
    child: Child,
    port: u16,
}

impl RunningServer {
    fn spawn(extra_args: &[&str]) -> Self {
        let port = find_free_port();
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_seonbi-api"));
        cmd.arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(port.to_string())
            .args(extra_args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let child = cmd.spawn().expect("spawn seonbi-api");
        let server = Self { child, port };
        server.wait_until_ready();
        server
    }

    fn wait_until_ready(&self) {
        for _ in 0..80 {
            if let Ok(resp) = send_http(self.port, "OPTIONS", "/", None, &[])
                && resp.status == 200
            {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("server did not become ready on port {}", self.port);
    }
}

impl Drop for RunningServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[derive(Debug)]
struct HttpResponse {
    status: u16,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

fn find_free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind free port");
    listener.local_addr().expect("local addr").port()
}

fn send_http(
    port: u16,
    method: &str,
    target: &str,
    body: Option<&str>,
    headers: &[(&str, &str)],
) -> Result<HttpResponse, String> {
    send_http_to("127.0.0.1", port, method, target, body, headers)
}

fn send_http_to(
    host: &str,
    port: u16,
    method: &str,
    target: &str,
    body: Option<&str>,
    headers: &[(&str, &str)],
) -> Result<HttpResponse, String> {
    let mut stream = TcpStream::connect((host, port)).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(3))).map_err(|e| e.to_string())?;

    let body_bytes = body.unwrap_or("").as_bytes();

    let mut req =
        format!("{method} {target} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n");
    for (k, v) in headers {
        req.push_str(k);
        req.push_str(": ");
        req.push_str(v);
        req.push_str("\r\n");
    }
    if body.is_some() {
        req.push_str("Content-Type: application/json\r\n");
    }
    req.push_str(&format!("Content-Length: {}\r\n\r\n", body_bytes.len()));

    stream
        .write_all(req.as_bytes())
        .and_then(|_| stream.write_all(body_bytes))
        .map_err(|e| e.to_string())?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).map_err(|e| e.to_string())?;

    parse_http_response(&raw)
}

fn parse_http_response(raw: &[u8]) -> Result<HttpResponse, String> {
    let sep = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| "invalid http response".to_string())?;

    let header_text = String::from_utf8_lossy(&raw[..sep]);
    let mut lines = header_text.lines();
    let status_line = lines.next().ok_or_else(|| "missing status line".to_string())?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| "missing status code".to_string())?
        .parse::<u16>()
        .map_err(|e| e.to_string())?;

    let mut headers = BTreeMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    let mut body = raw[sep + 4..].to_vec();
    if headers
        .get("transfer-encoding")
        .is_some_and(|value| value.to_ascii_lowercase().contains("chunked"))
    {
        body = decode_chunked_body(&body)?;
    }

    Ok(HttpResponse { status, headers, body })
}

fn decode_chunked_body(body: &[u8]) -> Result<Vec<u8>, String> {
    let mut pos = 0usize;
    let mut out = Vec::new();

    loop {
        let size_end = body[pos..]
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or_else(|| "invalid chunked body: missing chunk size terminator".to_string())?
            + pos;

        let size_line = std::str::from_utf8(&body[pos..size_end]).map_err(|e| e.to_string())?;
        let size_hex = size_line.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_hex, 16)
            .map_err(|e| format!("invalid chunk size `{size_hex}`: {e}"))?;

        pos = size_end + 2;
        if size == 0 {
            break;
        }
        if pos + size > body.len() {
            return Err("invalid chunked body: truncated chunk payload".to_string());
        }
        out.extend_from_slice(&body[pos..pos + size]);
        pos += size;

        if body.get(pos..pos + 2) != Some(b"\r\n") {
            return Err("invalid chunked body: missing chunk terminator".to_string());
        }
        pos += 2;
    }

    Ok(out)
}

fn parse_json_body(body: &[u8]) -> Value {
    serde_json::from_slice(body).expect("json body")
}

fn original_api_endpoint() -> Option<(String, u16)> {
    let raw = env::var("SEONBI_ORIGINAL_API_URL").ok()?;
    let raw = raw.trim();
    let without_scheme = raw.strip_prefix("http://")?;
    let no_path = without_scheme.trim_end_matches('/').split('/').next()?;
    let (host, port) = no_path.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;
    Some((host.to_string(), port))
}

#[derive(Debug, Clone, Copy)]
enum BodyComparison {
    ExactJson,
    ErrorShape,
}

#[derive(Debug, Clone, Copy)]
struct ApiCompareCase {
    name: &'static str,
    method: &'static str,
    target: &'static str,
    body: Option<&'static str>,
    comparison: BodyComparison,
}

fn assert_equivalent_headers(case_name: &str, original: &HttpResponse, current: &HttpResponse) {
    for key in
        ["content-type", "access-control-allow-headers", "vary", "access-control-allow-origin"]
    {
        assert_eq!(
            original.headers.get(key),
            current.headers.get(key),
            "header mismatch for {case_name}: {key}"
        );
    }

    let original_server = original
        .headers
        .get("server")
        .unwrap_or_else(|| panic!("missing server header in original for {case_name}"));
    let current_server = current
        .headers
        .get("server")
        .unwrap_or_else(|| panic!("missing server header in ported for {case_name}"));
    assert!(
        original_server.starts_with("Seonbi/"),
        "unexpected original server header for {case_name}: {original_server}"
    );
    assert!(
        current_server.starts_with("Seonbi/"),
        "unexpected ported server header for {case_name}: {current_server}"
    );
}

fn assert_equivalent_case(
    original_host: &str,
    original_port: u16,
    current_port: u16,
    case: ApiCompareCase,
) {
    let original =
        send_http_to(original_host, original_port, case.method, case.target, case.body, &[])
            .unwrap_or_else(|e| panic!("original request failed for {}: {e}", case.name));
    let current = send_http(current_port, case.method, case.target, case.body, &[])
        .unwrap_or_else(|e| panic!("ported request failed for {}: {e}", case.name));

    assert_eq!(original.status, current.status, "status mismatch for {}", case.name);
    assert_equivalent_headers(case.name, &original, &current);

    match case.comparison {
        BodyComparison::ExactJson => {
            let original_json = parse_json_body(&original.body);
            let current_json = parse_json_body(&current.body);
            assert_eq!(original_json, current_json, "json body mismatch for {}", case.name);
        }
        BodyComparison::ErrorShape => {
            let original_json = parse_json_body(&original.body);
            let current_json = parse_json_body(&current.body);
            assert_eq!(
                original_json["success"], current_json["success"],
                "success mismatch for {}",
                case.name
            );
            assert_eq!(
                original_json["success"], false,
                "expected error response for {}",
                case.name
            );
            assert!(
                original_json["message"].as_str().is_some_and(|m| !m.trim().is_empty()),
                "original message is empty for {}",
                case.name
            );
            assert!(
                current_json["message"].as_str().is_some_and(|m| !m.trim().is_empty()),
                "ported message is empty for {}",
                case.name
            );
        }
    }
}

#[test]
fn post_with_preset_returns_transformed_content() {
    let server = RunningServer::spawn(&[]);
    let body = r#"{"content":"<p>漢字</p>","preset":"ko-kr"}"#;
    let res = send_http(server.port, "POST", "/", Some(body), &[]).expect("request");

    assert_eq!(res.status, 200);
    assert_eq!(res.headers.get("content-type"), Some(&"application/json".to_string()));
    assert!(
        res.headers.get("server").is_some_and(|v| v.starts_with("Seonbi/")),
        "server header: {:?}",
        res.headers.get("server")
    );

    let parsed = parse_json_body(&res.body);
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["content"], "<p>한자</p>");
    assert_eq!(parsed["contentType"], "text/html");
}

#[test]
fn post_with_markdown_content_type_returns_markdown_output() {
    let server = RunningServer::spawn(&[]);
    let body = r#"{"content":"\"abc\"","preset":"ko-kr","contentType":"text/markdown"}"#;
    let res = send_http(server.port, "POST", "/", Some(body), &[]).expect("request");

    assert_eq!(res.status, 200);
    let parsed = parse_json_body(&res.body);
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["contentType"], "text/markdown");
    assert_eq!(parsed["content"], "“abc”\n");
    assert!(parsed.get("resultHtml").is_none());
}

#[test]
fn post_deprecated_keys_return_warnings() {
    let server = RunningServer::spawn(&[]);

    let source_html = send_http(
        server.port,
        "POST",
        "/",
        Some(r#"{"sourceHtml":"<p>漢字</p>","preset":"ko-kr"}"#),
        &[],
    )
    .expect("request");
    assert_eq!(source_html.status, 200);
    let parsed = parse_json_body(&source_html.body);
    assert_eq!(parsed["warnings"][0], "key \"sourceHtml\" is deprecated in favour of \"content\"");

    let xhtml = send_http(
        server.port,
        "POST",
        "/",
        Some(r#"{"content":"<p>漢字</p>","xhtml":true,"preset":"ko-kr"}"#),
        &[],
    )
    .expect("request");
    assert_eq!(xhtml.status, 200);
    let parsed = parse_json_body(&xhtml.body);
    assert_eq!(parsed["warnings"][0], "key \"xhtml\" is deprecated in favour of \"contentType\"");
    assert_eq!(parsed["contentType"], "application/xhtml+xml");
}

#[test]
fn invalid_requests_return_400() {
    let server = RunningServer::spawn(&[]);

    let invalid_preset =
        send_http(server.port, "POST", "/", Some(r#"{"content":"x","preset":"invalid"}"#), &[])
            .expect("request");
    assert_eq!(invalid_preset.status, 400);
    let parsed = parse_json_body(&invalid_preset.body);
    assert_eq!(parsed["success"], false);

    let invalid_json =
        send_http(server.port, "POST", "/", Some("{not-json}"), &[]).expect("request");
    assert_eq!(invalid_json.status, 400);
    let parsed = parse_json_body(&invalid_json.body);
    assert_eq!(parsed["success"], false);
}

#[test]
fn options_and_method_not_allowed_work() {
    let server = RunningServer::spawn(&["--allow-origin", "https://example.com"]);

    let options = send_http(server.port, "OPTIONS", "*", None, &[]).expect("request");
    assert_eq!(options.status, 200);
    assert_eq!(options.headers.get("access-control-allow-headers"), Some(&"*".to_string()));
    assert_eq!(options.headers.get("vary"), Some(&"origin".to_string()));
    assert_eq!(
        options.headers.get("access-control-allow-origin"),
        Some(&"https://example.com".to_string())
    );

    let put = send_http(server.port, "PUT", "/", None, &[]).expect("request");
    assert_eq!(put.status, 405);
    let parsed = parse_json_body(&put.body);
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["message"], "Unsupported method: PUT");
}

#[test]
fn hanja_dictionary_and_use_dictionaries_work() {
    let server = RunningServer::spawn(&[]);

    let body = r#"
{
  "content": "<p>困難 孫文</p>",
  "contentType": "text/html",
  "hanja": {
    "rendering": "HangulOnly",
    "reading": {
      "initialSoundLaw": true,
      "dictionary": {"孫文": "쑨원"},
      "useDictionaries": ["kr-stdict"]
    }
  }
}
"#;

    let res = send_http(server.port, "POST", "/", Some(body), &[]).expect("request");
    assert_eq!(res.status, 200);
    let parsed = parse_json_body(&res.body);
    assert_eq!(parsed["success"], true);
    assert_eq!(parsed["content"], "<p><![CDATA[곤란]]> <![CDATA[쑨원]]></p>");
}

#[test]
fn compares_with_original_when_configured() {
    let Some((host, port)) = original_api_endpoint() else {
        eprintln!("skipping: set SEONBI_ORIGINAL_API_URL (e.g. http://127.0.0.1:3800)");
        return;
    };

    let server = RunningServer::spawn(&[]);
    let cases = [
        ApiCompareCase {
            name: "post preset ko-kr",
            method: "POST",
            target: "/",
            body: Some(r#"{"content":"<p>漢字</p>","preset":"ko-kr"}"#),
            comparison: BodyComparison::ExactJson,
        },
        ApiCompareCase {
            name: "post quote at text end",
            method: "POST",
            target: "/",
            body: Some(r#"{"content":"<p>\"abc\"</p>","preset":"ko-kr"}"#),
            comparison: BodyComparison::ExactJson,
        },
        ApiCompareCase {
            name: "post markdown preset ko-kr",
            method: "POST",
            target: "/",
            body: Some(r#"{"content":"\"abc\"","preset":"ko-kr","contentType":"text/markdown"}"#),
            comparison: BodyComparison::ExactJson,
        },
        ApiCompareCase {
            name: "post with individual options",
            method: "POST",
            target: "/",
            body: Some(
                r#"{"content":"<p>'A' &lt;&lt;無情&gt;&gt; ... -- <-></p>","contentType":"text/html","quote":"guillemets","cite":"angle-quotes-with-cite","arrow":{"bidir":true,"double":true},"ellipsis":true,"emDash":true,"stop":"horizontal"}"#,
            ),
            comparison: BodyComparison::ExactJson,
        },
        ApiCompareCase {
            name: "post deprecated sourceHtml",
            method: "POST",
            target: "/",
            body: Some(r#"{"sourceHtml":"<p>漢字</p>","preset":"ko-kr"}"#),
            comparison: BodyComparison::ExactJson,
        },
        ApiCompareCase {
            name: "post deprecated xhtml",
            method: "POST",
            target: "/",
            body: Some(r#"{"content":"<p>漢字</p>","xhtml":true,"preset":"ko-kr"}"#),
            comparison: BodyComparison::ExactJson,
        },
        ApiCompareCase {
            name: "post invalid preset",
            method: "POST",
            target: "/",
            body: Some(r#"{"content":"x","preset":"invalid"}"#),
            comparison: BodyComparison::ExactJson,
        },
        ApiCompareCase {
            name: "post malformed json",
            method: "POST",
            target: "/",
            body: Some("{not-json}"),
            comparison: BodyComparison::ErrorShape,
        },
        ApiCompareCase {
            name: "post hanja dictionary + useDictionaries",
            method: "POST",
            target: "/",
            body: Some(
                r#"{"content":"<p>困難 孫文</p>","contentType":"text/html","hanja":{"rendering":"hangul-only","reading":{"initialSoundLaw":true,"dictionary":{"孫文":"쑨원"},"useDictionaries":["kr-stdict"]}}}"#,
            ),
            comparison: BodyComparison::ExactJson,
        },
        ApiCompareCase {
            name: "options wildcard",
            method: "OPTIONS",
            target: "*",
            body: None,
            comparison: BodyComparison::ExactJson,
        },
        ApiCompareCase {
            name: "unsupported method put",
            method: "PUT",
            target: "/",
            body: None,
            comparison: BodyComparison::ExactJson,
        },
    ];

    for case in cases {
        assert_equivalent_case(&host, port, server.port, case);
    }
}
