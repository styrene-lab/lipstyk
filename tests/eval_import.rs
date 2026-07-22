use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use lipstyk::eval::evaluate_manifest;
use lipstyk::eval_import::import_source;
use serde_json::{Value, json};

fn serve_once(body: String) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 4096];
        let _ = stream.read(&mut request).unwrap();
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        )
        .unwrap();
    });
    (url, handle)
}

fn source_json(url: &str, output: &str) -> Value {
    json!({
        "source_version": 1,
        "provider": "hugging_face",
        "dataset": "example/corpus",
        "revision": "0123456789abcdef0123456789abcdef01234567",
        "config": "production",
        "split": "validation",
        "rows_url": url,
        "corpus": {
            "id": "import-test",
            "description": "import integration test",
            "license": "MIT",
            "collected_at": "2026-07-21",
            "output": output
        },
        "evaluation": {"score": "per_100_lines", "threshold": 1.0},
        "fields": {"code": "code", "label": "label", "language": "language", "id": "id", "generator": null},
        "labels": {
            "human": {"label": "human", "generator": null},
            "ai": {"label": "agent", "generator": "fixture-agent"}
        },
        "languages": {"rs": "rust"},
        "selection": {
            "seed": "fixed-seed",
            "scan_rows": 4,
            "strata": [
                {"label": "human", "language": "rust", "limit": 1},
                {"label": "agent", "language": "rust", "limit": 1}
            ]
        }
    })
}

fn rows() -> String {
    json!({"rows": [
        {"row_idx": 10, "row": {"id": "h2", "label": "human", "language": "rs", "code": "fn parse(value: &str) -> bool { !value.is_empty() }"}},
        {"row_idx": 11, "row": {"id": "a1", "label": "ai", "language": "rs", "code": "fn process_data(data: String) -> String { data.clone() }"}},
        {"row_idx": 12, "row": {"id": "h1", "label": "human", "language": "rs", "code": "fn format(value: u32) -> String { value.to_string() }"}},
        {"row_idx": 13, "row": {"id": "a2", "label": "ai", "language": "rs", "code": "fn handle_request(input: String) -> String { input.clone() }"}}
    ]}).to_string()
}

#[test]
fn imports_pinned_rows_deterministically_and_emits_evaluable_corpus() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("source.json");

    let (url, server) = serve_once(rows());
    std::fs::write(
        &source_path,
        serde_json::to_vec_pretty(&source_json(&url, "generated")).unwrap(),
    )
    .unwrap();
    let first = import_source(&source_path).unwrap();
    server.join().unwrap();
    let first_lock = std::fs::read(&first.lock_path).unwrap();

    let (url, server) = serve_once(rows());
    std::fs::write(
        &source_path,
        serde_json::to_vec_pretty(&source_json(&url, "generated")).unwrap(),
    )
    .unwrap();
    let second = import_source(&source_path).unwrap();
    server.join().unwrap();
    let second_lock: Value =
        serde_json::from_slice(&std::fs::read(&second.lock_path).unwrap()).unwrap();
    let first_lock: Value = serde_json::from_slice(&first_lock).unwrap();

    assert_eq!(first.selected, 2);
    assert_eq!(first_lock["samples"], second_lock["samples"]);
    let report = evaluate_manifest(&second.corpus_path, None).unwrap();
    assert_eq!(report.samples.len(), 2);
}

#[test]
fn rejects_moving_revisions_before_fetching() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("source.json");
    let mut source = source_json("http://127.0.0.1:1", "generated");
    source["revision"] = Value::String("main".to_string());
    std::fs::write(&source_path, serde_json::to_vec_pretty(&source).unwrap()).unwrap();

    let error = import_source(&source_path).unwrap_err();
    assert!(error.to_string().contains("40-character"));
}

#[test]
fn rejects_output_path_escape_before_fetching() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("source.json");
    std::fs::write(
        &source_path,
        serde_json::to_vec_pretty(&source_json("http://127.0.0.1:1", "../escape")).unwrap(),
    )
    .unwrap();

    let error = import_source(&source_path).unwrap_err();
    assert!(error.to_string().contains("invalid output path"));
}
