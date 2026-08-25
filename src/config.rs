use crate::render::Shell;
use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

enum ConfigForm {
    Map(BTreeMap<String, ConfigValue>),
    Pairs(Vec<(String, ConfigValue)>),
}

impl<'de> Deserialize<'de> for ConfigForm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FormVisitor;
        impl<'de> serde::de::Visitor<'de> for FormVisitor {
            type Value = ConfigForm;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a map or a sequence of pairs")
            }
            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let de = serde::de::value::MapAccessDeserializer::new(map);
                let m = BTreeMap::<String, ConfigValue>::deserialize(de)?;
                Ok(ConfigForm::Map(m))
            }
            fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
            where
                S: serde::de::SeqAccess<'de>,
            {
                let de = serde::de::value::SeqAccessDeserializer::new(seq);
                let v = Vec::<(String, ConfigValue)>::deserialize(de)?;
                Ok(ConfigForm::Pairs(v))
            }
        }
        deserializer.deserialize_any(FormVisitor)
    }
}

enum ConfigValue {
    Raw(String),
    Detailed(DetailedValue),
}

impl<'de> Deserialize<'de> for ConfigValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;
        impl<'de> serde::de::Visitor<'de> for ValueVisitor {
            type Value = ConfigValue;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a string or an object with `cmd`")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ConfigValue::Raw(v.to_owned()))
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ConfigValue::Raw(v))
            }
            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let de = serde::de::value::MapAccessDeserializer::new(map);
                let d = DetailedValue::deserialize(de)?;
                Ok(ConfigValue::Detailed(d))
            }
        }
        deserializer.deserialize_any(ValueVisitor)
    }
}

struct DetailedValue {
    cmd: String,
    shell_include: Option<OneOrMany<Shell>>,
    shell_exclude: Option<OneOrMany<Shell>>,
    platform_include: Option<OneOrMany<Platform>>,
    platform_exclude: Option<OneOrMany<Platform>>,
}

impl<'de> Deserialize<'de> for DetailedValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            cmd: String,
            #[serde(default, rename = "shell")]
            shell: Option<OneOrMany<Shell>>,
            #[serde(default, rename = "shellInclude")]
            shell_include: Option<OneOrMany<Shell>>,
            #[serde(default, rename = "shellExclude")]
            shell_exclude: Option<OneOrMany<Shell>>,
            #[serde(default, rename = "platform")]
            platform: Option<OneOrMany<Platform>>,
            #[serde(default, rename = "platformInclude")]
            platform_include: Option<OneOrMany<Platform>>,
            #[serde(default, rename = "platformExclude")]
            platform_exclude: Option<OneOrMany<Platform>>,
        }

        let helper = Helper::deserialize(deserializer)?;
        let shell_count = helper.shell.is_some() as u8
            + helper.shell_include.is_some() as u8
            + helper.shell_exclude.is_some() as u8;
        if shell_count > 1 {
            return Err(serde::de::Error::custom(
                "`shell`, `shellInclude` and `shellExclude` are mutually exclusive; only one may be specified",
            ));
        }
        let platform_count = helper.platform.is_some() as u8
            + helper.platform_include.is_some() as u8
            + helper.platform_exclude.is_some() as u8;
        if platform_count > 1 {
            return Err(serde::de::Error::custom(
                "`platform`, `platformInclude` and `platformExclude` are mutually exclusive; only one may be specified",
            ));
        }
        Ok(DetailedValue {
            cmd: helper.cmd,
            shell_include: helper.shell.or(helper.shell_include),
            shell_exclude: helper.shell_exclude,
            platform_include: helper.platform.or(helper.platform_include),
            platform_exclude: helper.platform_exclude,
        })
    }
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
                let platform_ok = if let Some(include) = &detail.platform_include {
                    include.contains(&platform)
                } else if let Some(exclude) = &detail.platform_exclude {
                    !exclude.contains(&platform)
                } else {
                    true
                };
                let shell_ok = if let Some(include) = &detail.shell_include {
                    include.contains(&shell)
                } else if let Some(exclude) = &detail.shell_exclude {
                    !exclude.contains(&shell)
                } else {
                    true
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
#[path = "config_test.rs"]
mod tests;
