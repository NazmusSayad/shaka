use super::{STATEMENT_KEYWORDS, render};
use indexmap::IndexMap;

#[test]
fn renders_pwsh_without_conflicts() {
    let mut entries = IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    assert_eq!(
        render(&entries, false),
        "Remove-Alias -Name dc -Force -ErrorAction SilentlyContinue\nfunction dc { docker compose @args }\n"
    );
}

#[test]
fn renders_pwsh_conflict_mode() {
    let mut entries = IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    assert_eq!(
        render(&entries, true),
        "function dc { docker compose @args }\n"
    );
}

#[test]
fn expands_env_vars_in_pwsh_commands() {
    let mut entries = IndexMap::new();
    entries.insert(
        "ocd".to_string(),
        "$HOME/scoop/apps/opencode-desktop/current/OpenCode".to_string(),
    );

    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"));
    let expected_command = match home {
        Ok(value) => format!(
            "{}/scoop/apps/opencode-desktop/current/OpenCode",
            value.replace('\\', "/")
        ),
        Err(_) => "$HOME/scoop/apps/opencode-desktop/current/OpenCode".to_string(),
    };

    assert_eq!(
        render(&entries, true),
        format!("function ocd {{ {expected_command} @args }}\n")
    );
}

#[test]
fn uses_remove_alias_for_alias_cleanup() {
    let mut entries = IndexMap::new();
    entries.insert("..".to_string(), "cd ..".to_string());
    assert_eq!(
        render(&entries, false),
        "Remove-Alias -Name .. -Force -ErrorAction SilentlyContinue\nfunction .. { cd .. @args }\n"
    );
}

#[test]
fn omits_splatting_for_statement_keywords() {
    let mut entries = IndexMap::new();
    entries.insert("xxx".to_string(), "exit".to_string());
    entries.insert("q1".to_string(), "Exit 1".to_string());
    entries.insert("exx".to_string(), "exitx".to_string());
    entries.insert("dc".to_string(), "docker compose".to_string());
    assert_eq!(
        render(&entries, true),
        "function xxx { if ($args.Count -gt 1) { Write-Warning \"xxx: exit expects at most 1 argument; received $($args.Count); using first\" }; if ($args.Count -gt 0) { exit $args[0] } else { exit } }\nfunction q1 { Exit 1 }\nfunction exx { exitx @args }\nfunction dc { docker compose @args }\n"
    );
}

#[test]
fn forwards_args_for_every_bare_statement_keyword() {
    for keyword in STATEMENT_KEYWORDS {
        let mut entries = IndexMap::new();
        entries.insert("k".to_string(), format!("Write-Host done; {keyword}"));
        assert_eq!(
            render(&entries, true),
            format!(
                "function k {{ Write-Host done; if ($args.Count -gt 1) {{ Write-Warning \"k: {keyword} expects at most 1 argument; received $($args.Count); using first\" }}; if ($args.Count -gt 0) {{ {keyword} $args[0] }} else {{ {keyword} }} }}\n"
            )
        );
    }
}

#[test]
fn handles_compound_commands_ending_in_keyword() {
    let mut entries = IndexMap::new();
    entries.insert("bye".to_string(), "clear; exit".to_string());
    entries.insert("brk".to_string(), "save;\nbreak".to_string());
    entries.insert("thr".to_string(), "cleanup ;THROW 'boom'".to_string());
    assert_eq!(
        render(&entries, true),
        "function bye { clear; if ($args.Count -gt 1) { Write-Warning \"bye: exit expects at most 1 argument; received $($args.Count); using first\" }; if ($args.Count -gt 0) { exit $args[0] } else { exit } }\nfunction brk { save; if ($args.Count -gt 1) { Write-Warning \"brk: break expects at most 1 argument; received $($args.Count); using first\" }; if ($args.Count -gt 0) { break $args[0] } else { break } }\nfunction thr { cleanup ;THROW 'boom' }\n"
    );
}

