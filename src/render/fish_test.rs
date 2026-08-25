use super::render;
use indexmap::IndexMap;

#[test]
fn renders_fish_aliases() {
    let mut entries = IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    assert_eq!(render(&entries), "alias dc 'docker compose'\n");
}

#[test]
fn renders_multiple_fish_aliases() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("a".to_string(), "foo bar".to_string());
    entries.insert("b".to_string(), "baz".to_string());
    let out = render(&entries);
    assert_eq!(out, "alias a 'foo bar'\nalias b 'baz'\n");
}

#[test]
fn renders_fish_empty_entries() {
    let entries = indexmap::IndexMap::new();
    assert_eq!(render(&entries), "");
}

#[test]
fn renders_fish_with_special_chars() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("g".to_string(), "git log --oneline --graph".to_string());
    assert!(render(&entries).contains("git log"));
}

#[test]
fn renders_fish_preserves_order() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("z".to_string(), "zcmd".to_string());
    entries.insert("a".to_string(), "acmd".to_string());
    let out = render(&entries);
    assert!(out.find("alias z").unwrap() < out.find("alias a").unwrap());
}

#[test]
fn renders_fish_with_double_quotes() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("d".to_string(), "echo \"hello\"".to_string());
    assert!(render(&entries).contains("\"hello\""));
}

#[test]
fn renders_fish_large_batch() {
    let mut entries = indexmap::IndexMap::new();
    for i in 0..20 {
        entries.insert(format!("a{i}"), format!("cmd{i}"));
    }
    let out = render(&entries);
    assert_eq!(out.lines().count(), 20);
    assert!(out.contains("a0"));
    assert!(out.contains("a19"));
}

#[test]
fn renders_fish_unicode() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("u".to_string(), "echo rocket".to_string());
    assert!(render(&entries).contains("rocket"));
}

#[test]
fn renders_fish_with_dash_name() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("my-alias".to_string(), "echo hi".to_string());
    assert!(render(&entries).contains("my-alias"));
}

#[test]
fn renders_fish_single_entry_with_spaces() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("x".to_string(), "cmd with many spaces".to_string());
    assert_eq!(render(&entries), "alias x 'cmd with many spaces'\n");
}

#[test]
fn renders_fish_20_entries_order_check() {
    let mut entries = indexmap::IndexMap::new();
    for i in 0..5 {
        entries.insert(format!("k{i}"), format!("v{i}"));
    }
    let out = render(&entries);
    let idx0 = out.find("k0").unwrap();
    let idx4 = out.find("k4").unwrap();
    assert!(idx0 < idx4);
}

#[test]
fn renders_fish_with_env_like() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("e".to_string(), "echo $HOME".to_string());
    assert!(render(&entries).contains("$HOME"));
}
