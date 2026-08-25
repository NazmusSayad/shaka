use super::*;

fn conditional(json: &str) -> Conditional {
    serde_json::from_str(json).unwrap()
}

#[test]
fn uses_shell_command_overrides() {
    let json = r#"{
        "cmd": "base",
        "cmd.bash": "bash",
        "cmd.fish": "fish",
        "cmd.pwsh": "pwsh",
        "cmd.zsh": "zsh"
    }"#;

    assert_eq!(conditional(json).command(Shell::Bash), "bash");
    assert_eq!(conditional(json).command(Shell::Fish), "fish");
    assert_eq!(conditional(json).command(Shell::Pwsh), "pwsh");
    assert_eq!(conditional(json).command(Shell::PwshConflict), "pwsh");
    assert_eq!(conditional(json).command(Shell::Zsh), "zsh");
}

#[test]
fn falls_back_to_base_command() {
    assert_eq!(
        conditional(r#"{"cmd":"base"}"#).command(Shell::Bash),
        "base"
    );
}

#[test]
fn uses_pwsh_conflict_override_when_present() {
    let value = conditional(r#"{"cmd":"base","cmd.pwsh":"pwsh","cmd.pwsh-conflict":"conflict"}"#);

    assert_eq!(value.command(Shell::PwshConflict), "conflict");
}
