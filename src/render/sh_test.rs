use super::render;
use indexmap::IndexMap;

#[test]
fn renders_sh_aliases() {
    let mut entries = IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    assert_eq!(render(&entries), "alias dc='docker compose'\n");
}

#[test]
fn renders_multiple_sh_aliases() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("a".to_string(), "foo bar".to_string());
    entries.insert("b".to_string(), "baz".to_string());
    assert_eq!(render(&entries), "alias a='foo bar'\nalias b='baz'\n");
}

#[test]
fn renders_sh_empty() {
    let entries = indexmap::IndexMap::new();
    assert_eq!(render(&entries), "");
}

#[test]
fn renders_sh_with_dash() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("g".to_string(), "git commit -m test".to_string());
    assert!(render(&entries).contains("git commit"));
}

#[test]
fn renders_sh_preserves_order() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("b".to_string(), "bcmd".to_string());
    entries.insert("a".to_string(), "acmd".to_string());
    let out = render(&entries);
    assert!(out.find("alias b=").unwrap() < out.find("alias a=").unwrap());
}

#[test]
fn renders_sh_large_batch() {
    let mut entries = indexmap::IndexMap::new();
    for i in 0..30 { entries.insert(format!("k{i}"), format!("cmd{i} --flag")); }
    assert_eq!(render(&entries).lines().count(), 30);
}

#[test]
fn renders_sh_with_spaces() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("x".to_string(), "cmd with many spaces".to_string());
    assert_eq!(render(&entries), "alias x='cmd with many spaces'\n");
}

#[test]
fn renders_sh_unicode() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("u".to_string(), "echo hello world".to_string());
    assert!(render(&entries).contains("hello"));
}

#[test]
fn renders_sh_with_env() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("e".to_string(), "echo $HOME".to_string());
    assert!(render(&entries).contains("$HOME"));
}

#[test]
fn renders_sh_order_10() {
    let mut entries = indexmap::IndexMap::new();
    for i in 0..10 { entries.insert(format!("a{i}"), format!("v{i}")); }
    let out = render(&entries);
    assert!(out.find("a0").unwrap() < out.find("a9").unwrap());
}

#[test]
fn renders_sh_dash_name() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("my-alias".to_string(), "echo hi".to_string());
    assert!(render(&entries).contains("my-alias"));
}

#[test]
fn renders_sh_with_numbers() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("n1".to_string(), "echo 123".to_string());
    assert_eq!(render(&entries), "alias n1='echo 123'\n");
}
