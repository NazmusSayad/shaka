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
#[path = "mod_test.rs"]
mod tests;
