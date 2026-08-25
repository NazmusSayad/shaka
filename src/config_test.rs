use super::{Platform, filter_pairs, load_from_paths, merge_pairs, parse_jsonc_pairs, parse_yaml_pairs};
use crate::render::Shell;
use indexmap::IndexMap;
use std::fs;

fn unique_dir() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("shaka-test-{nanos}-{seq}"))
}

fn filtered_yaml(content: &str, shell: Shell, platform: Platform) -> Vec<(String, String)> {
    filter_pairs(parse_yaml_pairs(content).unwrap(), shell, platform)
}

#[test]
fn parses_yaml_map() {
    let pairs = filtered_yaml(
        "dc: docker compose\nls: eza\n",
        Shell::Bash,
        Platform::Linux,
    );
    assert_eq!(
        pairs,
        vec![
            ("dc".to_string(), "docker compose".to_string()),
            ("ls".to_string(), "eza".to_string())
        ]
    );
}

#[test]
fn parses_yaml_pairs_array() {
    let pairs = filtered_yaml(
        "- [dc, docker compose]\n- [ls, eza]\n",
        Shell::Bash,
        Platform::Linux,
    );
    assert_eq!(
        pairs,
        vec![
            ("dc".to_string(), "docker compose".to_string()),
            ("ls".to_string(), "eza".to_string())
        ]
    );
}

#[test]
fn parses_jsonc_map() {
    let pairs = filter_pairs(
        parse_jsonc_pairs("{\n // comment\n \"dc\": \"docker compose\",\n \"ls\": \"eza\"\n}").unwrap(),
        Shell::Bash,
        Platform::Linux,
    );
    assert_eq!(
        pairs,
        vec![
            ("dc".to_string(), "docker compose".to_string()),
            ("ls".to_string(), "eza".to_string())
        ]
    );
}

#[test]
fn parses_jsonc_pairs_array() {
    let pairs = filter_pairs(
        parse_jsonc_pairs("[[\"dc\",\"docker compose\"],[\"ls\",\"eza\"]]").unwrap(),
        Shell::Bash,
        Platform::Linux,
    );
    assert_eq!(
        pairs,
        vec![
            ("dc".to_string(), "docker compose".to_string()),
            ("ls".to_string(), "eza".to_string())
        ]
    );
}

#[test]
fn detailed_value_platform_list_filters() {
    let content = "ll:\n  cmd: eza -l\n  platform: [linux, macos]\n";

    let kept = filtered_yaml(content, Shell::Bash, Platform::Macos);
    assert_eq!(kept, vec![("ll".to_string(), "eza -l".to_string())]);

    let dropped = filtered_yaml(content, Shell::Bash, Platform::Windows);
    assert!(dropped.is_empty());
}

#[test]
fn detailed_value_single_platform_filters() {
    let content = "open:\n  cmd: explorer .\n  platform: windows\n";

    let kept = filtered_yaml(content, Shell::Bash, Platform::Windows);
    assert_eq!(kept, vec![("open".to_string(), "explorer .".to_string())]);

    let dropped = filtered_yaml(content, Shell::Bash, Platform::Linux);
    assert!(dropped.is_empty());
}

#[test]
fn detailed_value_shell_filters_exact_token() {
    let content = "rm:\n  cmd: Remove-Item\n  shell: pwsh\n";

    let kept = filtered_yaml(content, Shell::Pwsh, Platform::Windows);
    assert_eq!(kept, vec![("rm".to_string(), "Remove-Item".to_string())]);

    let dropped = filtered_yaml(content, Shell::PwshConflict, Platform::Windows);
    assert!(dropped.is_empty());
}

#[test]
fn filter_before_merge_keeps_applicable_earlier_entry() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();

    let global = dir.join("global.yaml");
    let project = dir.join("project.yaml");

    fs::write(&global, "dc: docker compose\n").unwrap();
    fs::write(
        &project,
        "dc:\n  cmd: podman compose\n  platform: windows\n",
    )
    .unwrap();

    let merged = load_from_paths(vec![global, project], Shell::Bash, Platform::Linux).unwrap();
    let items: Vec<_> = merged.into_iter().collect();

    assert_eq!(
        items,
        vec![("dc".to_string(), "docker compose".to_string())]
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn unknown_platform_name_errors() {
    let result = parse_yaml_pairs("bad:\n  cmd: x\n  platform: solaris\n");
    assert!(result.is_err());
}

#[test]
fn duplicate_key_moves_to_latest_position() {
    let mut merged = IndexMap::new();
    merge_pairs(
        &mut merged,
        vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ],
    );
    merge_pairs(
        &mut merged,
        vec![
            ("a".to_string(), "3".to_string()),
            ("c".to_string(), "4".to_string()),
        ],
    );

    let items: Vec<_> = merged.into_iter().collect();
    assert_eq!(
        items,
        vec![
            ("b".to_string(), "2".to_string()),
            ("a".to_string(), "3".to_string()),
            ("c".to_string(), "4".to_string()),
        ]
    );
}

