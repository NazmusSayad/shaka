use super::render;
use indexmap::IndexMap;

#[test]
fn renders_fish_aliases() {
    let mut entries = IndexMap::new();
    entries.insert("dc".to_string(), "docker compose".to_string());
    assert_eq!(render(&entries), "alias dc 'docker compose'\n");
}
