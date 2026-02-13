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
            if let Ok(resp) = send_http(self.port, "OPTIONS", "/", None, &[]) {
                if resp.status == 200 {
                    return;
                }
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

    Ok(HttpResponse { status, headers, body: raw[sep + 4..].to_vec() })
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
    "rendering": "hangul-only",
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
    let body = r#"{"content":"<p>漢字</p>","preset":"ko-kr"}"#;

    let original =
        send_http_to(&host, port, "POST", "/", Some(body), &[]).expect("original request");
    let current = send_http(server.port, "POST", "/", Some(body), &[]).expect("current request");

    assert_eq!(original.status, current.status);
    assert_eq!(original.headers.get("content-type"), current.headers.get("content-type"));

    let original_json = parse_json_body(&original.body);
    let current_json = parse_json_body(&current.body);
    assert_eq!(original_json, current_json);
}
