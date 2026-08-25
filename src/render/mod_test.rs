use super::{render, Shell};
use indexmap::IndexMap;

#[test]
fn dispatches_bash_to_sh_renderer() {
    let mut entries = IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    assert_eq!(render(Shell::Bash, &entries), "alias dc='docker compose'\n");
}

#[test]
fn dispatches_zsh_to_sh_renderer() {
    let mut entries = IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    assert_eq!(render(Shell::Zsh, &entries), "alias dc='docker compose'\n");
}
