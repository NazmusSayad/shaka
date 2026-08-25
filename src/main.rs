use shaka::{config, render};
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(shell) = std::env::args().nth(1) else {
        eprintln!("usage: shaka <bash|fish|pwsh|pwsh-conflict|zsh> [config]");
        return ExitCode::from(1);
    };

    let target = match shell.as_str() {
        "zsh" => render::Shell::Zsh,
        "bash" => render::Shell::Bash,
        "fish" => render::Shell::Fish,

        "pwsh" => render::Shell::Pwsh,
        "pwsh-conflict" => render::Shell::PwshConflict,
        _ => {
            eprintln!("unsupported shell: {shell}");
            eprintln!("usage: shaka <bash|fish|pwsh|pwsh-conflict|zsh> [config]");
            return ExitCode::from(1);
        }
    };

    let path = std::env::args().nth(2);
    let entries = match config::load_config(target, path.as_deref()) {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(1);
        }
    };

    print!("{}", render::render(target, &entries));
    ExitCode::SUCCESS
}
