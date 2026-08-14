use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run_hook(input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_recoverable-delete"))
        .arg("hook")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("hook binary should start");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes())
        .expect("hook input should be written");

    child.wait_with_output().expect("hook should finish")
}

fn assert_denied(input: &str) {
    let output = run_hook(input);
    let stdout = String::from_utf8(output.stdout).expect("hook output should be UTF-8");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains(r#""permissionDecision":"deny""#),
        "input: {input}; stdout: {stdout}"
    );
}

#[test]
fn allows_non_destructive_shell_commands() {
    for command in ["git status --short", "command -v rm", "sudo --version"] {
        let output = run_hook(&format!(
            r#"{{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":{command:?}}}}}"#,
        ));

        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    }
}

#[test]
fn denies_permanent_shell_deletion_commands() {
    for command in [
        "rm -rf -- build/cache",
        "find . -type f -name '*.tmp' -delete",
        "git clean -fd",
        "bash -lc 'rm -rf -- build/cache'",
        "powershell -Command \"Remove-Item -Recurse build\\cache\"",
        "cmd /c del /q build\\cache.tmp",
        "git status --short && /bin/rm -f build/cache.tmp",
        "cd build; find . -name '*.tmp' -delete",
        "printf '%s\\0' build/cache.tmp | xargs -0 rm",
        "env BUILD_KIND=test /bin/rm -rf build/cache",
        "env -u BUILD_KIND /bin/rm -rf build/cache",
        "env -C build /bin/rm -rf cache",
        "command -- /bin/rm -rf build/cache",
        "sudo -u builder -- /bin/rm -rf build/cache",
        "sudo --user builder -- /bin/rm -rf build/cache",
        "nohup /bin/rm -rf build/cache",
        "busybox rm -rf build/cache",
        "rtk proxy rm -rf build/cache",
        "rsync -a --delete source/ destination/",
        "eval 'rm -rf -- build/cache'",
        "timeout 5 /bin/rm -rf build/cache",
        "nice -n 5 /bin/rm -rf build/cache",
        "time /bin/rm -rf build/cache",
        "time -f '%E' /bin/rm -rf build/cache",
        "setsid /bin/rm -rf build/cache",
        "printf '%s\\0' build/cache.tmp | xargs -0 sh -c 'rm -f -- \"$@\"' sh",
        "python3 -c 'import shutil; shutil.rmtree(\"build/cache\")'",
        "node -e 'require(\"fs\").rmSync(\"build/cache\", { recursive: true })'",
        "ruby -e 'require \"fileutils\"; FileUtils.rm_rf(\"build/cache\")'",
    ] {
        assert_denied(&format!(
            r#"{{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":{command:?}}}}}"#
        ));
    }
}

#[test]
fn allows_destructive_words_when_they_are_only_echoed_text() {
    for command in [
        "echo 'rm -rf build/cache'",
        "echo 'rsync --delete source destination'",
        "python3 -c 'print(\"cleanup complete\")'",
        "node -e 'console.log(\"cleanup complete\")'",
        "ruby -e 'puts \"cleanup complete\"'",
    ] {
        let output = run_hook(&format!(
            r#"{{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":{command:?}}}}}"#,
        ));

        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
    }
}

#[test]
fn denies_malformed_hook_input_instead_of_failing_open() {
    assert_denied("not-json");
}

#[test]
fn denies_incomplete_hook_input_instead_of_failing_open() {
    for input in [
        "{}",
        r#"{"hook_event_name":"PreToolUse","tool_name":"Bash"}"#,
        r#"{"hook_event_name":"PreToolUse","tool_input":{"command":"git status"}}"#,
    ] {
        assert_denied(input);
    }
}

#[test]
fn denies_apply_patch_file_deletion() {
    for key in ["patch", "command"] {
        assert_denied(&format!(
            r#"{{"hook_event_name":"PreToolUse","tool_name":"apply_patch","tool_input":{{{key:?}:"*** Begin Patch\n*** Delete File: prompt.txt\n*** End Patch"}}}}"#,
        ));
    }
}

#[test]
fn allows_apply_patch_without_file_deletion() {
    let output = run_hook(
        r#"{"hook_event_name":"PreToolUse","tool_name":"apply_patch","tool_input":{"patch":"*** Begin Patch\n*** Update File: README.md\n@@\n-old\n+new\n*** End Patch"}}"#,
    );

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
}

#[test]
fn allows_recoverable_trash_commands() {
    let output = run_hook(
        r#"{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"/usr/bin/trash -- build/cache"}}"#,
    );

    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
}
