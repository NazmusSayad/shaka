pub fn expand_pwsh_env_vars(input: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] != '$' {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        let mut j = i + 1;

        let start = j;
        while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') {
            j += 1;
        }

        if start == j {
            out.push('$');
            i += 1;
            continue;
        }

        let name: String = chars[start..j].iter().collect();
        if let Ok(value) = std::env::var(&name) {
            out.push_str(&value.replace('\\', "/"));
        } else {
            out.push('$');
            out.push_str(&name);
        }

        i = j;
    }

    out
}

#[cfg(test)]
#[path = "env_expand_test.rs"]
mod tests;
