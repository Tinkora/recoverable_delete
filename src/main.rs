use std::io::{self, Read};
use std::process::ExitCode;

fn main() -> ExitCode {
    if std::env::args().nth(1).as_deref() != Some("hook") {
        eprintln!("usage: recoverable-delete hook");
        return ExitCode::from(2);
    }

    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("failed to read hook input");
        return ExitCode::from(2);
    }

    if let Some(response) = recoverable_delete::hook_response(&input) {
        println!("{response}");
    }

    ExitCode::SUCCESS
}
