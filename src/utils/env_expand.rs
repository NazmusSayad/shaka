pub fn expand_pwsh_env_vars(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::new();
    let mut copied = 0;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'$' {
            i += 1;
            continue;
        }

        let start = i + 1;
        let mut end = start;
        while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
            end += 1;
        }
        if start == end {
            i += 1;
            continue;
        }

        out.push_str(&input[copied..i]);
        let name = &input[start..end];
        if let Ok(value) = std::env::var(name) {
            out.push_str(&value.replace('\\', "/"));
        } else {
            out.push_str(&input[i..end]);
        }
        copied = end;
        i = end;
    }

    out.push_str(&input[copied..]);
    out
}

#[cfg(test)]
#[path = "env_expand_test.rs"]
mod tests;
