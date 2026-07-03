use crate::render::Shell;
use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(untagged)]
enum ConfigForm {
    Map(BTreeMap<String, ConfigValue>),
    Pairs(Vec<(String, ConfigValue)>),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ConfigValue {
    Raw(String),
    Detailed(DetailedValue),
}

#[derive(Deserialize)]
struct DetailedValue {
    cmd: String,
    #[serde(default)]
    shell: Option<OneOrMany<Shell>>,
    #[serde(default)]
    platform: Option<OneOrMany<Platform>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T: PartialEq> OneOrMany<T> {
    fn contains(&self, target: &T) -> bool {
        match self {
            OneOrMany::One(value) => value == target,
            OneOrMany::Many(values) => values.contains(target),
        }
    }
}

#[derive(Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Platform {
    Windows,
    Linux,
    Macos,
}

pub fn load_merged_config(shell: Shell) -> Result<IndexMap<String, String>, String> {
    let platform = current_platform()?;
    load_from_paths(config_paths(), shell, platform)
}

fn current_platform() -> Result<Platform, String> {
    match std::env::consts::OS {
        "windows" => Ok(Platform::Windows),
        "linux" => Ok(Platform::Linux),
        "macos" => Ok(Platform::Macos),
        other => Err(format!("unrecognized platform: {other}")),
    }
}

fn load_from_paths(
    paths: Vec<PathBuf>,
    shell: Shell,
    platform: Platform,
) -> Result<IndexMap<String, String>, String> {
    let mut merged = IndexMap::new();

    for path in paths {
        if !path.exists() {
            continue;
        }

        let pairs = parse_config_file(&path, shell, platform)?;
        merge_pairs(&mut merged, pairs);
    }

    Ok(merged)
}

fn parse_config_file(
    path: &Path,
    shell: Shell,
    platform: Platform,
) -> Result<Vec<(String, String)>, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed reading {}: {err}", path.display()))?;

    let pairs = if path.extension().is_some_and(|ext| ext == "yaml") {
        parse_yaml_pairs(&content)
            .map_err(|err| format!("failed parsing YAML {}: {err}", path.display()))?
    } else {
        parse_jsonc_pairs(&content)
            .map_err(|err| format!("failed parsing JSONC {}: {err}", path.display()))?
    };

    Ok(filter_pairs(pairs, shell, platform))
}

fn parse_yaml_pairs(content: &str) -> Result<Vec<(String, ConfigValue)>, serde_yaml_ng::Error> {
    let parsed: ConfigForm = serde_yaml_ng::from_str(content)?;
    Ok(normalize(parsed))
}

fn parse_jsonc_pairs(content: &str) -> Result<Vec<(String, ConfigValue)>, String> {
    let value = jsonc_parser::parse_to_serde_value(content, &Default::default())
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "empty JSON input".to_string())?;

    let parsed: ConfigForm = serde_json::from_value(value).map_err(|err| err.to_string())?;
    Ok(normalize(parsed))
}

fn normalize(parsed: ConfigForm) -> Vec<(String, ConfigValue)> {
    match parsed {
        ConfigForm::Map(map) => map.into_iter().collect(),
        ConfigForm::Pairs(pairs) => pairs,
    }
}

fn filter_pairs(
    pairs: Vec<(String, ConfigValue)>,
    shell: Shell,
    platform: Platform,
) -> Vec<(String, String)> {
    let mut kept = Vec::with_capacity(pairs.len());

    for (key, value) in pairs {
        match value {
            ConfigValue::Raw(cmd) => kept.push((key, cmd)),
            ConfigValue::Detailed(detail) => {
                let platform_ok = match &detail.platform {
                    None => true,
                    Some(allowed) => allowed.contains(&platform),
                };
                let shell_ok = match &detail.shell {
                    None => true,
                    Some(allowed) => allowed.contains(&shell),
                };

                if platform_ok && shell_ok {
                    kept.push((key, detail.cmd));
                }
            }
        }
    }

    kept
}

fn merge_pairs(merged: &mut IndexMap<String, String>, pairs: Vec<(String, String)>) {
    for (key, value) in pairs {
        if merged.contains_key(&key) {
            merged.shift_remove(&key);
        }
        merged.insert(key, value);
    }
}

fn config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(6);

    if let Some(home) = home_dir() {
        paths.push(home.join(".config").join("shaka.yaml"));
        paths.push(home.join(".config").join("shaka.json"));
        paths.push(home.join(".shaka.yaml"));
        paths.push(home.join(".shaka.json"));
    }

    if let Ok(current_dir) = std::env::current_dir() {
        paths.push(current_dir.join(".shaka.yaml"));
        paths.push(current_dir.join(".shaka.json"));
    }

    paths
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}

#[cfg(test)]
mod tests {
    use super::{
        Platform, filter_pairs, load_from_paths, merge_pairs, parse_jsonc_pairs, parse_yaml_pairs,
    };
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
            parse_jsonc_pairs("{\n // comment\n \"dc\": \"docker compose\",\n \"ls\": \"eza\"\n}")
                .unwrap(),
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
}
