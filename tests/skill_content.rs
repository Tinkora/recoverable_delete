use std::fs;
use std::path::PathBuf;

#[test]
fn macos_trash_example_matches_the_system_command() {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let skill = fs::read_to_string(
        repository.join("plugins/recoverable-delete/skills/recoverable-delete/SKILL.md"),
    )
    .unwrap();

    assert!(
        !skill.contains("/usr/bin/trash --"),
        "macOS /usr/bin/trash treats -- as a file path"
    );
    assert!(skill.contains("/usr/bin/trash <exact paths>"));
}
