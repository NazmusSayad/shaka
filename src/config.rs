use crate::render::Shell;
use indexmap::IndexMap;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(untagged)]
enum Config {
    Map(IndexMap<String, Value>),
    Pairs(Vec<(String, Value)>),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Value {
    Command(String),
    Conditional(Box<Conditional>),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Conditional {
    cmd: String,
    #[serde(rename = "cmd.bash")]
    cmd_bash: Option<String>,
    #[serde(rename = "cmd.fish")]
    cmd_fish: Option<String>,
    #[serde(rename = "cmd.pwsh")]
    cmd_pwsh: Option<String>,
    #[serde(rename = "cmd.pwsh-conflict")]
    cmd_pwsh_conflict: Option<String>,
    #[serde(rename = "cmd.zsh")]
    cmd_zsh: Option<String>,

    #[serde(alias = "shellInclude")]
    shell: Option<OneOrMany<Shell>>,
    shell_exclude: Option<OneOrMany<Shell>>,

    #[serde(alias = "platformInclude")]
    platform: Option<OneOrMany<Platform>>,
    platform_exclude: Option<OneOrMany<Platform>>,
}

impl Conditional {
    fn command(self, shell: Shell) -> String {
        let command = match shell {
            Shell::Bash => self.cmd_bash,
            Shell::Fish => self.cmd_fish,
            Shell::Pwsh => self.cmd_pwsh,
            Shell::PwshConflict => self.cmd_pwsh_conflict.or(self.cmd_pwsh),
            Shell::Zsh => self.cmd_zsh,
        };

        command.unwrap_or(self.cmd)
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T: PartialEq> OneOrMany<T> {
    fn contains(&self, value: &T) -> bool {
        match self {
            Self::One(item) => item == value,
            Self::Many(items) => items.contains(value),
        }
    }
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Platform {
    Windows,
    Linux,
    Macos,
}

pub fn load_config(shell: Shell, path: Option<&str>) -> Result<IndexMap<String, String>, String> {
    let custom_path = path.is_some();
    let home = || std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    let path = match path {
        Some("~") => home().map(PathBuf::from),
        Some(path) if path.starts_with("~/") => {
            home().map(|home| PathBuf::from(home).join(&path[2..]))
        }
        Some(path) => Some(PathBuf::from(path)),
        None => home().map(|home| PathBuf::from(home).join(".config/shaka/config.json")),
    };
    let Some(path) = path else {
        return Err("home directory not found".into());
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if !custom_path && error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(IndexMap::new());
        }
        Err(error) => return Err(format!("failed reading {}: {error}", path.display())),
    };
    let config: Config = serde_json::from_str(&content)
        .map_err(|error| format!("failed parsing JSON {}: {error}", path.display()))?;
    let platform = match std::env::consts::OS {
        "windows" => Platform::Windows,
        "linux" => Platform::Linux,
        "macos" => Platform::Macos,
        value => return Err(format!("unrecognized platform: {value}")),
    };
    let values = match config {
        Config::Map(values) => values.into_iter().collect(),
        Config::Pairs(values) => values,
    };
    let mut entries = IndexMap::new();

    for (name, value) in values {
        let command = match value {
            Value::Command(command) => Some(command),
            Value::Conditional(value) => {
                if value.shell.is_some() && value.shell_exclude.is_some() {
                    return Err("shell include and exclude are mutually exclusive".into());
                }
                if value.platform.is_some() && value.platform_exclude.is_some() {
                    return Err("platform include and exclude are mutually exclusive".into());
                }

                let shell_matches = value
                    .shell
                    .as_ref()
                    .is_none_or(|values| values.contains(&shell))
                    && value
                        .shell_exclude
                        .as_ref()
                        .is_none_or(|values| !values.contains(&shell));
                let platform_matches = value
                    .platform
                    .as_ref()
                    .is_none_or(|values| values.contains(&platform))
                    && value
                        .platform_exclude
                        .as_ref()
                        .is_none_or(|values| !values.contains(&platform));

                (shell_matches && platform_matches).then(|| value.command(shell))
            }
        };

        if let Some(command) = command {
            entries.shift_remove(&name);
            entries.insert(name, command);
        }
    }

    Ok(entries)
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
