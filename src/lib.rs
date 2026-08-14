use serde_json::{Value, json};

pub fn hook_response(input: &str) -> Option<Value> {
    let Ok(payload) = serde_json::from_str::<Value>(input) else {
        return Some(deny(
            "Malformed hook input was blocked instead of failing open.",
        ));
    };

    if payload.get("hook_event_name").and_then(Value::as_str) != Some("PreToolUse") {
        return Some(deny(
            "Incomplete or unexpected hook input was blocked instead of failing open.",
        ));
    }

    let Some(tool_name) = payload.get("tool_name").and_then(Value::as_str) else {
        return Some(deny(
            "Incomplete or unexpected hook input was blocked instead of failing open.",
        ));
    };
    let Some(tool_input) = payload.get("tool_input") else {
        return Some(deny(
            "Incomplete or unexpected hook input was blocked instead of failing open.",
        ));
    };

    if is_shell_tool(tool_name) {
        let command = ["command", "cmd", "script"]
            .into_iter()
            .find_map(|key| tool_input.get(key).and_then(Value::as_str));

        let Some(command) = command else {
            return Some(deny(
                "Shell hook input without a command was blocked instead of failing open.",
            ));
        };

        if is_destructive_shell_command(command) {
            return Some(deny(
                "Permanent deletion command blocked. Move the exact targets to the operating system Trash.",
            ));
        }

        return None;
    }

    if is_patch_tool(tool_name) && serialized_input(tool_input).contains("*** Delete File:") {
        return Some(deny(
            "apply_patch file deletion blocked. Move the exact file to Trash instead.",
        ));
    }

    if is_patch_tool(tool_name) {
        return None;
    }

    Some(deny(
        "Unexpected matched tool was blocked instead of failing open.",
    ))
}

fn is_shell_tool(tool_name: &str) -> bool {
    ["bash", "exec_command", "shell", "terminal"]
        .iter()
        .any(|candidate| tool_name.eq_ignore_ascii_case(candidate))
}

fn is_patch_tool(tool_name: &str) -> bool {
    ["apply_patch", "edit", "write"]
        .iter()
        .any(|candidate| tool_name.eq_ignore_ascii_case(candidate))
}

fn serialized_input(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
}

fn deny(reason: &str) -> Value {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason
        }
    })
}

fn is_destructive_shell_command(command: &str) -> bool {
    split_shell_segments(command).into_iter().any(|segment| {
        let Ok(words) = shell_words::split(&segment) else {
            return true;
        };
        classify_words(&words)
    })
}

fn split_shell_segments(command: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut single_quoted = false;
    let mut double_quoted = false;
    let mut escaped = false;

    for character in command.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' && !single_quoted {
            current.push(character);
            escaped = true;
            continue;
        }
        match character {
            '\'' if !double_quoted => single_quoted = !single_quoted,
            '"' if !single_quoted => double_quoted = !double_quoted,
            ';' | '|' | '&' | '\n' if !single_quoted && !double_quoted => {
                if !current.trim().is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }
    }

    if !current.trim().is_empty() {
        segments.push(current);
    }
    segments
}

fn classify_words(words: &[String]) -> bool {
    let mut segment = Vec::new();
    for word in words {
        if is_shell_separator(word) {
            if classify_words_segment(&segment) {
                return true;
            }
            segment.clear();
        } else {
            segment.push(word.clone());
        }
    }

    classify_words_segment(&segment)
}

fn classify_words_segment(words: &[String]) -> bool {
    let Some((program, args)) = command_and_args(words) else {
        return false;
    };
    let program = executable_name(program);

    if matches!(
        program.as_str(),
        "bash" | "sh" | "zsh" | "pwsh" | "powershell" | "cmd"
    ) {
        if let Some(nested) = nested_command(program.as_str(), args) {
            return is_destructive_shell_command(&nested);
        }
    }

    if matches!(
        program.as_str(),
        "rm" | "rmdir" | "unlink" | "del" | "erase" | "rd" | "remove-item"
    ) {
        return true;
    }

    match program.as_str() {
        "busybox" => args.first().is_some_and(|arg| is_delete_program(arg)),
        "find" => {
            args.iter().any(|arg| arg.eq_ignore_ascii_case("-delete"))
                || args.iter().enumerate().any(|(index, arg)| {
                    arg.eq_ignore_ascii_case("-exec")
                        && args[index + 1..]
                            .iter()
                            .any(|nested| is_delete_program(nested))
                })
        }
        "git" => git_subcommand(args).is_some_and(|subcommand| subcommand == "clean"),
        "xargs" => args.iter().any(|arg| is_delete_program(arg)),
        "rtk" => classify_words(args.strip_prefix(&["proxy".to_owned()]).unwrap_or(args)),
        _ => false,
    }
}

fn is_shell_separator(word: &str) -> bool {
    matches!(word, ";" | "&&" | "||" | "|" | "|&" | "&")
}

fn is_delete_program(word: &str) -> bool {
    matches!(
        executable_name(word).as_str(),
        "rm" | "rmdir" | "unlink" | "del" | "erase" | "rd" | "remove-item"
    )
}

fn command_and_args(words: &[String]) -> Option<(&str, &[String])> {
    let mut index = 0;

    while let Some(word) = words.get(index) {
        let name = executable_name(word);
        match name.as_str() {
            "command" => {
                index += 1;
                while let Some(arg) = words.get(index) {
                    if matches!(arg.as_str(), "-v" | "-V") {
                        return None;
                    }
                    if arg == "--" {
                        index += 1;
                        break;
                    }
                    if !arg.starts_with('-') {
                        break;
                    }
                    index += 1;
                }
            }
            "sudo" | "doas" => {
                index += 1;
                while let Some(arg) = words.get(index) {
                    if arg == "--" {
                        index += 1;
                        break;
                    }
                    if arg == "--version" || arg == "-V" {
                        return None;
                    }
                    if !arg.starts_with('-') {
                        break;
                    }
                    let takes_value = matches!(
                        arg.as_str(),
                        "-C" | "-D" | "-g" | "-h" | "-p" | "-R" | "-T" | "-u"
                    );
                    index += if takes_value { 2 } else { 1 };
                }
            }
            "nohup" => {
                index += 1;
                if words.get(index).is_some_and(|arg| arg == "--") {
                    index += 1;
                }
            }
            "env" => {
                index += 1;
                while words
                    .get(index)
                    .is_some_and(|arg| arg.starts_with('-') || arg.contains('='))
                {
                    index += 1;
                }
            }
            _ => return Some((word, &words[index + 1..])),
        }
    }

    None
}

fn executable_name(program: &str) -> String {
    program
        .trim_matches(['\'', '"'])
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(program)
        .to_ascii_lowercase()
}

fn nested_command(program: &str, args: &[String]) -> Option<String> {
    let marker = if program == "cmd" { "/c" } else { "-c" };
    let marker_index = args.iter().position(|arg| {
        arg.eq_ignore_ascii_case(marker)
            || arg.eq_ignore_ascii_case("-lc")
            || arg.eq_ignore_ascii_case("-command")
    })?;

    Some(args[marker_index + 1..].join(" "))
}

fn git_subcommand(args: &[String]) -> Option<String> {
    let mut skip_next = false;

    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if matches!(arg.as_str(), "-C" | "-c" | "--git-dir" | "--work-tree") {
            skip_next = true;
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        return Some(arg.to_ascii_lowercase());
    }

    None
}
