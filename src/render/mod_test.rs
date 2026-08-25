use super::{render, Shell};
use indexmap::IndexMap;

#[test]
fn dispatches_bash_to_sh_renderer() {
    let mut entries = IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    assert_eq!(render(Shell::Bash, &entries), "alias dc='docker compose'\n");
}

#[test]
fn dispatches_zsh_to_sh_renderer() {
    let mut entries = IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    assert_eq!(render(Shell::Zsh, &entries), "alias dc='docker compose'\n");
}

#[test]
fn dispatches_fish_to_fish_renderer() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    assert_eq!(render(Shell::Fish, &entries), "alias dc 'docker compose'\n");
}

#[test]
fn dispatches_pwsh_to_functions() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    let out = render(Shell::Pwsh, &entries);
    assert!(out.contains("Remove-Alias"));
    assert!(out.contains("function dc"));
}

#[test]
fn dispatches_pwsh_conflict_no_remove() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    let out = render(Shell::PwshConflict, &entries);
    assert!(!out.contains("Remove-Alias"));
    assert!(out.contains("function dc"));
}

#[test]
fn dispatch_bash_and_zsh_same_output() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("a".to_string(), "echo hi".to_string());
    assert_eq!(render(Shell::Bash, &entries), render(Shell::Zsh, &entries));
}

#[test]
fn dispatch_empty_entries_all_shells() {
    let entries = indexmap::IndexMap::new();
    for s in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh, Shell::PwshConflict] {
        assert_eq!(render(s, &entries), "");
    }
}

#[test]
fn dispatch_large_entries_pwsh_vs_sh() {
    let mut entries = indexmap::IndexMap::new();
    for i in 0..10 { entries.insert(format!("k{i}"), format!("cmd{i}")); }
    let bash_out = render(Shell::Bash, &entries);
    let pwsh_out = render(Shell::Pwsh, &entries);
    assert_ne!(bash_out, pwsh_out);
    assert!(bash_out.contains("alias"));
    assert!(pwsh_out.contains("function"));
}

#[test]
fn dispatch_fish_vs_sh_quoting_differs() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("q".to_string(), "echo hi".to_string());
    let fish = render(Shell::Fish, &entries);
    let sh = render(Shell::Bash, &entries);
    assert_ne!(fish, sh);
    assert!(fish.contains("alias q '"));
    assert!(sh.contains("alias q='"));
}

#[test]
fn dispatch_all_shells_contain_alias_name() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("mycmd".to_string(), "echo test".to_string());
    for s in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh, Shell::PwshConflict] {
        assert!(render(s, &entries).contains("mycmd"));
    }
}

#[test]
fn dispatch_pwsh_and_pwsh_conflict_differ() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("x".to_string(), "echo hi".to_string());
    assert_ne!(render(Shell::Pwsh, &entries), render(Shell::PwshConflict, &entries));
}

#[test]
fn dispatch_sh_handles_many() {
    let mut entries = indexmap::IndexMap::new();
    for i in 0..5 { entries.insert(format!("a{i}"), format!("cmd{i}")); }
    let out_bash = render(Shell::Bash, &entries);
    let out_zsh = render(Shell::Zsh, &entries);
    assert_eq!(out_bash, out_zsh);
    assert_eq!(out_bash.lines().count(), 5);
}

#[test]
fn dispatch_fish_handles_many() {
    let mut entries = indexmap::IndexMap::new();
    for i in 0..5 { entries.insert(format!("f{i}"), format!("cmd{i}")); }
    let out = render(Shell::Fish, &entries);
    assert_eq!(out.lines().count(), 5);
    assert!(out.contains("f0"));
}

#[test]
fn dispatch_pwsh_contains_at_args_or_keyword() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("a".to_string(), "git status".to_string());
    let out = render(Shell::Pwsh, &entries);
    assert!(out.contains("@args") || out.contains("git status"));
}

#[test]
fn dispatch_bash_single() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("single".to_string(), "echo one".to_string());
    assert_eq!(render(Shell::Bash, &entries), "alias single='echo one'\n");
}

#[test]
fn dispatch_zsh_single() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("single".to_string(), "echo one".to_string());
    assert_eq!(render(Shell::Zsh, &entries), "alias single='echo one'\n");
}

#[test]
fn dispatch_pwsh_single() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("single".to_string(), "echo one".to_string());
    let out = render(Shell::Pwsh, &entries);
    assert!(out.contains("function single"));
}

#[test]
fn dispatch_order_preserved_across_shells() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("z".to_string(), "zcmd".to_string());
    entries.insert("a".to_string(), "acmd".to_string());
    for s in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        let out = render(s, &entries);
        // search for the alias name with surrounding context to avoid matching inside "alias"
        let pos_z = out.find(" z").or_else(|| out.find("\"z\"")).or_else(|| out.find("'z'")).unwrap_or_else(|| out.find("z").unwrap());
        let pos_a = out.find(" a").or_else(|| out.find("\"a\"")).or_else(|| out.find("'a'")).unwrap_or_else(|| out.find("a").unwrap());
        // fallback to alias name search
        let pz = out.find("zcmd").unwrap();
        let pa = out.find("acmd").unwrap();
        assert!(pz < pa, "order failed for shell {:?}", s as u8);
        // also ensure the alias definitions are ordered
        assert!(pos_z < pos_a || pz < pa);
    }
}
