<h1 align="center">SHAKA</h1>

<p align="center"><strong>One config for every shell shortcut.</strong></p>

<p align="center">
  Generate aliases and functions for <code>bash</code>, <code>zsh</code>, <code>fish</code>, and PowerShell from a single YAML or JSONC config.
</p>

## Why shaka?

Define your shell shortcuts once and generate the right output for every shell. No more aliases duplicated across `.zshrc`, `.bashrc`, and PowerShell profiles, drifting out of sync.

- Single source of truth for bash, zsh, fish, and PowerShell
- Project-level overrides for repository-specific commands
- Built-in PowerShell conflict handling

## Quick Start

1. Create `~/.config/shaka.yaml`:

   ```yaml
   dc: docker compose
   gs: git status
   ```

2. Evaluate the generated code in your shell (swap `zsh` for your shell):

   ```sh
   eval "$(shaka zsh)"
   ```

   This makes `gs`, `dc`, etc. available in the current session. Add the same line to your shell profile to load them automatically.

## Installation

```sh
# Linux/macOS
curl -fsSL https://github.com/NazmusSayad/shaka/raw/main/install.sh | sh

# Windows (PowerShell)
(Invoke-WebRequest -UseBasicParsing https://github.com/NazmusSayad/shaka/raw/main/install.ps1).Content | Invoke-Expression
```

The installers detect OS/architecture, download the latest release, verify checksums, replace older versions, and give PATH guidance.

From source:

```sh
cargo install --path .   # local checkout
cargo install shaka      # from crates.io
cargo run -- zsh         # development
```

## Usage

`shaka` prints shell code to stdout. Load it per shell:

```sh
eval "$(shaka bash)"
eval "$(shaka zsh)"
shaka fish | source
Invoke-Expression (& shaka pwsh | Out-String)
```

Valid arguments: `bash`, `zsh`, `fish`, `pwsh`, `pwsh-conflict`. A missing or unsupported argument exits with an error and the usage string.

## Configuration

Files are loaded in this order; later files override earlier ones by key.

| Scope   | Paths                                                                            |
| ------- | -------------------------------------------------------------------------------- |
| Global  | `~/.config/shaka.yaml`, `~/.config/shaka.json`, `~/.shaka.yaml`, `~/.shaka.json` |
| Project | `./.shaka.yaml`, `./.shaka.json` (higher priority than global)                   |

So personal defaults live in your home directory, and a repository can override or add commands locally:

```yaml
# ~/.config/shaka.yaml
dc: docker compose
ls: eza
```

```yaml
# ./.shaka.yaml
dc: docker compose -f dev.yml # replaces the global dc
test: cargo test
```

### Format

YAML or JSONC:

```yaml
dc: docker compose
gs: git status
```

```jsonc
{
  // comments are allowed
  "dc": "docker compose",
  "gs": "git status",
}
```

### Conditional entries

A value can be an object with a required `cmd` plus optional `platform` and/or `shell` filters. The entry is emitted only when the current platform and shell match; otherwise it is dropped before merging (so it never shadows a matching entry from an earlier file).

- `platform` — `windows`, `linux`, or `macos`
- `shell` — `bash`, `zsh`, `fish`, `pwsh`, or `pwsh-conflict`

Both accept a single value or a list; when both are given, both must match. A plain string applies everywhere. An unknown name is a configuration error.

```yaml
gs: git status # all platforms and shells
ll:
  cmd: eza -l
  platform: [linux, macos]
open:
  cmd: explorer .
  platform: windows
  shell: pwsh
```

## Output

`bash`, `zsh`, and `fish` render aliases:

```sh
alias dc='docker compose'
```

PowerShell renders functions. By default `shaka pwsh` removes any existing alias of the same name first, avoiding conflicts with built-ins:

```sh
Remove-Alias -Name dc -Force -ErrorAction SilentlyContinue
function dc { docker compose @args }
```

To keep built-in aliases and emit only functions, use `pwsh-conflict`:

```sh
function dc { docker compose @args }
```

### PowerShell variable expansion

In `pwsh` mode only, `shaka` expands environment variables (`$NAME` and `$env:NAME`) in command values before rendering. Missing variables are left unchanged. This keeps machine-specific paths out of your config:

```yaml
n: $HOME/.local/bin/node
```

```sh
function n { C:/Users/you/.local/bin/node @args }
```