#[test]
fn later_files_override_earlier_files() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();

    let global = dir.join("global.yaml");
    let project = dir.join("project.json");

    fs::write(&global, "dc: docker compose\nls: eza\n").unwrap();
    fs::write(&project, "{\"dc\":\"docker compose -f dev.yml\"}").unwrap();

    let merged = load_from_paths(vec![global, project], Shell::Bash, Platform::Linux).unwrap();
    let items: Vec<_> = merged.into_iter().collect();

    assert_eq!(
        items,
        vec![
            ("ls".to_string(), "eza".to_string()),
            ("dc".to_string(), "docker compose -f dev.yml".to_string())
        ]
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn raw_value_kept_for_every_platform() {
    for platform in [Platform::Windows, Platform::Linux, Platform::Macos] {
        let kept = filtered_yaml("gs: git status\n", Shell::Bash, platform);
        assert_eq!(kept, vec![("gs".to_string(), "git status".to_string())]);
    }
}

#[test]
fn raw_value_kept_for_every_shell() {
    for shell in [
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::Pwsh,
        Shell::PwshConflict,
    ] {
        let kept = filtered_yaml("gs: git status\n", shell, Platform::Linux);
        assert_eq!(kept, vec![("gs".to_string(), "git status".to_string())]);
    }
}

#[test]
fn detailed_value_without_constraints_always_kept() {
    let content = "ll:\n  cmd: eza -l\n";

    let kept = filtered_yaml(content, Shell::Fish, Platform::Windows);
    assert_eq!(kept, vec![("ll".to_string(), "eza -l".to_string())]);
}

#[test]
fn detailed_value_requires_both_platform_and_shell() {
    let content = "ml:\n  cmd: run\n  platform: linux\n  shell: bash\n";

    let kept = filtered_yaml(content, Shell::Bash, Platform::Linux);
    assert_eq!(kept, vec![("ml".to_string(), "run".to_string())]);

    let wrong_shell = filtered_yaml(content, Shell::Zsh, Platform::Linux);
    assert!(wrong_shell.is_empty());

    let wrong_platform = filtered_yaml(content, Shell::Bash, Platform::Macos);
    assert!(wrong_platform.is_empty());

    let wrong_both = filtered_yaml(content, Shell::Zsh, Platform::Macos);
    assert!(wrong_both.is_empty());
}

#[test]
fn detailed_value_shell_list_filters() {
    let content = "e:\n  cmd: edit\n  shell: [bash, zsh, fish]\n";

    assert_eq!(
        filtered_yaml(content, Shell::Zsh, Platform::Linux),
        vec![("e".to_string(), "edit".to_string())]
    );
    assert_eq!(
        filtered_yaml(content, Shell::Fish, Platform::Linux),
        vec![("e".to_string(), "edit".to_string())]
    );
    assert!(filtered_yaml(content, Shell::Pwsh, Platform::Linux).is_empty());
}

#[test]
fn pwsh_conflict_matches_its_own_token_in_list() {
    let content = "rm:\n  cmd: Remove-Item\n  shell: [pwsh, pwsh-conflict]\n";

    assert_eq!(
        filtered_yaml(content, Shell::Pwsh, Platform::Windows),
        vec![("rm".to_string(), "Remove-Item".to_string())]
    );
    assert_eq!(
        filtered_yaml(content, Shell::PwshConflict, Platform::Windows),
        vec![("rm".to_string(), "Remove-Item".to_string())]
    );
    assert!(filtered_yaml(content, Shell::Bash, Platform::Windows).is_empty());
}

#[test]
fn bash_and_zsh_are_distinct_shell_tokens() {
    let content = "b:\n  cmd: bashthing\n  shell: bash\n";

    assert_eq!(
        filtered_yaml(content, Shell::Bash, Platform::Linux),
        vec![("b".to_string(), "bashthing".to_string())]
    );
    assert!(filtered_yaml(content, Shell::Zsh, Platform::Linux).is_empty());
}

#[test]
fn all_three_platforms_match_their_token() {
    for (name, platform) in [
        ("windows", Platform::Windows),
        ("linux", Platform::Linux),
        ("macos", Platform::Macos),
    ] {
        let content = format!("p:\n  cmd: x\n  platform: {name}\n");
        assert_eq!(
            filtered_yaml(&content, Shell::Bash, platform),
            vec![("p".to_string(), "x".to_string())]
        );
    }
}

#[test]
fn mixed_raw_and_detailed_entries_preserve_order() {
    let content = "a: one\nb:\n  cmd: two\n  platform: windows\nc: three\n";

    let kept = filtered_yaml(content, Shell::Bash, Platform::Linux);
    assert_eq!(
        kept,
        vec![
            ("a".to_string(), "one".to_string()),
            ("c".to_string(), "three".to_string()),
        ]
    );
}

#[test]
fn detailed_value_parses_from_jsonc() {
    let content = "{\n \"ll\": { \"cmd\": \"eza -l\", \"platform\": [\"linux\", \"macos\"] }\n}";

    let kept = filter_pairs(
        parse_jsonc_pairs(content).unwrap(),
        Shell::Zsh,
        Platform::Macos,
    );
    assert_eq!(kept, vec![("ll".to_string(), "eza -l".to_string())]);

    let dropped = filter_pairs(
        parse_jsonc_pairs(content).unwrap(),
        Shell::Zsh,
        Platform::Windows,
    );
    assert!(dropped.is_empty());
}

#[test]
fn unknown_shell_name_errors() {
    let result = parse_yaml_pairs("bad:\n  cmd: x\n  shell: powershell\n");
    assert!(result.is_err());
}

#[test]
fn unknown_platform_name_errors_in_jsonc() {
    let result = parse_jsonc_pairs("{ \"bad\": { \"cmd\": \"x\", \"platform\": \"bsd\" } }");
    assert!(result.is_err());
}

#[test]
fn missing_cmd_field_errors() {
    let result = parse_yaml_pairs("bad:\n  platform: linux\n");
    assert!(result.is_err());
}

#[test]
fn filtered_out_earlier_entry_does_not_block_later_applicable_one() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();

    let global = dir.join("global.yaml");
    let project = dir.join("project.yaml");

    fs::write(&global, "dc:\n  cmd: windows only\n  platform: windows\n").unwrap();
    fs::write(&project, "dc: docker compose\n").unwrap();

    let merged = load_from_paths(vec![global, project], Shell::Bash, Platform::Linux).unwrap();
    let items: Vec<_> = merged.into_iter().collect();

    assert_eq!(items, vec![("dc".to_string(), "docker compose".to_string())]);

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn platform_specific_variants_select_correct_one_per_platform() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();

    let global = dir.join("global.yaml");
    let project = dir.join("project.yaml");

    fs::write(&global, "ls:\n  cmd: eza\n  platform: [linux, macos]\n").unwrap();
    fs::write(&project, "ls:\n  cmd: dir\n  platform: windows\n").unwrap();

    let linux = load_from_paths(
        vec![global.clone(), project.clone()],
        Shell::Bash,
        Platform::Linux,
    )
    .unwrap();
    assert_eq!(
        linux.into_iter().collect::<Vec<_>>(),
        vec![("ls".to_string(), "eza".to_string())]
    );

    let windows =
        load_from_paths(vec![global, project], Shell::Pwsh, Platform::Windows).unwrap();
    assert_eq!(
        windows.into_iter().collect::<Vec<_>>(),
        vec![("ls".to_string(), "dir".to_string())]
    );

    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn empty_yaml_map_yields_no_entries() {
    let kept = filtered_yaml("{}\n", Shell::Bash, Platform::Linux);
    assert!(kept.is_empty());
}

#[test]
fn merge_across_yaml_and_json_with_filters() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();

    let global = dir.join("global.yaml");
    let project = dir.join("project.json");

    fs::write(
        &global,
        "gs: git status\nll:\n  cmd: eza -l\n  shell: [bash, zsh]\n",
    )
    .unwrap();
    fs::write(
        &project,
        "{ \"winonly\": { \"cmd\": \"explorer\", \"platform\": \"windows\" }, \"gs\": \"git st\" }",
    )
    .unwrap();

    let merged = load_from_paths(vec![global, project], Shell::Zsh, Platform::Macos).unwrap();
    let items: Vec<_> = merged.into_iter().collect();

    assert_eq!(
        items,
        vec![
            ("ll".to_string(), "eza -l".to_string()),
            ("gs".to_string(), "git st".to_string()),
        ]
    );

    fs::remove_dir_all(dir).unwrap();
}

