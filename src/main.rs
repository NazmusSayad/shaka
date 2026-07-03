use std::process::ExitCode;

use shaka::{config, render};

fn main() -> ExitCode {
    let Some(shell) = std::env::args().nth(1) else {
        eprintln!("usage: shaka <bash|fish|pwsh|pwsh-conflict|zsh>");
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
            eprintln!("usage: shaka <bash|fish|pwsh|pwsh-conflict|zsh>");
            return ExitCode::from(1);
        }
    };

    let entries = match config::load_merged_config() {
        Ok(entries) => entries,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(1);
        }
    };

    print!("{}", render::render(target, &entries));
    ExitCode::SUCCESS
}
