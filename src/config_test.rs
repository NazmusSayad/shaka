use super::load_config;
use crate::render::Shell;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "shaka-config-test-{}-{}",
        std::process::id(),
        NEXT_DIR.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn config(content: &str) -> (PathBuf, PathBuf) {
    let dir = temp_dir();
    let path = dir.join("config.json");
    fs::write(&path, content).unwrap();
    (dir, path)
}

#[test]
fn loads_a_config_file() {
    let (dir, path) = config(r#"{"gs":"git status","dc":"docker compose"}"#);

    let result = load_config(Shell::Bash, &path).unwrap();

    assert_eq!(result["gs"], "git status");
    assert_eq!(result["dc"], "docker compose");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rejects_a_directory_path() {
    let (dir, _) = config(r#"{"gs":"git status"}"#);

    let error = load_config(Shell::Bash, &dir).unwrap_err();

    assert!(error.contains("failed reading"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn resolves_shell_filters() {
    let (dir, path) = config(
        r#"{
            "all":"always",
            "bash":{"cmd":"bash only","shell":"bash"},
            "fish":{"cmd":"fish only","shell":"fish"},
            "unix":{"cmd":"bash or zsh","shell":["bash","zsh"]},
            "notBash":{"cmd":"not bash","shellExclude":"bash"}
        }"#,
    );

    let result = load_config(Shell::Bash, &path).unwrap();

    assert_eq!(
        result.keys().map(String::as_str).collect::<Vec<_>>(),
        ["all", "bash", "unix"]
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn resolves_platform_filters() {
    let current = std::env::consts::OS;
    let other = if current == "windows" {
        "linux"
    } else {
        "windows"
    };
    let (dir, path) = config(&format!(
        r#"{{
            "current":{{"cmd":"yes","platform":"{current}"}},
            "other":{{"cmd":"no","platform":"{other}"}},
            "excluded":{{"cmd":"no","platformExclude":"{current}"}}
        }}"#
    ));

    let result = load_config(Shell::Bash, &path).unwrap();

    assert_eq!(
        result.keys().map(String::as_str).collect::<Vec<_>>(),
        ["current"]
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn pair_entries_use_the_last_value() {
    let (dir, path) = config(r#"[["first","1"],["same","old"],["last","3"],["same","new"]]"#);

    let result = load_config(Shell::Bash, &path).unwrap();

    assert_eq!(
        result.keys().map(String::as_str).collect::<Vec<_>>(),
        ["first", "last", "same"]
    );
    assert_eq!(result["same"], "new");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rejects_include_and_exclude_together() {
    let (dir, path) = config(r#"{"bad":{"cmd":"no","shell":"bash","shellExclude":"fish"}}"#);

    let error = load_config(Shell::Bash, &path).unwrap_err();

    assert_eq!(error, "shell include and exclude are mutually exclusive");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rejects_invalid_json() {
    let (dir, path) = config(r#"{"gs":"git status",}"#);

    let error = load_config(Shell::Bash, &path).unwrap_err();

    assert!(error.contains("failed parsing JSON"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn reports_a_missing_custom_path() {
    let dir = temp_dir();
    let path = dir.join("missing.json");

    let error = load_config(Shell::Bash, &path).unwrap_err();

    assert!(error.contains("failed reading"));
    fs::remove_dir_all(dir).unwrap();
}
