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

#[test]
fn expands_var_with_prefix_and_suffix() {
    let (name, value) = known_var();
    let value = value.replace('\\', "/");
    assert_eq!(expand_pwsh_env_vars(&format!("pre-${name}-suf")), format!("pre-{value}-suf"));
}

#[test]
fn expands_multiple_same_var_twice() {
    let (name, value) = known_var();
    let value = value.replace('\\', "/");
    assert_eq!(expand_pwsh_env_vars(&format!("${name}:${name}")), format!("{value}:{value}"));
}

#[test]
fn expands_var_at_end() {
    let (name, value) = known_var();
    let value = value.replace('\\', "/");
    assert_eq!(expand_pwsh_env_vars(&format!("path/${name}")), format!("path/{value}"));
}

#[test]
fn expands_var_at_start() {
    let (name, value) = known_var();
    let value = value.replace('\\', "/");
    assert_eq!(expand_pwsh_env_vars(&format!("${name}/path")), format!("{value}/path"));
}

#[test]
fn does_not_expand_env_colon_form() {
    assert_eq!(expand_pwsh_env_vars("$env:PATH"), "$env:PATH");
    assert_eq!(expand_pwsh_env_vars("$env:HOME/test"), "$env:HOME/test");
}

#[test]
fn handles_backslash_in_value_replaced_with_slash() {
    unsafe { std::env::set_var("SHAKA_TEST_BACKSLASH", r"C:\foo\bar\baz") };
    let out = expand_pwsh_env_vars("$SHAKA_TEST_BACKSLASH/bin");
    assert_eq!(out, "C:/foo/bar/baz/bin");
    unsafe { std::env::remove_var("SHAKA_TEST_BACKSLASH") };
}

#[test]
fn unknown_var_stays_literal_with_underscore() {
    assert_eq!(expand_pwsh_env_vars("$UNKNOWN_1234_VAR_xyz"), "$UNKNOWN_1234_VAR_xyz");
}

#[test]
fn expands_two_vars_with_text_between() {
    let ((a_name, a_value), (b_name, b_value)) = known_var_two();
    let a_value = a_value.replace('\\', "/");
    let b_value = b_value.replace('\\', "/");
    let input = format!("start-${a_name}-mid-${b_name}-end");
    let expected = format!("start-{a_value}-mid-{b_value}-end");
    assert_eq!(expand_pwsh_env_vars(&input), expected);
}

#[test]
fn handles_consecutive_vars_no_sep() {
    let ((a_name, a_value), (b_name, b_value)) = known_var_two();
    let a_value = a_value.replace('\\', "/");
    let b_value = b_value.replace('\\', "/");
    assert_eq!(expand_pwsh_env_vars(&format!("${a_name}${b_name}")), format!("{a_value}{b_value}"));
}

#[test]
fn handles_long_string_with_many_vars() {
    let (name, value) = known_var();
    let value = value.replace('\\', "/");
    let input = format!("a-${name}-b-${name}-c-${name}-d");
    let expected = format!("a-{value}-b-{value}-c-{value}-d");
    assert_eq!(expand_pwsh_env_vars(&input), expected);
}

#[test]
fn handles_mixed_case_var_name() {
    unsafe { std::env::set_var("ShAkA_MiXeD", "mixedval") };
    assert_eq!(expand_pwsh_env_vars("$ShAkA_MiXeD"), "mixedval");
    unsafe { std::env::remove_var("ShAkA_MiXeD") };
}

#[test]
fn handles_numeric_suffix_in_var() {
    unsafe { std::env::set_var("SHAKA_VAR1", "val1") };
    unsafe { std::env::set_var("SHAKA_VAR2", "val2") };
    assert_eq!(expand_pwsh_env_vars("$SHAKA_VAR1/$SHAKA_VAR2"), "val1/val2");
    unsafe { std::env::remove_var("SHAKA_VAR1") };
    unsafe { std::env::remove_var("SHAKA_VAR2") };
}

#[test]
fn leaves_url_unchanged_if_no_var() {
    assert_eq!(expand_pwsh_env_vars("https://example.com/test"), "https://example.com/test");
}

#[test]
fn handles_var_with_underscore_start() {
    unsafe { std::env::set_var("_SHAKA_UNDERSCORE", "uscore") };
    assert_eq!(expand_pwsh_env_vars("$_SHAKA_UNDERSCORE"), "uscore");
    unsafe { std::env::remove_var("_SHAKA_UNDERSCORE") };
}

#[test]
fn stress_many_expansions() {
    let (name, value) = known_var();
    let value = value.replace('\\', "/");
    let mut input = String::new();
    let mut expected = String::new();
    for _ in 0..20 {
        input.push_str(&format!("${name}/"));
        expected.push_str(&format!("{value}/"));
    }
    assert_eq!(expand_pwsh_env_vars(&input), expected);
}

#[test]
fn handles_var_boundary_with_dot_and_slash() {
    let (name, value) = known_var();
    let value = value.replace('\\', "/");
    assert_eq!(expand_pwsh_env_vars(&format!("${name}.txt")), format!("{value}.txt"));
    assert_eq!(expand_pwsh_env_vars(&format!("${name}/a/b")), format!("{value}/a/b"));
}

#[test]
fn does_not_expand_partial_var() {
    assert_eq!(expand_pwsh_env_vars("$"), "$");
    assert_eq!(expand_pwsh_env_vars("$$"), "$$");
    assert_eq!(expand_pwsh_env_vars("$-"), "$-");
}

#[test]
fn handles_empty_and_only_dollar() {
    assert_eq!(expand_pwsh_env_vars(""), "");
    assert_eq!(expand_pwsh_env_vars("$"), "$");
    assert_eq!(expand_pwsh_env_vars("$$$"), "$$$");
}

#[test]
fn unknown_var_with_numbers() {
    assert_eq!(expand_pwsh_env_vars("$VAR123"), "$VAR123");
    unsafe { std::env::set_var("VAR123", "numval") };
    assert_eq!(expand_pwsh_env_vars("$VAR123"), "numval");
    unsafe { std::env::remove_var("VAR123") };
}

#[test]
fn expands_three_vars_in_row() {
    let (name, value) = known_var();
    let value = value.replace('\\', "/");
    unsafe { std::env::set_var("SHAKA_TMP_A", "aval") };
    unsafe { std::env::set_var("SHAKA_TMP_B", "bval") };
    let input = format!("${name}/$SHAKA_TMP_A/$SHAKA_TMP_B");
    let expected = format!("{value}/aval/bval");
    assert_eq!(expand_pwsh_env_vars(&input), expected);
    unsafe { std::env::remove_var("SHAKA_TMP_A") };
    unsafe { std::env::remove_var("SHAKA_TMP_B") };
}
