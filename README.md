# Shaka

Shaka generates shell aliases and PowerShell functions from one JSON config. It supports Bash, Fish, PowerShell, and Zsh, with optional shell-specific commands and platform filters.

## Supported shells

- `bash`: Bash
- `fish`: Fish
- `zsh`: Zsh
- `pwsh`: PowerShell
- `pwsh-conflict`: PowerShell without removing existing aliases

These values are used by the CLI, shell-specific command overrides, and shell filters.

## Installation

### Cargo

```sh
cargo install shaka
```

### mise

```sh
mise use -g cargo:shaka
```

## Configuration

Shaka reads `~/.config/shaka/config.json` by default. A basic config maps alias names to commands:

```json
{
  "g": "git",
  "dc": "docker compose",
  "ll": "ls -la"
}
```

Pass a different file as the second argument when needed:

```sh
shaka bash ~/.config/shaka.json
```

### Shell-specific commands

Use an object when an alias needs a different command for a particular shell. `cmd` is required and acts as the fallback.

```json
{
  "where": {
    "cmd": "which",
    "cmd.bash": "type -a",
    "cmd.fish": "type -a",
    "cmd.pwsh": "Get-Command",
    "cmd.zsh": "whence -a"
  }
}
```

Use `cmd.<shell>` with any supported shell value listed above. PowerShell conflict mode uses `cmd.pwsh` unless `cmd.pwsh-conflict` is also set.

### Filters

Aliases can be limited by shell or platform. Each filter accepts one value or an array.

```json
{
  "ll": {
    "cmd": "ls -la",
    "shell": ["bash", "zsh"]
  },
  "copy": {
    "cmd": "pbcopy",
    "platform": "macos"
  },
  "search": {
    "cmd": "rg",
    "shellExclude": "pwsh"
  }
}
```

Shell filters accept any supported shell value listed above. Platform values are `windows`, `linux`, and `macos`.

Available filters are `shell`, `shellExclude`, `platform`, and `platformExclude`. `shellInclude` and `platformInclude` are accepted aliases for `shell` and `platform`. Include and exclude filters for the same category cannot be used together.

### Repeated aliases

Use a top-level array of name-value pairs to define the same alias for different conditions:

```json
[
  ["open", { "cmd": "open", "platform": "macos" }],
  ["open", { "cmd": "xdg-open", "platform": "linux" }]
]
```

## Shell setup

Add the command for your shell to its startup file.

### Bash

Add to `~/.bashrc`:

```sh
eval "$(shaka bash)"
```

### Zsh

Add to `~/.zshrc`:

```sh
eval "$(shaka zsh)"
```

### Fish

Add to `~/.config/fish/config.fish`:

```fish
shaka fish | source
```

### PowerShell

Add to your PowerShell profile:

```powershell
Invoke-Expression (& shaka pwsh | Out-String)
```

Use `pwsh-conflict` instead of `pwsh` to keep existing PowerShell aliases rather than removing them before defining Shaka functions:

```powershell
Invoke-Expression (& shaka pwsh-conflict | Out-String)
```

## Usage

```text
shaka <bash|fish|pwsh|pwsh-conflict|zsh> [config-file]
```

Shaka writes shell code to standard output so it can be evaluated by the current shell. A missing default config produces no output; a missing explicitly provided config returns an error.

## License

[MIT](LICENSE)
