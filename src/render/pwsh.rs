use indexmap::IndexMap;
use crate::utils::env_expand::expand_pwsh_env_vars;

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
        out.push_str(&expanded_command);
        out.push_str(" @args }\n");
    }

    out
}

#[cfg(test)]
mod tests {
    use super::render;
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
}
