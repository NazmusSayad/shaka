mod fish;
mod pwsh;
mod sh;
use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Clone, Copy, PartialEq, Deserialize)]
pub enum Shell {
    #[serde(rename = "bash")]
    Bash,
    #[serde(rename = "fish")]
    Fish,
    #[serde(rename = "pwsh")]
    Pwsh,
    #[serde(rename = "pwsh-conflict")]
    PwshConflict,
    #[serde(rename = "zsh")]
    Zsh,
}

pub fn render(shell: Shell, entries: &IndexMap<String, String>) -> String {
    match shell {
        Shell::Bash | Shell::Zsh => sh::render(entries),
        Shell::Fish => fish::render(entries),
        Shell::Pwsh => pwsh::render(entries, false),
        Shell::PwshConflict => pwsh::render(entries, true),
    }
}

#[cfg(test)]
mod tests {
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
}