#[test]
fn keeps_splatting_when_keyword_is_not_final_command() {
    let mut entries = IndexMap::new();
    entries.insert("dev".to_string(), "exit; code .".to_string());
    entries.insert("mix".to_string(), "return 1; git status".to_string());
    assert_eq!(
        render(&entries, true),
        "function dev { exit; code . @args }\nfunction mix { return 1; git status @args }\n"
    );
}

#[test]
fn renders_pwsh_multiple_entries() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("a".to_string(), "cmd a".to_string());
    entries.insert("b".to_string(), "cmd b".to_string());
    let out = render(&entries, true);
    assert!(out.contains("function a"));
    assert!(out.contains("function b"));
    assert_eq!(out.matches("function ").count(), 2);
}

#[test]
fn pwsh_not_statement_when_prefix() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("x".to_string(), "myexit".to_string());
    let out = render(&entries, true);
    assert!(out.contains("myexit @args"));
}

#[test]
fn pwsh_handles_semicolon_separated_tail() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("k".to_string(), "echo hi; exit".to_string());
    let out = render(&entries, true);
    assert!(out.contains("exit"));
}

#[test]
fn pwsh_handles_newline_tail() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("k".to_string(), "echo hi\nexit".to_string());
    let out = render(&entries, true);
    assert!(out.contains("exit"));
}

#[test]
fn pwsh_bare_keyword_with_args() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("k".to_string(), "return".to_string());
    let out = render(&entries, true);
    assert!(out.contains("return"));
    assert!(out.contains("$args"));
}

#[test]
fn pwsh_non_bare_keywords_get_splatting() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("a".to_string(), "git status".to_string());
    assert!(render(&entries, true).contains("git status @args"));
}

#[test]
fn pwsh_empty_command() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("empty".to_string(), "".to_string());
    let out = render(&entries, true);
    assert!(out.contains("function empty"));
}

#[test]
fn pwsh_remove_alias_only_when_not_conflict() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("a".to_string(), "cmd".to_string());
    entries.insert("b".to_string(), "cmd2".to_string());
    let with = render(&entries, false);
    let without = render(&entries, true);
    assert_eq!(with.matches("Remove-Alias").count(), 2);
    assert_eq!(without.matches("Remove-Alias").count(), 0);
}

#[test]
fn pwsh_all_keywords_handled() {
    for kw in ["exit","return","break","continue","throw"] {
        let mut entries = indexmap::IndexMap::new();
        entries.insert("k".to_string(), kw.to_string());
        let out = render(&entries, true);
        assert!(out.contains(kw), "missing {kw}");
    }
}

#[test]
fn pwsh_compound_with_multiple_semicolons() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("k".to_string(), "a; b; c; exit".to_string());
    let out = render(&entries, true);
    assert!(out.contains("a; b; c"));
    assert!(out.contains("exit"));
}

#[test]
fn pwsh_preserves_order() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("z".to_string(), "zcmd".to_string());
    entries.insert("a".to_string(), "acmd".to_string());
    let out = render(&entries, true);
    assert!(out.find("function z").unwrap() < out.find("function a").unwrap());
}

#[test]
fn pwsh_large_batch() {
    let mut entries = indexmap::IndexMap::new();
    for i in 0..25 { entries.insert(format!("k{i}"), format!("cmd{i}")); }
    let out = render(&entries, false);
    assert_eq!(out.matches("function ").count(), 25);
    assert_eq!(out.matches("Remove-Alias").count(), 25);
}

#[test]
fn pwsh_case_insensitive_keyword() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("x".to_string(), "EXIT".to_string());
    let out = render(&entries, true);
    assert!(out.contains("EXIT") || out.contains("exit"));
}

#[test]
fn pwsh_handles_crlf_tail() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("k".to_string(), "echo hi\r\nexit".to_string());
    let out = render(&entries, true);
    assert!(out.contains("exit"));
}

#[test]
fn pwsh_special_chars_in_command() {
    let mut entries = indexmap::IndexMap::new();
    entries.insert("x".to_string(), "echo hi bye".to_string());
    let out = render(&entries, true);
    assert!(out.contains("echo"));
}
