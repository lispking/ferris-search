use assert_cmd::Command;
use tempfile::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("ferris-search").unwrap()
}

#[test]
fn help_flag_succeeds() {
    cmd().arg("--help").assert().success();
}

#[test]
fn version_flag_succeeds() {
    cmd().arg("--version").assert().success();
}

#[test]
fn list_engines_text() {
    cmd()
        .arg("list-engines")
        .assert()
        .success()
        .stdout(predicates::str::contains("bing"));
}

#[test]
fn list_engines_json() {
    cmd()
        .args(["list-engines", "--format", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"all_engines\""));
}

#[test]
fn show_config_text() {
    cmd()
        .arg("show-config")
        .assert()
        .success()
        .stdout(predicates::str::contains("Default engine"));
}

#[test]
fn show_config_json() {
    cmd()
        .args(["show-config", "--format", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"default_search_engine\""));
}

#[test]
fn unknown_subcommand_fails() {
    cmd().arg("nonexistent").assert().failure();
}

#[test]
fn search_missing_query_fails() {
    cmd().arg("search").assert().failure();
}

#[test]
fn fetch_missing_url_fails() {
    cmd().arg("fetch").assert().failure();
}

// ─── index-local ────────────────────────────────────────────────────────────

#[test]
fn index_local_missing_path_fails() {
    cmd().arg("index-local").assert().failure();
}

#[test]
fn index_local_nonexistent_path_fails() {
    let dir = TempDir::new().unwrap();
    let bad_path = dir.path().join("does_not_exist");

    cmd()
        .args(["index-local", "--path"])
        .arg(&bad_path)
        .assert()
        .failure();
}

#[test]
fn index_local_indexes_files() {
    let docs = TempDir::new().unwrap();
    let idx = TempDir::new().unwrap();

    std::fs::write(docs.path().join("hello.md"), "# Hello\n\nWorld").unwrap();
    std::fs::write(docs.path().join("notes.txt"), "Some notes").unwrap();

    cmd()
        .arg("index-local")
        .arg("--path")
        .arg(docs.path())
        .arg("--index-path")
        .arg(idx.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Indexed 2 documents"));
}

#[test]
fn index_local_json_output_is_valid() {
    let docs = TempDir::new().unwrap();
    let idx = TempDir::new().unwrap();

    std::fs::write(docs.path().join("a.md"), "# A\n\nContent").unwrap();

    let output = cmd()
        .arg("index-local")
        .arg("--path")
        .arg(docs.path())
        .arg("--index-path")
        .arg(idx.path())
        .args(["--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("index-local JSON output must be valid JSON");
    assert_eq!(json["indexed"], 1);
    assert!(json["index_path"].is_string());
}

// ─── search-local ───────────────────────────────────────────────────────────

#[test]
fn search_local_missing_query_fails() {
    cmd().arg("search-local").assert().failure();
}

#[test]
fn search_local_missing_index_fails() {
    let dir = TempDir::new().unwrap();
    let bad_idx = dir.path().join("no_index_here");

    cmd()
        .args(["search-local", "anything", "--index-path"])
        .arg(&bad_idx)
        .assert()
        .failure();
}

#[test]
fn search_local_returns_results() {
    let docs = TempDir::new().unwrap();
    let idx = TempDir::new().unwrap();

    std::fs::write(
        docs.path().join("rust.md"),
        "# Rust Guide\n\nRust is a systems programming language focused on safety.",
    )
    .unwrap();

    // Build index first
    cmd()
        .arg("index-local")
        .arg("--path")
        .arg(docs.path())
        .arg("--index-path")
        .arg(idx.path())
        .assert()
        .success();

    // Search
    cmd()
        .args(["search-local", "rust systems", "--index-path"])
        .arg(idx.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Rust Guide"));
}

#[test]
fn search_local_json_output_is_valid() {
    let docs = TempDir::new().unwrap();
    let idx = TempDir::new().unwrap();

    std::fs::write(
        docs.path().join("test.md"),
        "# Test Doc\n\nSome searchable content here.",
    )
    .unwrap();

    cmd()
        .arg("index-local")
        .arg("--path")
        .arg(docs.path())
        .arg("--index-path")
        .arg(idx.path())
        .assert()
        .success();

    let output = cmd()
        .args([
            "search-local",
            "searchable content",
            "--format",
            "json",
            "--index-path",
        ])
        .arg(idx.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("search-local JSON output must be valid JSON");
    assert!(json.is_array());
    let arr = json.as_array().unwrap();
    assert!(!arr.is_empty());
    assert!(arr[0]["title"].is_string());
    assert!(arr[0]["path"].is_string());
    assert!(arr[0]["score"].is_number());
}

// ─── existing JSON output structure validation ──────────────────────────────

#[test]
fn list_engines_json_is_valid() {
    let output = cmd()
        .args(["list-engines", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("list-engines JSON must be valid JSON");
    assert!(json["default_engine"].is_string());
    assert!(json["allowed_engines"].is_array());
    assert!(json["all_engines"].is_array());
}

#[test]
fn show_config_json_is_valid() {
    let output = cmd()
        .args(["show-config", "--format", "json"])
        .output()
        .unwrap();

    assert!(output.status.success());

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("show-config JSON must be valid JSON");
    assert!(json["default_search_engine"].is_string());
    assert!(json["allowed_search_engines"].is_array());
    assert!(json["local_docs_index_path"].is_string());
}