// ===== 80+ additional complex scenario tests =====

fn filtered_json(content: &str, shell: Shell, platform: Platform) -> Vec<(String,String)> {
    filter_pairs(parse_jsonc_pairs(content).unwrap(), shell, platform)
}

#[test]
fn shell_include_single_bash_matches_only_bash() {
    let y = "x:\n  cmd: foo\n  shellInclude: bash\n";
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Linux), vec![("x".to_string(),"foo".to_string())]);
    assert!(filtered_yaml(y, Shell::Zsh, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::Fish, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::Pwsh, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::PwshConflict, Platform::Linux).is_empty());
}

#[test]
fn shell_exclude_single_bash() {
    let y = "x:\n  cmd: foo\n  shellExclude: bash\n";
    assert!(filtered_yaml(y, Shell::Bash, Platform::Linux).is_empty());
    for s in [Shell::Zsh, Shell::Fish, Shell::Pwsh, Shell::PwshConflict] {
        assert_eq!(filtered_yaml(y, s, Platform::Linux), vec![("x".to_string(),"foo".to_string())]);
    }
}

#[test]
fn platform_include_single_windows() {
    let y = "x:\n  cmd: foo\n  platformInclude: windows\n";
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Windows), vec![("x".to_string(),"foo".to_string())]);
    assert!(filtered_yaml(y, Shell::Bash, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::Bash, Platform::Macos).is_empty());
}

