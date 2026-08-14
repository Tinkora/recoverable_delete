#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn packages_a_runnable_plugin_for_the_current_unix_host() {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_root = repository
        .join("target/package_tests")
        .join(std::process::id().to_string());
    let script = repository.join("scripts/package_plugin.sh");

    let output = Command::new("sh")
        .arg(script)
        .arg(&output_root)
        .env(
            "RECOVERABLE_DELETE_BIN",
            env!("CARGO_BIN_EXE_recoverable-delete"),
        )
        .output()
        .expect("package script should start");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let plugin = output_root.join("recoverable-delete");
    for relative_path in [
        ".codex-plugin/plugin.json",
        "hooks/hooks.json",
        "skills/recoverable-delete/SKILL.md",
        "bin/recoverable-delete",
    ] {
        assert!(
            plugin.join(relative_path).is_file(),
            "missing {relative_path}"
        );
    }

    let mode = fs::metadata(plugin.join("bin/recoverable-delete"))
        .unwrap()
        .permissions()
        .mode();
    assert_ne!(mode & 0o111, 0, "packaged policy binary must be executable");
}
