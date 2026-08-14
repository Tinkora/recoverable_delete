#![cfg(unix)]

use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn run_dispatch(input: &str, binary: Option<&str>, plugin_root: &str) -> Output {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("plugins/recoverable-delete/hooks/dispatch_hook.sh");
    let mut command = Command::new("sh");
    command
        .arg(script)
        .env("PLUGIN_ROOT", plugin_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(binary) = binary {
        command.env("RECOVERABLE_DELETE_BIN", binary);
    }

    let mut child = command.spawn().expect("dispatch hook should start");
    let write_result = child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(input.as_bytes());
    if let Err(error) = write_result {
        assert_eq!(
            error.kind(),
            ErrorKind::BrokenPipe,
            "hook input should be written"
        );
    }
    child
        .wait_with_output()
        .expect("dispatch hook should finish")
}

#[test]
fn dispatches_to_the_policy_binary() {
    let input = r#"{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"rm -f prompt.txt"}}"#;
    let output = run_dispatch(
        input,
        Some(env!("CARGO_BIN_EXE_recoverable-delete")),
        env!("CARGO_MANIFEST_DIR"),
    );

    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains(r#""permissionDecision":"deny""#)
    );
}

#[test]
fn blocks_all_matched_tools_when_the_policy_binary_is_missing() {
    let output = run_dispatch(
        r#"{"hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"git status"}}"#,
        Some("/path/that/does/not/exist/recoverable-delete"),
        "/path/that/does/not/exist/plugin",
    );

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(r#""permissionDecision":"deny""#));
    assert!(stdout.contains("policy binary is unavailable"));
}