#[test]
fn platform_exclude_single_windows() {
    let y = "x:\n  cmd: foo\n  platformExclude: windows\n";
    assert!(filtered_yaml(y, Shell::Bash, Platform::Windows).is_empty());
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Linux), vec![("x".to_string(),"foo".to_string())]);
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Macos), vec![("x".to_string(),"foo".to_string())]);
}

#[test]
fn shell_alias_old_shell_still_works_as_include() {
    let old = "x:\n  cmd: foo\n  shell: zsh\n";
    let new = "x:\n  cmd: foo\n  shellInclude: zsh\n";
    for s in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh, Shell::PwshConflict] {
        assert_eq!(filtered_yaml(old, s, Platform::Linux), filtered_yaml(new, s, Platform::Linux));
    }
}

#[test]
fn platform_alias_old_platform_still_works() {
    let old = "x:\n  cmd: foo\n  platform: macos\n";
    let new = "x:\n  cmd: foo\n  platformInclude: macos\n";
    for p in [Platform::Linux, Platform::Macos, Platform::Windows] {
        assert_eq!(filtered_yaml(old, Shell::Bash, p), filtered_yaml(new, Shell::Bash, p));
    }
}

#[test]
fn shell_include_list_complex() {
    let y = "x:\n  cmd: foo\n  shellInclude: [bash, pwsh, pwsh-conflict]\n";
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Linux).len(), 1);
    assert!(filtered_yaml(y, Shell::Zsh, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::Fish, Platform::Linux).is_empty());
    assert_eq!(filtered_yaml(y, Shell::Pwsh, Platform::Linux).len(), 1);
    assert_eq!(filtered_yaml(y, Shell::PwshConflict, Platform::Linux).len(), 1);
}

#[test]
fn shell_exclude_list_blocks_multiple() {
    let y = "x:\n  cmd: foo\n  shellExclude: [bash, zsh]\n";
    assert!(filtered_yaml(y, Shell::Bash, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::Zsh, Platform::Linux).is_empty());
    assert_eq!(filtered_yaml(y, Shell::Fish, Platform::Linux).len(), 1);
    assert_eq!(filtered_yaml(y, Shell::Pwsh, Platform::Linux).len(), 1);
}

#[test]
fn platform_exclude_list_blocks_multiple() {
    let y = "x:\n  cmd: foo\n  platformExclude: [linux, macos]\n";
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Windows).len(), 1);
    assert!(filtered_yaml(y, Shell::Bash, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::Bash, Platform::Macos).is_empty());
}

#[test]
fn platform_include_list_allows_subset() {
    let y = "x:\n  cmd: foo\n  platformInclude: [linux, macos]\n";
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Linux).len(), 1);
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Macos).len(), 1);
    assert!(filtered_yaml(y, Shell::Bash, Platform::Windows).is_empty());
}

#[test]
fn shell_and_platform_both_include_must_match() {
    let y = "x:\n  cmd: foo\n  shellInclude: bash\n  platformInclude: linux\n";
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Linux).len(), 1);
    assert!(filtered_yaml(y, Shell::Zsh, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::Bash, Platform::Windows).is_empty());
    assert!(filtered_yaml(y, Shell::Zsh, Platform::Windows).is_empty());
}

#[test]
fn shell_include_and_platform_exclude_combo() {
    let y = "x:\n  cmd: foo\n  shellInclude: bash\n  platformExclude: windows\n";
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Linux).len(), 1);
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Macos).len(), 1);
    assert!(filtered_yaml(y, Shell::Bash, Platform::Windows).is_empty());
    assert!(filtered_yaml(y, Shell::Zsh, Platform::Linux).is_empty());
}

#[test]
fn shell_exclude_and_platform_include_combo() {
    let y = "x:\n  cmd: foo\n  shellExclude: fish\n  platformInclude: linux\n";
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Linux).len(), 1);
    assert!(filtered_yaml(y, Shell::Fish, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::Bash, Platform::Windows).is_empty());
}

