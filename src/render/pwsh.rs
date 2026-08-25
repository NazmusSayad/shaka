use crate::utils::env_expand::expand_pwsh_env_vars;
use indexmap::IndexMap;

const STATEMENT_KEYWORDS: [&str; 5] = ["exit", "return", "break", "continue", "throw"];

fn ends_with_statement_command(command: &str) -> bool {
    let first = statement_tail(command)
        .and_then(|segment| segment.split_whitespace().next())
        .unwrap_or("");
    STATEMENT_KEYWORDS
        .iter()
        .any(|keyword| keyword.eq_ignore_ascii_case(first))
}

fn statement_tail(command: &str) -> Option<&str> {
    command
        .split([';', '\n', '\r'])
        .rev()
        .find(|segment| !segment.trim().is_empty())
        .map(|segment| segment.trim())
}

fn bare_keyword_split(command: &str) -> Option<(String, String)> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }
    let segments: Vec<&str> = trimmed.split([';', '\n', '\r']).collect();
    let last_idx = segments.iter().rposition(|s| !s.trim().is_empty())?;
    let tail = segments[last_idx].trim().to_string();
    if !STATEMENT_KEYWORDS
        .iter()
        .any(|kw| kw.eq_ignore_ascii_case(&tail))
    {
        return None;
    }
    let prefix = segments[..last_idx]
        .iter()
        .filter_map(|s| {
            let t = s.trim();
            if t.is_empty() { None } else { Some(t) }
        })
        .collect::<Vec<_>>()
        .join("; ");
    Some((prefix, tail))
}

pub fn render(entries: &IndexMap<String, String>, conflict_mode: bool) -> String {
    let mut out = String::new();

    for (name, command) in entries {
        let expanded_command = expand_pwsh_env_vars(command);

        if !conflict_mode {
            out.push_str("Remove-Alias -Name ");
            out.push_str(name);
            out.push_str(" -Force -ErrorAction SilentlyContinue\n");
        }
        out.push_str("function ");
        out.push_str(name);
        out.push_str(" { ");
        if let Some((prefix, keyword)) = bare_keyword_split(&expanded_command) {
            if !prefix.is_empty() {
                out.push_str(&prefix);
                out.push_str("; ");
            }
            out.push_str(&format!(
                "if ($args.Count -gt 1) {{ Write-Warning \"{name}: {keyword} expects at most 1 argument; received $($args.Count); using first\" }}; if ($args.Count -gt 0) {{ {keyword} $args[0] }} else {{ {keyword} }}"
            ));
        } else if ends_with_statement_command(&expanded_command) {
            out.push_str(&expanded_command);
        } else {
            out.push_str(&expanded_command);
            out.push_str(" @args");
        }
        out.push_str(" }\n");
    }

    out
}

#[cfg(test)]
mod tests {
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
}
