use crate::utils::env_expand::expand_pwsh_env_vars;
use indexmap::IndexMap;

const STATEMENT_KEYWORDS: [&str; 5] = ["exit", "return", "break", "continue", "throw"];

fn ends_with_statement_command(command: &str) -> bool {
    let first = statement_tail(command)
        .and_then(|segment| segment.split_whitespace().next())
        .unwrap_or("");
    STATEMENT_KEYWORDS
        .iter()
        .any(|keyword| keyword.eq_ignore_ascii_case(first))
}

fn statement_tail(command: &str) -> Option<&str> {
    command
        .split([';', '\n', '\r'])
        .rev()
        .find(|segment| !segment.trim().is_empty())
        .map(|segment| segment.trim())
}

fn bare_keyword_split(command: &str) -> Option<(String, String)> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }
    let segments: Vec<&str> = trimmed.split([';', '\n', '\r']).collect();
    let last_idx = segments.iter().rposition(|s| !s.trim().is_empty())?;
    let tail = segments[last_idx].trim().to_string();
    if !STATEMENT_KEYWORDS
        .iter()
        .any(|kw| kw.eq_ignore_ascii_case(&tail))
    {
        return None;
    }
    let prefix = segments[..last_idx]
        .iter()
        .filter_map(|s| {
            let t = s.trim();
            if t.is_empty() { None } else { Some(t) }
        })
        .collect::<Vec<_>>()
        .join("; ");
    Some((prefix, tail))
}

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
        if let Some((prefix, keyword)) = bare_keyword_split(&expanded_command) {
            if !prefix.is_empty() {
                out.push_str(&prefix);
                out.push_str("; ");
            }
            out.push_str(&format!(
                "if ($args.Count -gt 1) {{ Write-Warning \"{name}: {keyword} expects at most 1 argument; received $($args.Count); using first\" }}; if ($args.Count -gt 0) {{ {keyword} $args[0] }} else {{ {keyword} }}"
            ));
        } else if ends_with_statement_command(&expanded_command) {
            out.push_str(&expanded_command);
        } else {
            out.push_str(&expanded_command);
            out.push_str(" @args");
        }
        out.push_str(" }\n");
    }

    out
}

#[cfg(test)]
#[path = "pwsh_test.rs"]
mod tests;