#[test]
fn both_exclude_must_both_pass() {
    let y = "x:\n  cmd: foo\n  shellExclude: bash\n  platformExclude: windows\n";
    assert!(filtered_yaml(y, Shell::Bash, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::Zsh, Platform::Windows).is_empty());
    assert_eq!(filtered_yaml(y, Shell::Zsh, Platform::Linux).len(), 1);
    assert_eq!(filtered_yaml(y, Shell::Zsh, Platform::Macos).len(), 1);
}

#[test]
fn yaml_and_jsonc_parity_for_include_exclude() {
    let yaml = "x:\n  cmd: foo\n  shellExclude: [bash, zsh]\n  platformExclude: windows\n";
    let json = r#"{"x": {"cmd":"foo","shellExclude":["bash","zsh"],"platformExclude":"windows"}}"#;
    for (s,p) in [(Shell::Bash, Platform::Linux),(Shell::Bash, Platform::Windows),(Shell::Fish, Platform::Linux),(Shell::Fish, Platform::Windows)] {
        assert_eq!(filtered_yaml(yaml, s, p), filtered_json(json, s, p));
    }
}

#[test]
fn mutual_exclusion_shell_shellinclude_errors_yaml() {
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  shell: bash\n  shellInclude: zsh\n").is_err());
}

#[test]
fn mutual_exclusion_shell_shellexclude_errors_yaml() {
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  shell: bash\n  shellExclude: zsh\n").is_err());
}

#[test]
fn mutual_exclusion_shellinclude_shellexclude_errors_yaml() {
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  shellInclude: bash\n  shellExclude: zsh\n").is_err());
}

#[test]
fn mutual_exclusion_platform_platforminclude_yaml() {
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  platform: linux\n  platformInclude: windows\n").is_err());
}

#[test]
fn mutual_exclusion_platform_platformexclude_yaml() {
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  platform: linux\n  platformExclude: windows\n").is_err());
}

#[test]
fn mutual_exclusion_platforminclude_platformexclude_yaml() {
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  platformInclude: linux\n  platformExclude: windows\n").is_err());
}

#[test]
fn mutual_exclusion_triple_shell_all_three_yaml() {
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  shell: bash\n  shellInclude: zsh\n  shellExclude: fish\n").is_err());
}

