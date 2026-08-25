use shaka::{config, render};
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(shell) = args.next() else {
        eprintln!("usage: shaka <bash|fish|pwsh|pwsh-conflict|zsh> [config-file]");
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
            eprintln!("usage: shaka <bash|fish|pwsh|pwsh-conflict|zsh> [config-file]");
            return ExitCode::from(1);
        }
    };

    let path = args.next();
    if args.next().is_some() {
        eprintln!("usage: shaka <bash|fish|pwsh|pwsh-conflict|zsh> [config-file]");
        return ExitCode::from(1);
    }
    let entries = match config::load_config(target, path.as_deref()) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(1);
        }
    };
    let output = render::render(target, &entries);

    if let Err(error) = io::stdout().write_all(output.as_bytes())
        && error.kind() != io::ErrorKind::BrokenPipe
    {
        eprintln!("failed writing output: {error}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}
