use super::expand_pwsh_env_vars;

fn known_var() -> (&'static str, String) {
    for name in ["HOME", "USERPROFILE", "PATH"] {
        if let Ok(value) = std::env::var(name) {
            return (name, value);
        }
    }
    panic!("no known env var found for tests");
}

fn known_var_two() -> ((&'static str, String), (&'static str, String)) {
    let mut found = Vec::new();
    for name in ["HOME", "USERPROFILE", "PATH", "TEMP"] {
        if let Ok(value) = std::env::var(name) {
            found.push((name, value));
        }
        if found.len() == 2 {
            return (found.remove(0), found.remove(0));
        }
    }
    panic!("not enough env vars found for tests");
}

#[test]
fn expands_dollar_var() {
    let (name, value) = known_var();
    let value = value.replace('\\', "/");
    assert_eq!(
        expand_pwsh_env_vars(&format!("${name}/bin")),
        format!("{value}/bin")
    );
}

#[test]
fn expands_multiple_vars_in_one_string() {
    let ((a_name, a_value), (b_name, b_value)) = known_var_two();
    let a_value = a_value.replace('\\', "/");
    let b_value = b_value.replace('\\', "/");
    assert_eq!(
        expand_pwsh_env_vars(&format!("${a_name}:${b_name}")),
        format!("{a_value}:{b_value}")
    );
}

#[test]
fn expands_adjacent_vars() {
    let ((a_name, a_value), (b_name, b_value)) = known_var_two();
    let a_value = a_value.replace('\\', "/");
    let b_value = b_value.replace('\\', "/");
    assert_eq!(
        expand_pwsh_env_vars(&format!("${a_name}${b_name}")),
        format!("{a_value}{b_value}")
    );
}

#[test]
fn keeps_unknown_dollar_var_literal() {
    assert_eq!(
        expand_pwsh_env_vars("$THIS_SHOULD_NOT_EXIST_12345/bin"),
        "$THIS_SHOULD_NOT_EXIST_12345/bin"
    );
}

#[test]
fn leaves_dollar_env_prefix_form_unchanged() {
    assert_eq!(expand_pwsh_env_vars("$env:HOME/bin"), "$env:HOME/bin");
}

#[test]
fn keeps_lone_dollar_literal() {
    assert_eq!(expand_pwsh_env_vars("$"), "$");
}

#[test]
fn keeps_dollar_before_non_var_chars_literal() {
    assert_eq!(expand_pwsh_env_vars("$- $/ $ "), "$- $/ $ ");
}

#[test]
fn respects_var_boundaries_with_punctuation() {
    let (name, value) = known_var();
    let value = value.replace('\\', "/");
    assert_eq!(
        expand_pwsh_env_vars(&format!("${name}.suffix")),
        format!("{value}.suffix")
    );
    assert_eq!(
        expand_pwsh_env_vars(&format!("${name}/path")),
        format!("{value}/path")
    );
}

#[test]
fn supports_underscore_and_digits_in_var_name() {
    assert_eq!(
        expand_pwsh_env_vars("$THIS_VAR_99/test"),
        "$THIS_VAR_99/test"
    );
}

#[test]
fn handles_empty_input() {
    assert_eq!(expand_pwsh_env_vars(""), "");
}

#[test]
fn leaves_strings_without_vars_unchanged() {
    assert_eq!(expand_pwsh_env_vars("docker compose"), "docker compose");
}

#[test]
fn expands_mixed_literal_var_literal() {
    let (name, value) = known_var();
    let value = value.replace('\\', "/");
    assert_eq!(
        expand_pwsh_env_vars(&format!("prefix-${name}-suffix")),
        format!("prefix-{value}-suffix")
    );
}
