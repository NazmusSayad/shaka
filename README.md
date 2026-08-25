<h1 align="center">SHAKA</h1>

<p align="center"><strong>One config for every shell shortcut.</strong></p>

<p align="center">
  Generate aliases and functions for Bash, Fish, PowerShell, and Zsh from a single JSON config.
</p>

## Why Shaka?

Define shell shortcuts once and generate the right output for every shell. No more aliases duplicated across shell profiles and drifting out of sync.

- One JSON source for every supported shell
- Per-shell command overrides
- Shell and platform filters
- Built-in PowerShell conflict handling

## Supported Shells

- `bash`: Bash
- `fish`: Fish
- `zsh`: Zsh
- `pwsh`: PowerShell
- `pwsh-conflict`: PowerShell without removing existing aliases

These values are used by the CLI, shell-specific command overrides, and shell filters.

## Quick Start

1. Create `~/.config/shaka/config.json`:

   ```json
   {
     "dc": "docker compose",
     "gs": "git status"
   }
   ```

2. Evaluate the generated code in your shell:

   ```sh
   eval "$(shaka zsh)"
   ```

   Replace `zsh` with your shell value. Add the command to your shell profile to load the shortcuts automatically.

## Installation

### Cargo

```sh
cargo install shaka
```

### mise

```sh
mise use -g cargo:shaka
```

## Usage

Shaka writes shell code to standard output:

```text
shaka <bash|fish|pwsh|pwsh-conflict|zsh> [config-file]
```

It reads `~/.config/shaka/config.json` by default. Pass another file as the second argument when needed:

```sh
shaka bash ~/.config/shaka.json
```

A missing default config produces no output. A missing explicitly provided config returns an error.

## Configuration

A plain string defines an alias for every shell and platform:

```json
{
  "g": "git",
  "dc": "docker compose",
  "ll": "ls -la"
}
```

### Shell-Specific Commands

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

Use `cmd.<shell>` with any supported shell value. PowerShell conflict mode uses `cmd.pwsh` unless `cmd.pwsh-conflict` is also set.

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

Shell filters accept any supported shell value. Platform values are `windows`, `linux`, and `macos`.

Available filters are `shell`, `shellExclude`, `platform`, and `platformExclude`. `shellInclude` and `platformInclude` are accepted aliases for `shell` and `platform`. Include and exclude filters for the same category cannot be used together.

### Repeated Aliases

Use a top-level array of name-value pairs to define the same alias for different conditions:

```json
[
  ["open", { "cmd": "open", "platform": "macos" }],
  ["open", { "cmd": "xdg-open", "platform": "linux" }]
]
```

## Shell Setup

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

Use `pwsh-conflict` to keep existing PowerShell aliases and emit only functions:

```powershell
Invoke-Expression (& shaka pwsh-conflict | Out-String)
```

## License

[MIT](LICENSE)
