use indexmap::IndexMap;

pub fn render(entries: &IndexMap<String, String>) -> String {
    let mut out = String::new();

    for (name, command) in entries {
        out.push_str("alias ");
        out.push_str(name);
        out.push_str(" '");
        out.push_str(&escape_single_quotes(command));
        out.push_str("'\n");
    }

    out
}

fn escape_single_quotes(input: &str) -> String {
    input.replace('\'', "'\\''")
}

#[cfg(test)]
#[path = "fish_test.rs"]
mod tests;
