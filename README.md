<h1 align="center">SHAKA</h1>

<p align="center"><strong>One config for every shell shortcut.</strong></p>

<p align="center">
  Generate aliases and functions for <code>bash</code>, <code>zsh</code>, <code>fish</code>, and PowerShell from a single YAML or JSONC config.
</p>

## Why you should choose shaka?

Managing shell shortcuts across multiple shells (bash, zsh, fish, PowerShell) becomes fragmented and hard to maintain. Aliases defined in `.zshrc` don't apply to bash, project-specific commands scatter across different files, and keeping them in sync is tedious.

`shaka` solves this by letting you define all your shortcuts once in a single YAML or JSONC file and generate the appropriate shell-specific output. Benefits include:

- Single source of truth for all commands
- Automatic generation for bash, zsh, fish, and PowerShell
- Project-level overrides for repository-specific commands
- Built-in PowerShell compatibility (no manual alias conflict resolution)

## Quick Start

The example below uses `zsh`, but the same workflow applies to the other supported shells.

1. Create a global config file at `~/.config/shaka.yaml`:

```yaml
dc: docker compose
gs: git status
```

2. Ask `shaka` to generate shell code for your shell:

```sh
shaka zsh
```

Example output:

```sh
alias dc='docker compose'
alias gs='git status'
```

3. Evaluate that output in your shell so the aliases become available in the current session:

```sh
eval "$(shaka zsh)"
```

4. Use the alias:

```sh
gs
```

Example behavior:

```sh
$ gs
git status
On branch main
nothing to commit, working tree clean
```

To make this automatic every time you open a shell, add the same `eval "$(shaka zsh)"` line to your shell profile.

## Installation

Install the latest GitHub release with the platform installer script:

- Linux/macOS:

  ```sh
  curl -fsSL https://github.com/NazmusSayad/shaka/raw/main/install.sh | sh
  ```

- Windows (PowerShell):

  ```powershell
  (Invoke-WebRequest -UseBasicParsing https://github.com/NazmusSayad/shaka/raw/main/install.ps1).Content | Invoke-Expression
  ```

The installers detect OS/architecture, download the latest release archive, verify checksums, replace previous installed versions, and handle PATH guidance.

Install from the current source checkout:

```sh
cargo install --path .
```

If `shaka` is published on crates.io, install it as a package:

```sh
cargo install shaka
```

Build and run locally during development:

```sh
cargo run -- zsh
```

## Usage

`shaka` prints shell code to standard output:

```sh
shaka <bash|fish|pwsh|pwsh-conflict|zsh>
```

Typical usage:

- `bash`

  ```sh
  eval "$(shaka bash)"
  ```

- `zsh`

  ```sh
  eval "$(shaka zsh)"
  ```

- `fish`

  ```sh
  shaka fish | source
  ```

- `pwsh`

  ```sh
  Invoke-Expression (& shaka pwsh | Out-String)
  ```

If the shell argument is missing or unsupported, `shaka` exits with an error and prints the expected usage string.

## Supported Shells

- `bash`
- `fish`
- `pwsh`
- `zsh`

## Configuration

`shaka` loads configuration files in the following order. Later files override earlier files.

### Global

User-level configuration:

- `~/.config/shaka.yaml`
- `~/.config/shaka.json`
- `~/.shaka.yaml`
- `~/.shaka.json`

### Project

Project-level configuration from the current directory. These files have higher priority than global configuration:

- `./.shaka.yaml`
- `./.shaka.json`

## Configuration Format

`shaka` accepts either YAML or JSONC.

Map form:

```yaml
dc: docker compose
gs: git status
```

Pair-list form:

```yaml
- [dc, docker compose]
- [gs, git status]
```

JSONC form:

```jsonc
{
  // comments are allowed
  "dc": "docker compose",
  "gs": "git status",
}
```

### Conditional entries

A value can also be an object with a `cmd` field plus optional `platform` and/or `shell` filters. The entry is only emitted when the current platform and shell match; otherwise it is dropped before merging.

- `cmd` — the command string (required).
- `platform` — restrict to `windows`, `linux`, or `macos`.
- `shell` — restrict to `bash`, `zsh`, `fish`, `pwsh`, or `pwsh-conflict`.

Both `platform` and `shell` accept a single value or a list. When both are given, the entry applies only if both match. A plain string value has no restrictions and applies everywhere.

```yaml
gs: git status # raw string, all platforms and shells
ll:
  cmd: eza -l
  platform: [linux, macos]
open:
  cmd: explorer .
  platform: windows
  shell: pwsh
```

The same in JSONC:

```jsonc
{
  "gs": "git status",
  "ll": { "cmd": "eza -l", "platform": ["linux", "macos"] },
  "open": { "cmd": "explorer .", "platform": "windows", "shell": "pwsh" }
}
```

`shell` matches the exact argument you pass to `shaka` — `pwsh` and `pwsh-conflict` are distinct tokens. An unknown `platform` or `shell` name is a configuration error.

Because entries are filtered per file before merging, an entry that does not apply to the current platform/shell never shadows a matching entry with the same key from an earlier file.

## Precedence

`shaka` loads global configuration first and then applies project-level configuration on top of it. In practice, this means your personal defaults can live in your home directory, while a repository can override or add commands locally without changing your global setup.

For example, you might keep this in your global config:

```yaml
# ~/.config/shaka.yaml
dc: docker compose
ls: eza
```

Then, inside a specific project, you might define:

```yaml
# ./.shaka.yaml
dc: docker compose -f dev.yml
test: cargo test
```

When `shaka` merges these files, the project value for `dc` replaces the global one, while the other commands remain available. The final result behaves as if you had written:

```yaml
ls: eza
dc: docker compose -f dev.yml
test: cargo test
```

## Output

`shaka` outputs shell code that you evaluate in your shell profile or startup script.

For the config:

```yaml
dc: docker compose
```

`bash`, `zsh`, and `fish` render aliases:

```sh
alias dc='docker compose'
```

PowerShell renders functions:

```sh
Remove-Alias -Name dc -Force -ErrorAction SilentlyContinue
function dc { docker compose @args }
```

## PowerShell Modes

By default, `shaka pwsh` removes an existing alias with the same name before defining the function. This avoids conflicts with built-in aliases.

If you want to keep built-in aliases and only emit functions, use:

```sh
Invoke-Expression (& shaka pwsh-conflict | Out-String)
```

That renders:

```sh
function dc { docker compose @args }
```

`shaka` uses `Remove-Alias` for cleanup in PowerShell mode.

## PowerShell Environment Variable Expansion

In `pwsh` output mode, `shaka` expands environment variables inside command values before rendering functions. This is useful when your shortcuts depend on machine-specific locations such as your home directory or application install paths.

- Supported forms: `$NAME` and `$env:NAME`
- Missing variables are left unchanged
- Expansion is only applied for `pwsh`; `bash`, `fish`, and `zsh` outputs are unchanged

Example use case:

You want a shortcut that opens your projects directory in your editor without hardcoding your user-specific home path.

Example config:

```yaml
n: $HOME/.local/bin/node
```

Generated PowerShell function:

```sh
function n { C:/User/.local/bin/node @args }
```

This keeps the config portable while still producing a concrete command at render time. It is helpful when the command should stay the same logically, but the underlying absolute path differs from one machine to another.
