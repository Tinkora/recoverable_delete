use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn binary_name() -> &'static str {
    if cfg!(windows) {
        "recoverable-delete.exe"
    } else {
        "recoverable-delete"
    }
}

fn prepare_package(case_name: &str) -> PathBuf {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let package = repository
        .join("target/release_artifact_tests")
        .join(std::process::id().to_string())
        .join(case_name)
        .join("recoverable-delete");
    let plugin_source = repository.join("plugins/recoverable-delete");

    for relative_path in [
        ".codex-plugin/plugin.json",
        "hooks/dispatch_hook.ps1",
        "hooks/dispatch_hook.sh",
        "hooks/hooks.json",
        "skills/recoverable-delete/SKILL.md",
        "skills/recoverable-delete/agents/openai.yaml",
    ] {
        let source = plugin_source.join(relative_path);
        let destination = package.join(relative_path);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(source, destination).unwrap();
    }

    let binary = package.join("bin").join(binary_name());
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_recoverable-delete"), binary).unwrap();

    package
}

fn run_release_tool(package: &Path, archive: &Path) -> std::process::Output {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let python = if cfg!(windows) { "python" } else { "python3" };

    Command::new(python)
        .arg(repository.join("scripts/build_release_artifact.py"))
        .arg("--package-root")
        .arg(package)
        .arg("--archive")
        .arg(archive)
        .arg("--expected-version")
        .arg(env!("CARGO_PKG_VERSION"))
        .arg("--binary-name")
        .arg(binary_name())
        .output()
        .expect("release tool should start")
}

#[test]
fn creates_deterministic_archive_checksum_and_content_manifest() {
    let package = prepare_package("valid");
    let output_root = package.parent().unwrap();
    let first_archive = output_root.join("recoverable-delete-test-a.tar.gz");
    let second_archive = output_root.join("recoverable-delete-test-b.tar.gz");

    let first = run_release_tool(&package, &first_archive);
    assert!(
        first.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&first.stderr)
    );
    let second = run_release_tool(&package, &second_archive);
    assert!(
        second.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    assert_eq!(
        fs::read(&first_archive).unwrap(),
        fs::read(&second_archive).unwrap()
    );

    let checksum = fs::read_to_string(format!("{}.sha256", first_archive.display())).unwrap();
    assert!(checksum.ends_with("  recoverable-delete-test-a.tar.gz\n"));

    let contents = fs::read_to_string(format!("{}.contents.txt", first_archive.display())).unwrap();
    assert!(contents.contains("version: 0.1.0\n"));
    assert!(contents.contains("recoverable-delete/.codex-plugin/plugin.json\n"));
    assert!(contents.contains(&format!("recoverable-delete/bin/{}\n", binary_name())));
}

#[test]
fn rejects_unexpected_package_files() {
    let package = prepare_package("unexpected_file");
    fs::write(package.join("source_cache.rs"), "not distributable").unwrap();
    let archive = package.parent().unwrap().join("unexpected.tar.gz");

    let output = run_release_tool(&package, &archive);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected package file"));
    assert!(!archive.exists());
}

#[test]
fn rejects_plugin_version_mismatch() {
    let package = prepare_package("version_mismatch");
    let manifest_path = package.join(".codex-plugin/plugin.json");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap()
        .replace("\"version\": \"0.1.0\"", "\"version\": \"9.9.9\"");
    fs::write(manifest_path, manifest).unwrap();
    let archive = package.parent().unwrap().join("version_mismatch.zip");

    let output = run_release_tool(&package, &archive);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("version mismatch"));
    assert!(!archive.exists());
}
