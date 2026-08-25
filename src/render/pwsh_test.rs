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