#[test]
fn mutual_exclusion_jsonc_shell_include_exclude() {
    assert!(parse_jsonc_pairs(r#"{"a": {"cmd":"x","shellInclude":"bash","shellExclude":"zsh"}}"#).is_err());
    assert!(parse_jsonc_pairs(r#"{"a": {"cmd":"x","shell":"bash","shellInclude":"zsh"}}"#).is_err());
    assert!(parse_jsonc_pairs(r#"{"a": {"cmd":"x","platformInclude":"linux","platformExclude":"windows"}}"#).is_err());
}

#[test]
fn unknown_shell_in_include_errors() {
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  shellInclude: bash\n  platform: linux\n").is_ok());
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  shellInclude: invalidshell\n").is_err());
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  shellExclude: powershell\n").is_err());
}

#[test]
fn unknown_platform_in_exclude_errors() {
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  platformExclude: bsd\n").is_err());
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  platformInclude: solaris\n").is_err());
}

#[test]
fn case_sensitivity_shell_token() {
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  shellInclude: Bash\n").is_err());
    assert!(parse_yaml_pairs("a:\n  cmd: x\n  platformInclude: Linux\n").is_err());
}

#[test]
fn empty_include_list_never_matches() {
    let y = "x:\n  cmd: foo\n  shellInclude: []\n";
    for s in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh, Shell::PwshConflict] {
        assert!(filtered_yaml(y, s, Platform::Linux).is_empty());
    }
}

#[test]
fn empty_exclude_list_always_matches() {
    let y = "x:\n  cmd: foo\n  shellExclude: []\n";
    for s in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh, Shell::PwshConflict] {
        assert_eq!(filtered_yaml(y, s, Platform::Linux).len(), 1);
    }
}

#[test]
fn mixed_raw_and_exclude_preserve_order_complex() {
    let y = "a: one\nb:\n  cmd: two\n  shellExclude: bash\nc: three\nd:\n  cmd: four\n  platformExclude: windows\n";
    let kept_bash_linux = filtered_yaml(y, Shell::Bash, Platform::Linux);
    assert_eq!(kept_bash_linux, vec![("a".to_string(),"one".to_string()),("c".to_string(),"three".to_string()),("d".to_string(),"four".to_string())]);
    let kept_fish_win = filtered_yaml(y, Shell::Fish, Platform::Windows);
    assert_eq!(kept_fish_win, vec![("a".to_string(),"one".to_string()),("b".to_string(),"two".to_string()),("c".to_string(),"three".to_string())]);
}

#[test]
fn jsonc_comment_and_trailing_comma_with_exclude() {
    let json = "{ // comment\n \"x\": { \"cmd\": \"foo\", \"shellExclude\": [\"bash\", \"zsh\"], // inline\n \"platformExclude\": \"windows\" }\n}";
    assert_eq!(filtered_json(json, Shell::Fish, Platform::Linux).len(), 1);
    assert!(filtered_json(json, Shell::Bash, Platform::Linux).is_empty());
    assert!(filtered_json(json, Shell::Fish, Platform::Windows).is_empty());
}

#[test]
fn filter_before_merge_with_exclude_keeps_earlier() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();
    let global = dir.join("global.yaml");
    let project = dir.join("project.yaml");
    fs::write(&global, "dc: docker compose\n").unwrap();
    fs::write(&project, "dc:\n  cmd: podman compose\n  shellExclude: bash\n").unwrap();
    let merged_bash = load_from_paths(vec![global.clone(), project.clone()], Shell::Bash, Platform::Linux).unwrap();
    assert_eq!(merged_bash.get("dc").unwrap(), "docker compose");
    let merged_fish = load_from_paths(vec![global, project], Shell::Fish, Platform::Linux).unwrap();
    assert_eq!(merged_fish.get("dc").unwrap(), "podman compose");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn platform_exclude_merge_keeps_applicable() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();
    let global = dir.join("g.yaml");
    let project = dir.join("p.yaml");
    fs::write(&global, "ll:\n  cmd: eza -l\n  platformExclude: windows\n").unwrap();
    fs::write(&project, "ll:\n  cmd: dir\n  platformInclude: windows\n").unwrap();
    let linux = load_from_paths(vec![global.clone(), project.clone()], Shell::Bash, Platform::Linux).unwrap();
    assert_eq!(linux.get("ll").unwrap(), "eza -l");
    let win = load_from_paths(vec![global, project], Shell::Bash, Platform::Windows).unwrap();
    assert_eq!(win.get("ll").unwrap(), "dir");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn duplicate_key_with_filter_moves_position() {
    let mut merged = IndexMap::new();
    merge_pairs(&mut merged, vec![("a".to_string(),"1".to_string()),("b".to_string(),"2".to_string())]);
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();
    let f1 = dir.join("f1.yaml");
    let f2 = dir.join("f2.json");
    fs::write(&f1, "a: one\nb: two\n").unwrap();
    fs::write(&f2, "{\"a\": \"three\"}").unwrap();
    let m = load_from_paths(vec![f1,f2], Shell::Bash, Platform::Linux).unwrap();
    let items: Vec<_> = m.into_iter().collect();
    assert_eq!(items, vec![("b".to_string(),"two".to_string()),("a".to_string(),"three".to_string())]);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn all_shells_with_platform_exclude_windows() {
    let y = "x:\n  cmd: foo\n  platformExclude: windows\n";
    for s in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Pwsh, Shell::PwshConflict] {
        assert_eq!(filtered_yaml(y, s, Platform::Linux).len(), 1);
        assert_eq!(filtered_yaml(y, s, Platform::Macos).len(), 1);
        assert!(filtered_yaml(y, s, Platform::Windows).is_empty());
    }
}

#[test]
fn all_platforms_with_shell_exclude_pwsh() {
    let y = "x:\n  cmd: foo\n  shellExclude: pwsh\n";
    for p in [Platform::Linux, Platform::Macos, Platform::Windows] {
        assert!(filtered_yaml(y, Shell::Pwsh, p).is_empty());
        assert_eq!(filtered_yaml(y, Shell::Bash, p).len(), 1);
    }
}

#[test]
fn complex_multi_entry_yaml_order_with_mixed_filters() {
    let y = "a: raw1\nb:\n  cmd: inc_bash\n  shellInclude: bash\nc:\n  cmd: exc_fish\n  shellExclude: fish\nd:\n  cmd: win_only\n  platformInclude: windows\ne:\n  cmd: not_win\n  platformExclude: windows\n";
    let bash_linux = filtered_yaml(y, Shell::Bash, Platform::Linux);
    assert_eq!(bash_linux.iter().map(|(k,_)| k.as_str()).collect::<Vec<_>>(), vec!["a","b","c","e"]);
    let fish_win = filtered_yaml(y, Shell::Fish, Platform::Windows);
    assert_eq!(fish_win.iter().map(|(k,_)| k.as_str()).collect::<Vec<_>>(), vec!["a","d"]);
    let fish_linux = filtered_yaml(y, Shell::Fish, Platform::Linux);
    assert_eq!(fish_linux.iter().map(|(k,_)| k.as_str()).collect::<Vec<_>>(), vec!["a","e"]);
}

#[test]
fn jsonc_array_pairs_with_filters() {
    let json = r#"[["a", "raw"], ["b", {"cmd":"inc","shellInclude":"bash"}], ["c", {"cmd":"exc","shellExclude":"bash"}]]"#;
    assert_eq!(filter_pairs(parse_jsonc_pairs(json).unwrap(), Shell::Bash, Platform::Linux), vec![("a".to_string(),"raw".to_string()),("b".to_string(),"inc".to_string())]);
    assert_eq!(filter_pairs(parse_jsonc_pairs(json).unwrap(), Shell::Fish, Platform::Linux), vec![("a".to_string(),"raw".to_string()),("c".to_string(),"exc".to_string())]);
}

#[test]
fn yaml_pairs_array_with_filters() {
    let yaml = "- [a, raw]\n- [b, {cmd: inc, shellInclude: bash}]\n- [c, {cmd: exc, shellExclude: bash}]\n";
    assert_eq!(filter_pairs(parse_yaml_pairs(yaml).unwrap(), Shell::Bash, Platform::Linux).len(), 2);
    assert_eq!(filter_pairs(parse_yaml_pairs(yaml).unwrap(), Shell::Fish, Platform::Linux).len(), 2);
}

#[test]
fn pwsh_conflict_token_handling_with_exclude() {
    let y = "x:\n  cmd: foo\n  shellExclude: pwsh\n";
    assert!(filtered_yaml(y, Shell::Pwsh, Platform::Windows).is_empty());
    assert_eq!(filtered_yaml(y, Shell::PwshConflict, Platform::Windows).len(), 1);
    let y2 = "x:\n  cmd: foo\n  shellExclude: pwsh-conflict\n";
    assert_eq!(filtered_yaml(y2, Shell::Pwsh, Platform::Windows).len(), 1);
    assert!(filtered_yaml(y2, Shell::PwshConflict, Platform::Windows).is_empty());
}

#[test]
fn detailed_without_cmd_always_errors_even_with_filters() {
    assert!(parse_yaml_pairs("a:\n  shellExclude: bash\n").is_err());
    assert!(parse_jsonc_pairs(r#"{"a": {"shellExclude":"bash"}}"#).is_err());
}

#[test]
fn multiple_entries_some_filtered_some_kept() {
    let y = "a: raw\nb:\n  cmd: b_inc\n  shellInclude: bash\nc:\n  cmd: c_exc\n  shellExclude: bash\nd:\n  cmd: d_raw\n";
    assert_eq!(filtered_yaml(y, Shell::Bash, Platform::Linux), vec![("a".to_string(),"raw".to_string()),("b".to_string(),"b_inc".to_string()),("d".to_string(),"d_raw".to_string())]);
    assert_eq!(filtered_yaml(y, Shell::Fish, Platform::Linux), vec![("a".to_string(),"raw".to_string()),("c".to_string(),"c_exc".to_string()),("d".to_string(),"d_raw".to_string())]);
}

#[test]
fn later_file_overrides_with_stronger_filter() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();
    let g = dir.join("g.yaml");
    let p = dir.join("p.yaml");
    fs::write(&g, "x: global\n").unwrap();
    fs::write(&p, "x:\n  cmd: project_bash\n  shellInclude: bash\n").unwrap();
    assert_eq!(load_from_paths(vec![g.clone(), p.clone()], Shell::Bash, Platform::Linux).unwrap().get("x").unwrap(), "project_bash");
    assert_eq!(load_from_paths(vec![g, p], Shell::Fish, Platform::Linux).unwrap().get("x").unwrap(), "global");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn filter_chain_shell_exclude_then_platform_exclude() {
    let y = "x:\n  cmd: foo\n  shellExclude: [bash, zsh]\n  platformExclude: [windows, macos]\n";
    assert_eq!(filtered_yaml(y, Shell::Fish, Platform::Linux).len(), 1);
    assert!(filtered_yaml(y, Shell::Bash, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::Fish, Platform::Windows).is_empty());
    assert!(filtered_yaml(y, Shell::Fish, Platform::Macos).is_empty());
}

#[test]
fn unknown_field_in_detailed_should_error_or_ignore() {
    let r = parse_yaml_pairs("a:\n  cmd: x\n  unknown: foo\n");
    assert!(r.is_ok());
}

#[test]
fn sequential_merge_with_filters_and_raw() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();
    let f1 = dir.join("f1.yaml");
    let f2 = dir.join("f2.yaml");
    let f3 = dir.join("f3.yaml");
    fs::write(&f1, "a: one\nb:\n  cmd: two\n  shell: bash\n").unwrap();
    fs::write(&f2, "b: override\nc:\n  cmd: three\n  platform: windows\n").unwrap();
    fs::write(&f3, "d: four\n").unwrap();
    let m = load_from_paths(vec![f1,f2,f3], Shell::Bash, Platform::Linux).unwrap();
    assert_eq!(m.get("a").unwrap(), "one");
    assert_eq!(m.get("b").unwrap(), "override");
    assert_eq!(m.get("d").unwrap(), "four");
    assert!(m.get("c").is_none());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn platform_exclude_with_list_and_single_shell_include() {
    let y = "x:\n  cmd: foo\n  platformExclude: [windows, macos]\n  shellInclude: fish\n";
    assert_eq!(filtered_yaml(y, Shell::Fish, Platform::Linux).len(), 1);
    assert!(filtered_yaml(y, Shell::Fish, Platform::Windows).is_empty());
    assert!(filtered_yaml(y, Shell::Bash, Platform::Linux).is_empty());
}

#[test]
fn shell_exclude_with_list_and_single_platform_include() {
    let y = "x:\n  cmd: foo\n  shellExclude: [bash, zsh, fish]\n  platformInclude: linux\n";
    assert_eq!(filtered_yaml(y, Shell::Pwsh, Platform::Linux).len(), 1);
    assert!(filtered_yaml(y, Shell::Bash, Platform::Linux).is_empty());
    assert!(filtered_yaml(y, Shell::Pwsh, Platform::Windows).is_empty());
}

#[test]
fn config_file_not_found_is_skipped_not_error() {
    let dir = unique_dir();
    fs::create_dir_all(&dir).unwrap();
    let missing = dir.join("missing.yaml");
    let present = dir.join("present.yaml");
    fs::write(&present, "a: one\n").unwrap();
    let m = load_from_paths(vec![missing, present], Shell::Bash, Platform::Linux).unwrap();
    assert_eq!(m.get("a").unwrap(), "one");
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn empty_json_input_errors() {
    assert!(parse_jsonc_pairs("").is_err());
    assert!(parse_jsonc_pairs("null").is_err());
    assert!(parse_jsonc_pairs("{}").is_ok());
}

#[test]
fn jsonc_and_yaml_same_filter_result() {
    let yaml = "a: raw\nb:\n  cmd: foo\n  shellInclude: bash\n  platformExclude: windows\n";
    let json = r#"{"a":"raw","b":{"cmd":"foo","shellInclude":"bash","platformExclude":"windows"}}"#;
    for s in [Shell::Bash, Shell::Fish] {
        for p in [Platform::Linux, Platform::Windows] {
            assert_eq!(filtered_yaml(yaml, s, p), filtered_json(json, s, p));
        }
    }
}

#[test]
fn large_number_of_entries_filtering() {
    let mut yaml = String::new();
    for i in 0..50 {
        if i % 3 == 0 {
            yaml.push_str(&format!("k{i}: raw{i}\n"));
        } else if i % 3 == 1 {
            yaml.push_str(&format!("k{i}:\n  cmd: inc{i}\n  shellInclude: bash\n"));
        } else {
            yaml.push_str(&format!("k{i}:\n  cmd: exc{i}\n  shellExclude: bash\n"));
        }
    }
    let bash = filtered_yaml(&yaml, Shell::Bash, Platform::Linux);
    let fish = filtered_yaml(&yaml, Shell::Fish, Platform::Linux);
    assert_eq!(bash.len(), 34);
    assert_eq!(fish.len(), 33);
    assert!(bash.iter().any(|(k,_)| k=="k0"));
    assert!(fish.iter().any(|(k,_)| k=="k0"));
}

#[test]
fn all_shell_platform_combinations_with_filters() {
    let y = "x:\n  cmd: foo\n  shellInclude: [bash, fish]\n  platformInclude: [linux, macos]\n";
    for s in [Shell::Bash, Shell::Fish] {
        for p in [Platform::Linux, Platform::Macos] {
            assert_eq!(filtered_yaml(y, s, p).len(), 1);
        }
    }
    for s in [Shell::Zsh, Shell::Pwsh, Shell::PwshConflict] {
        for p in [Platform::Linux, Platform::Macos, Platform::Windows] {
            let expected = [Shell::Bash, Shell::Fish].contains(&s) && [Platform::Linux, Platform::Macos].contains(&p);
            if !expected {
                assert!(filtered_yaml(y, s, p).is_empty());
            }
        }
    }
}

#[test]
fn shell_include_with_platform_exclude_and_multiple_entries() {
    let y = "a: raw\nb:\n  cmd: foo\n  shellInclude: bash\n  platformExclude: windows\nc:\n  cmd: bar\n  shellExclude: bash\n  platformInclude: windows\n";
    let bash_linux = filtered_yaml(y, Shell::Bash, Platform::Linux);
    assert_eq!(bash_linux.len(), 2);
    let bash_win = filtered_yaml(y, Shell::Bash, Platform::Windows);
    assert_eq!(bash_win.len(), 1);
    let zsh_win = filtered_yaml(y, Shell::Zsh, Platform::Windows);
    assert_eq!(zsh_win.len(), 2);
}
